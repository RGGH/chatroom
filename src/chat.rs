// src/chat.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use warp::ws::Message;
use uuid::Uuid;

// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

// Connection ID type
pub type ConnectionId = String;

// Shared state
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub connections: HashMap<ConnectionId, mpsc::UnboundedSender<Result<Message, warp::Error>>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            connections: HashMap::new(),
        }
    }
    
    pub fn add_message(&mut self, message: ChatMessage) {
        // Add to history (limit to 100 messages)
        self.messages.push(message.clone());
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
        
        // Convert to JSON
        let message_json = serde_json::to_string(&message).unwrap_or_default();
        
        // Broadcast to all clients
        let connections = self.connections.clone();
        for tx in connections.values() {
            let _ = tx.send(Ok(Message::text(&message_json)));
        }
    }
    
    pub fn add_connection(&mut self, tx: mpsc::UnboundedSender<Result<Message, warp::Error>>) -> ConnectionId {
        // Generate unique ID
        let id = Uuid::new_v4().to_string();
        
        // Add to connections
        self.connections.insert(id.clone(), tx);
        
        id
    }
    
    pub fn remove_connection(&mut self, id: &ConnectionId) {
        self.connections.remove(id);
    }
    
    pub fn send_history(&self, tx: &mpsc::UnboundedSender<Result<Message, warp::Error>>) {
        for message in &self.messages {
            if let Ok(json) = serde_json::to_string(message) {
                let _ = tx.send(Ok(Message::text(json)));
            }
        }
    }
}
