
use fr_rust::prelude::*;
use actix_web::{get, web};
use actix_web::web::Path; 

use ::actix_ws::{Message, Session, MessageStream}; 

use futures_util::StreamExt; 

use tokio::sync::mpsc; 
use tokio::time::Instant; 

use serde::{Deserialize, Serialize};

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

async fn handle_client_command(
    ws_manager: &WsManager,
    user_id: &str,
    text: &str,
) -> Result<(), serde_json::Error> {
    let command: ClientCommand = serde_json::from_str(text)?;

    match command {
        ClientCommand::Join { room } => {
            ws_manager.join_room(&user_id, &room);
        }
        ClientCommand::Leave { room } => {
            ws_manager.leave_room(&user_id, &room);
        }
        ClientCommand::DropRoom { room } => {
            // Drop room requirement: Print all messages of the room before dropping it
            let messages = ws_manager.get_room_msgs(&room).await.expect("Error fetching messages");
            println!("--- History for Room [{}] before drop ---", room);
            for msg in messages {
                println!("{:?}", msg);
            }
            println!("-----------------------------------------");
            
            ws_manager.drop_room(&room);
        }
        ClientCommand::MsgUser { target_id, msg } => {
            ws_manager.msg_user(&target_id, msg);
        }
        ClientCommand::MsgRoom { room, msg } => {
            let user_msg = UserMsg::new(user_id.to_string(), room.clone(), msg);
            ws_manager.msg_room(&room, user_msg);
        }
        ClientCommand::Broadcast { msg } => {
            ws_manager.broadcast(msg);
        }
    }
    Ok(())
}

/// Centralized cleanup function when a user drops connection
fn cleanup_user(ws_manager: &WsManager, user_id: &str) {
    println!("Cleaning up connection for user: {}", user_id);
    // Explicitly drops the user and cleans up allocations from internal states/maps
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

    // 1. Setup the high-performance bounded channel
    let (tx, mut rx) = mpsc::channel::<String>(128);

    // Perform the WebSocket handshake
    let (res, session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(tuple) => tuple,
        Err(e) => return http_bad("Internal Server Error!"), 
    };

    // 2. Register User with the manager
    ws_manager.register(&user_id.clone(), tx);

    // Clone manager, user_id, and session for the tasks
    let ws_manager_outbound = ws_manager.clone();
    let user_id_outbound = user_id.clone();
    let mut session_outbound = session.clone();

    let ws_manager_inbound = ws_manager.clone();
    let user_id_inbound = user_id.clone();
    let mut session_inbound = session.clone(); // Cloned so 'session' ownership isn't lost

    // Task 1: Outbound Loop (Listen to the mpsc channel and push to the WS client)
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session_outbound.text(msg).await.is_err() {
                break; // Client disconnected, break loop to trigger cleanup
            }
        }
        // Cleanup if the outbound channel cuts off early
        cleanup_user(&ws_manager_outbound, &user_id_outbound);
    });

    // Task 2: Inbound Loop (Read from WS client and route commands)
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    if let Err(e) = handle_client_command(&ws_manager_inbound, &user_id_inbound, &text).await {
                        let _ = session_inbound.text(format!("Error parsing command: {}", e)).await;
                    }
                }
                Message::Ping(bytes) => {
                    let _ = session_inbound.pong(&bytes).await;
                }
                Message::Close(_) => break, // Exit loop on close frame
                _ => {}
            }
        }

        // 3. User Disconnected - Run meticulous cleanup
        cleanup_user(&ws_manager_inbound, &user_id_inbound);
    });

    http_ok("Ok!")
}
