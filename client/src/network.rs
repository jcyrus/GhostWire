// GhostWire Client - Network Layer
// This module handles WebSocket communication in a separate async task

use crate::app::{MessageMeta, MessageType, WireMessage};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Successfully connected to server
    Connected,
    
    /// Disconnected from server
    Disconnected,
    
    /// Received a chat message
    Message {
        sender: String,
        content: String,
        timestamp: i64,
        channel_id: String,
    },
    
    /// User joined
    UserJoined { username: String },
    
    /// User left
    UserLeft { username: String },
    
    /// System message
    SystemMessage { content: String },
    
    /// Error occurred
    Error { message: String },
}

/// Messages sent from the UI to the network task
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    /// Send a chat message to a specific channel
    SendMessage { content: String, channel_id: String },
    
    /// Authenticate with username (for reconnection scenarios)
    #[allow(dead_code)]
    Authenticate { username: String },
    
    /// Disconnect from server
    Disconnect,
}

/// Network task that runs in a separate tokio runtime
/// This is the CRITICAL async/sync split - this task is async, UI is sync
pub async fn network_task(
    server_url: String,
    username: String,
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    mut command_rx: mpsc::UnboundedReceiver<NetworkCommand>,
) {
    // Attempt to connect to the server
    let ws_stream = match connect_async(&server_url).await {
        Ok((stream, _)) => {
            let _ = event_tx.send(NetworkEvent::Connected);
            stream
        }
        Err(e) => {
            let _ = event_tx.send(NetworkEvent::Error {
                message: format!("Failed to connect: {}", e),
            });
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Send authentication message
    let auth_msg = WireMessage {
        msg_type: MessageType::Auth,
        payload: username.clone(),
        channel: "global".to_string(),
        meta: MessageMeta {
            sender: username.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        },
    };

    if let Ok(json) = serde_json::to_string(&auth_msg) {
        if let Err(e) = write.send(Message::Text(json)).await {
            let _ = event_tx.send(NetworkEvent::Error {
                message: format!("Failed to authenticate: {}", e),
            });
            return;
        }
    }

    // Main network loop
    loop {
        tokio::select! {
            // Handle incoming messages from server
            Some(msg_result) = read.next() => {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        // Parse the wire message
                        if let Ok(wire_msg) = serde_json::from_str::<WireMessage>(&text) {
                            handle_wire_message(wire_msg, &event_tx);
                        } else {
                            let _ = event_tx.send(NetworkEvent::Error {
                                message: "Failed to parse message".to_string(),
                            });
                        }
                    }
                    Ok(Message::Close(_)) => {
                        let _ = event_tx.send(NetworkEvent::Disconnected);
                        break;
                    }
                    Err(e) => {
                        let _ = event_tx.send(NetworkEvent::Error {
                            message: format!("WebSocket error: {}", e),
                        });
                        break;
                    }
                    _ => {}
                }
            }

            // Handle commands from UI
            Some(command) = command_rx.recv() => {
                match command {
                    NetworkCommand::SendMessage { content, channel_id } => {
                        let msg = WireMessage {
                            msg_type: MessageType::Message,
                            payload: content,
                            channel: channel_id,
                            meta: MessageMeta {
                                sender: username.clone(),
                                timestamp: chrono::Utc::now().timestamp(),
                            },
                        };

                        if let Ok(json) = serde_json::to_string(&msg) {
                            // Use if let to handle errors gracefully (no .unwrap())
                            if let Err(e) = write.send(Message::Text(json)).await {
                                let _ = event_tx.send(NetworkEvent::Error {
                                    message: format!("Failed to send message: {}", e),
                                });
                            }
                        }
                    }
                    NetworkCommand::Authenticate { username: new_username } => {
                        let msg = WireMessage {
                            msg_type: MessageType::Auth,
                            payload: new_username.clone(),
                            channel: "global".to_string(),
                            meta: MessageMeta {
                                sender: new_username,
                                timestamp: chrono::Utc::now().timestamp(),
                            },
                        };

                        if let Ok(json) = serde_json::to_string(&msg) {
                            if let Err(e) = write.send(Message::Text(json)).await {
                                let _ = event_tx.send(NetworkEvent::Error {
                                    message: format!("Failed to authenticate: {}", e),
                                });
                            }
                        }
                    }
                    NetworkCommand::Disconnect => {
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                }
            }

            // If both channels are closed, exit
            else => break,
        }
    }

    let _ = event_tx.send(NetworkEvent::Disconnected);
}

/// Handle a wire message and convert it to a NetworkEvent
fn handle_wire_message(
    msg: WireMessage,
    event_tx: &mpsc::UnboundedSender<NetworkEvent>,
) {
    match msg.msg_type {
        MessageType::Message => {
            let _ = event_tx.send(NetworkEvent::Message {
                sender: msg.meta.sender,
                content: msg.payload,
                timestamp: msg.meta.timestamp,
                channel_id: msg.channel,
            });
        }
        MessageType::System => {
            // Parse system messages for user join/leave
            if msg.payload.contains("joined") {
                let _ = event_tx.send(NetworkEvent::UserJoined {
                    username: msg.meta.sender,
                });
            } else if msg.payload.contains("left") {
                let _ = event_tx.send(NetworkEvent::UserLeft {
                    username: msg.meta.sender,
                });
            } else {
                let _ = event_tx.send(NetworkEvent::SystemMessage {
                    content: msg.payload,
                });
            }
        }
        MessageType::Auth => {
            // User authenticated - add them to roster
            let username = msg.meta.sender.clone();
            let _ = event_tx.send(NetworkEvent::UserJoined { username });
        }
    }
}
