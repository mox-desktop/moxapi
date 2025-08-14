mod idle;
mod notify;

use actix_web::web;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::connect_async;
use url::Url;

struct State {
    idle: Arc<RwLock<idle::Idle>>,
    notify: notify::NotificationManager,
}

#[derive(Serialize)]
struct Status {
    active: bool,
    active_time: u32,
    inhibited: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    GetStatus,
    IdleInhibit,
    IdleUninhibit,
    IdleLock,
    IdleUnlock,
    SimulateUserActivity,
    Notify {
        summary: String,
        body: String,
        timeout: i32,
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ServerMessage {
    Status {
        active: bool,
        active_time: u32,
        inhibited: bool,
    },
    Ok,
    Error {
        message: String,
    },
}

#[tokio::main]
async fn main() {
    let state = web::Data::new(State {
        idle: Arc::new(RwLock::new(idle::Idle::new().await.unwrap())),
        notify: notify::NotificationManager::new().await.unwrap(),
    });

    let url = Url::parse("ws://localhost:8080/echo").unwrap();
    let (ws_stream, _) = connect_async(url.as_str())
        .await
        .expect("Failed to connect");
    println!("WebSocket connected");

    let (_write, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
            match serde_json::from_str::<ClientMessage>(&txt) {
                Ok(ClientMessage::GetStatus) => {
                    let idle = state.idle.read().await;
                    let status = Status {
                        active: idle.get_active().await.unwrap(),
                        active_time: idle.get_active_time().await.unwrap(),
                        inhibited: idle.get_inhibited(),
                    };
                    //println!("Status: {:?}", status);
                }
                Ok(ClientMessage::IdleInhibit) => {
                    state.idle.write().await.inhibit().await.unwrap();
                }
                Ok(ClientMessage::IdleUninhibit) => {
                    state.idle.write().await.uninhibit().await.unwrap();
                }
                Ok(ClientMessage::IdleLock) => {
                    state.idle.read().await.lock().await.unwrap();
                }
                Ok(ClientMessage::IdleUnlock) => {
                    state.idle.read().await.unlock().await.unwrap();
                }
                Ok(ClientMessage::SimulateUserActivity) => {
                    state
                        .idle
                        .read()
                        .await
                        .simulate_user_activity()
                        .await
                        .unwrap();
                }
                Ok(ClientMessage::Notify {
                    summary,
                    body,
                    timeout,
                    id,
                }) => {
                    let builder = state.notify.builder().await;
                    builder
                        .with_summary(&summary)
                        .with_body(&body)
                        .with_timeout(timeout)
                        .with_id(id)
                        .send()
                        .await
                        .unwrap();
                }
                Err(e) => eprintln!("Invalid message: {e}"),
            }
        }
    }
}
