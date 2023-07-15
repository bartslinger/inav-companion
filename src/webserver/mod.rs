use crate::messages::{IncomingWebsocketMessage, SetRawRcMessage, TimestampedInavMessage};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{headers, Router, TypedHeader};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;

pub(crate) struct AppState {
    broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
    raw_rc_channel_tx: tokio::sync::mpsc::Sender<SetRawRcMessage>,
}

pub(crate) async fn run_webserver(
    broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
    raw_rc_channel_tx: tokio::sync::mpsc::Sender<SetRawRcMessage>,
) {
    let app_state = Arc::new(AppState {
        broadcast_channel,
        raw_rc_channel_tx,
    });

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
    let raw_rc_channel_tx = state.raw_rc_channel_tx.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, broadcast_channel, raw_rc_channel_tx))
}

async fn handle_socket(
    socket: WebSocket,
    mut broadcast_channel: tokio::sync::broadcast::Receiver<TimestampedInavMessage>,
    mut raw_rc_channel_tx: tokio::sync::mpsc::Sender<SetRawRcMessage>,
) {
    let (mut socket_sender, mut socket_receiver) = socket.split();

    let send_task = async move {
        while let Ok(message) = broadcast_channel.recv().await {
            if let Ok(text) = serde_json::to_string(&message) {
                let send_result = socket_sender.send(Message::Text(text)).await;
            }
        }
    };

    let receive_task = async move {
        while let Some(Ok(incoming)) = socket_receiver.next().await {
            match incoming {
                Message::Text(content) => {
                    let message =
                        serde_json::from_str::<IncomingWebsocketMessage>(content.as_str());
                    if let Ok(IncomingWebsocketMessage::SetRawRc(value)) = message {
                        let send_result = raw_rc_channel_tx.try_send(value);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}
