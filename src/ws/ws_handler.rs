use fr_rust::prelude::*;
use tokio::time::Instant;
use tokio::sync::mpsc;
use actix_ws::{Message, Session, MessageStream};
use actix_web::{get, web, web::Path};
use futures_util::{StreamExt, future::BoxFuture};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ClientCommand {
    Join { room: String },
    Leave { room: String },
    DropRoom { room: String },
    MsgUser { target_id: String, msg: String },
    MsgRoom { room: String, msg: String },
    Broadcast { msg: String },
}

/// Handles incoming client commands.
/// Performance Tip: Use `&str` for user_id to avoid heap allocations via `.to_string()`.
async fn handle_client_command(
    ws_manager: &WsManager,
    user_id: &str,
    text: &str,
) -> Result<(), serde_json::Error> {
    let command: ClientCommand = serde_json::from_str(text)?;

    match command {
        ClientCommand::Join { room } => {
            ws_manager.join_room(user_id, &room);
        }
        ClientCommand::Leave { room } => {
            ws_manager.leave_room(user_id, &room);
        }
        ClientCommand::DropRoom { room } => {
            // Drop room requirement: Fetch and print history before eviction
            if let Ok(messages) = ws_manager.get_room_msgs(&room).await {
                println!("--- History for Room [{}] before drop ---", room);
                for msg in messages {
                    println!("{:?}", msg);
                }
                println!("-----------------------------------------");
            }
            ws_manager.drop_room(&room);
        }
        ClientCommand::MsgUser { target_id, msg } => {
            ws_manager.msg_user(&target_id, msg);
        }
        ClientCommand::MsgRoom { room, msg } => {
            // Optimal allocation placement: string clone happens only inside this specific execution arm
            let user_msg = UserMsg::new(user_id.to_string(), room.clone(), msg);
            ws_manager.msg_room(&room, user_msg);
        }
        ClientCommand::Broadcast { msg } => {
            ws_manager.broadcast(msg);
        }
    }
    Ok(())
}

/// Centralized cleanup function executed when a user disconnects
fn cleanup_user(ws_manager: &WsManager, user_id: &str) {
    println!("Cleaning up connection for user: {}", user_id);
    ws_manager.drop_user(user_id);
}

#[get("/ws/{user_id}")]
async fn ws_handler(
    req: Rqs,
    body: Payload,
    ws_manager: web::Data<WsManager>,
    path: Path<String>,
) -> Rsp {
    let user_id = path.into_inner();

    // 1. Setup high-performance bounded channel (128 items is ideal for memory/backpressure balance)
    let (tx, mut rx) = mpsc::channel::<String>(128);

    // Perform WebSocket handshake
    let (res, mut session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(tuple) => tuple,
        Err(_) => return http_bad("Internal Server Error!"), 
    };

    // 2. Register user with the manager
    ws_manager.register(&user_id, tx);

    // Clone manager and user_id *once* for the spawned worker loop
    let ws_manager_clone = ws_manager.get_ref().clone();
    let user_id_clone = user_id.clone();
    let mut session_clone = session.clone();

    // 3. Optimized Single Task Execution Loop
    // Merging inbound and outbound handling prevents spawning 2 tasks per user, saving memory and CPU scheduling overhead.
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Outbound branch: Listen to internal MPSC channel and push text to client
                Some(msg) = rx.recv() => {
                    if session_clone.text(msg).await.is_err() {
                        break; // Client disconnected or connection fractured
                    }
                }

                // Inbound branch: Stream messages originating directly from the websocket client
                maybe_msg = msg_stream.next() => {
                    match maybe_msg {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(text) => {
                                    if let Err(e) = handle_client_command(&ws_manager_clone, &user_id_clone, &text).await {
                                        let _ = session_clone.text(format!("Error parsing command: {}", e)).await;
                                    }
                                }
                                Message::Ping(bytes) => {
                                    let _ = session_clone.pong(&bytes).await;
                                }
                                Message::Close(_) => break, // Clean close received from client
                                _ => {}
                            }
                        }
                        _ => break, // Stream broke or ended
                    }
                }
            }
        }

        // Guaranteed singular cleanup point. No double-drops or race conditions possible.
        cleanup_user(&ws_manager_clone, &user_id_clone);
    });

    http_ok("Ok!")
}