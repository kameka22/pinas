use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::System;
use tokio::sync::broadcast;
use tokio::time::interval;

use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    #[serde(rename = "system.stats")]
    SystemStats(SystemStats),
    #[serde(rename = "notification")]
    Notification(Notification),
    #[serde(rename = "task.progress")]
    TaskProgress(TaskProgressEvent),
}

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
}

#[derive(Debug, Serialize)]
pub struct Notification {
    pub id: String,
    pub level: String,
    pub message: String,
}

/// Task progress event sent via WebSocket broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub package_id: String,
    pub status: String,
    pub progress: i32,
    pub total_steps: i32,
    pub progress_percent: i32,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
}

/// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let task_rx = state.task_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, task_rx))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, mut task_rx: broadcast::Receiver<TaskProgressEvent>) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to send periodic system stats AND task progress events
    let send_task = tokio::spawn(async move {
        let mut stats_interval = interval(Duration::from_secs(2));
        let mut sys = System::new_all();

        loop {
            tokio::select! {
                _ = stats_interval.tick() => {
                    sys.refresh_all();

                    let cpu_usage = sys.global_cpu_info().cpu_usage();
                    let memory_total = sys.total_memory();
                    let memory_used = sys.used_memory();
                    let memory_usage = (memory_used as f32 / memory_total as f32) * 100.0;

                    let event = WsEvent::SystemStats(SystemStats {
                        cpu_usage,
                        memory_usage,
                        memory_used,
                        memory_total,
                    });

                    let msg = serde_json::to_string(&event).unwrap();
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                result = task_rx.recv() => {
                    match result {
                        Ok(progress) => {
                            let event = WsEvent::TaskProgress(progress);
                            let msg = serde_json::to_string(&event).unwrap();
                            if sender.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("WebSocket task receiver lagged by {} messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received WebSocket message: {}", text);
            }
            Ok(Message::Close(_)) => {
                tracing::debug!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Clean up
    send_task.abort();
}
