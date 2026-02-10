use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::System;
use tokio::sync::broadcast;
use tokio::time::interval;

use crate::api::cookies;
use crate::services::auth::validate_jwt;
use crate::AppState;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    #[serde(rename = "system.stats")]
    SystemStats(SystemStats),
    #[serde(rename = "notification")]
    Notification(Notification),
    #[serde(rename = "task.progress")]
    TaskProgress(TaskProgressEvent),
    #[serde(rename = "file.task")]
    FileTask(FileTaskEvent),
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

/// File operation task event sent via WebSocket broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTaskEvent {
    pub task_id: String,
    pub task_type: String,    // "upload", "copy", "move", "delete", "create_folder", "create_file"
    pub file_name: String,
    pub status: String,       // "in_progress", "completed", "error"
    pub progress: i32,        // 0-100
    pub error_message: Option<String>,
}

/// WebSocket handler (requires valid token via cookie or query param)
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    // Try cookie first, then query param fallback
    let token = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(cookies::extract_token_from_cookies)
        .map(|s| s.to_string())
        .or_else(|| query.token.filter(|t| !t.is_empty()));

    let token = match token {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if validate_jwt(&token, &state.config).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let task_rx = state.task_tx.subscribe();
    let file_task_rx = state.file_task_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, task_rx, file_task_rx))
        .into_response()
}

/// Handle individual WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    mut task_rx: broadcast::Receiver<TaskProgressEvent>,
    mut file_task_rx: broadcast::Receiver<FileTaskEvent>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to send periodic system stats, task progress, and file task events
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
                result = file_task_rx.recv() => {
                    match result {
                        Ok(file_event) => {
                            let event = WsEvent::FileTask(file_event);
                            let msg = serde_json::to_string(&event).unwrap();
                            if sender.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("WebSocket file task receiver lagged by {} messages", n);
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
