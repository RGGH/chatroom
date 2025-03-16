// src/main.rs
mod chat;
mod handlers;

use std::sync::{Arc, Mutex};
use warp::Filter;

use crate::chat::ChatState;
use crate::handlers::{handle_websocket, serve_html};

// Use include_str! to include the HTML at compile time, making it 'static
const HTML: &'static str = include_str!("html/chat.html");

#[tokio::main]
async fn main() {
    // Create shared state
    let chat_state = Arc::new(Mutex::new(ChatState::new()));

    // Routes
    let html_route = warp::path::end()
        .map(|| serve_html(HTML));
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || chat_state.clone()))
        .map(|ws: warp::ws::Ws, state| {
            ws.on_upgrade(move |socket| handle_websocket(socket, state))
        });
    
    let routes = html_route.or(ws_route);
    
    println!("Chat server started at http://127.0.0.1:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
