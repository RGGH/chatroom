// src/handlers.rs
use std::sync::{Arc, Mutex};
use futures::{StreamExt, SinkExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::reply;
use warp::ws::{Message, WebSocket};

use crate::chat::{ChatState, ChatMessage};

// Serve the HTML page
pub fn serve_html(html: &'static str) -> impl warp::Reply {
    reply::html(html)
}

// Handle WebSocket connections
pub async fn handle_websocket(ws: WebSocket, state: Arc<Mutex<ChatState>>) {
    // Split the socket
    let (mut ws_tx, mut ws_rx) = ws.split();
    
    // Create channel for this connection
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    
    // Register the connection and get ID
    let conn_id = {
        let mut state = state.lock().unwrap();
        let id = state.add_connection(tx.clone());
        
        // Send message history to new client
        state.send_history(&tx);
        
        id
    };
    
    // Task to send messages from channel to WebSocket
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            if let Err(e) = ws_tx.send(message.unwrap_or(Message::text("Error"))).await {
                eprintln!("WebSocket send error: {}", e);
                break;
            }
        }
    });
    
    // Process incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                // Handle text messages only
                if let Ok(text) = msg.to_str() {
                    // Try to parse as a ChatMessage
                    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                        // Add message to state (which broadcasts to all clients)
                        let mut state = state.lock().unwrap();
                        state.add_message(chat_msg);
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // Connection closed, remove from state
    let mut state = state.lock().unwrap();
    state.remove_connection(&conn_id);
}
