use crate::messages::TimestampedInavMessage;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{headers, Router, TypedHeader};
use std::sync::Arc;

pub(crate) struct AppState {
    broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
}

pub(crate) async fn run_webserver(
    broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
) {
    let app_state = Arc::new(AppState { broadcast_channel });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/test", get(test_handler))
        .with_state(app_state);

    let _ = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await;
}

async fn test_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "hoi")
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    state: State<Arc<AppState>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    println!("HOI");
    // println!("{:?}", state.test);
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    // println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let broadcast_channel = state.broadcast_channel.resubscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, broadcast_channel))
}

async fn handle_socket(
    mut socket: WebSocket,
    mut broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
) {
    while let Ok(message) = broadcast_channel.recv().await {
        if let Ok(text) = serde_json::to_string(&message) {
            let send_result = socket.send(Message::Text(text)).await;
        }
    }
}
