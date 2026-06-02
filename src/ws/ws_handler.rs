use fr_rust::prelude::{
  *, actix_ws::{Message, Session, Stream},
  futures_util::StreamExt,
  tokio::sync::mpsc,
  tokio::time::instant
};

use std::time::Duration;


#[derive(Deserialize)]
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
            ws_manager.join_room(user_id.to_string(), room);
        }
        ClientCommand::Leave { room } => {
            ws_manager.leave_room(user_id.to_string(), room);
        }
        ClientCommand::DropRoom { room } => {
            // Drop room requirement: Print all messages of the room before dropping it
            let messages = ws_manager.get_room_msgs(room.clone());
            println!("--- History for Room [{}] before drop ---", room);
            for msg in messages {
                println!("{:?}", msg);
            }
            println!("-----------------------------------------");
            
            ws_manager.drop_room(room);
        }
        ClientCommand::MsgUser { target_id, msg } => {
            ws_manager.msg_user(target_id, msg);
        }
        ClientCommand::MsgRoom { room, msg } => {
            let user_msg = UserMsg::new(user_id, &room, &msg);
            ws_manager.msg_room(room, user_msg);
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
    ws_manager.drop_user(user_id.to_string());
}

#[get("/ws/{user_id}")]
async fn ws_handler(
    req: Rqs,
    body: Payload,
    ws_manager: AppData<WsManager>,
    path: Path<String>,
) -> Rsp {
    let user_id = path.into_inner();

    // 1. Setup the high-performance bounded channel
    let (tx, mut rx) = mpsc::channel::<String>(128);

    // Perform the WebSocket handshake
    let (session, mut msg_stream) = match actix_ws::handle(&req, body) {
        Ok(res) => res,
        Err(e) => return ErrorResponse::internal_server_error(e.to_string()), // Adjust to fr_rust error handling
    };

    // 2. Register User with the manager
    ws_manager.register(user_id.clone(), tx);

    // Clone manager and user_id for the background write loop task
    let ws_manager_clone = ws_manager.clone();
    let user_id_clone = user_id.clone();
    let mut session_clone = session.clone();

    // Task 1: Outbound Loop (Listen to the mpsc channel and push to the WS client)
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session_clone.text(msg).await.is_err() {
                break; // Client disconnected, break loop to trigger cleanup
            }
        }
        // Cleanup if the outbound channel cuts off early
        cleanup_user(&ws_manager_clone, &user_id_clone);
    });

    // Task 2: Inbound Loop (Read from WS client and route commands)
    tokio::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Let's parse the text. Assuming a simple JSON string or command format.
                    // For flexibility, let's assume incoming payloads look like a command:
                    // e.g., {"cmd": "join", "room": "lobby"} or {"cmd": "msg_room", "room": "lobby", "text": "hi"}
                    if let Err(e) = handle_client_command(&ws_manager, &user_id, &text).await {
                        let _ = session.text(format!("Error parsing command: {}", e)).await;
                    }
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Close(_) => break, // Exit loop on close frame
                _ => {}
            }
        }

        // 3. User Disconnected - Run meticulous cleanup
        cleanup_user(&ws_manager, &user_id);
    });

    http_ok("Ok!")
}