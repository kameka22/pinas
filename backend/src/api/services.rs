use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::service::{ServiceManager, ServiceStatus, LogEntry};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_services))
        .route("/:name/status", get(get_service_status))
        .route("/:name/start", post(start_service))
        .route("/:name/stop", post(stop_service))
        .route("/:name/restart", post(restart_service))
        .route("/:name/enable", post(enable_service))
        .route("/:name/disable", post(disable_service))
        .route("/:name/logs", get(get_service_logs))
}

/// List all PiNAS-managed services
async fn list_services(State(_state): State<AppState>) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.list_services().await {
        Ok(services) => Json(services).into_response(),
        Err(e) => {
            tracing::error!("Failed to list services: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get status of a specific service
async fn get_service_status(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.get_status(&name).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => {
            tracing::error!("Failed to get service status for {}: {}", name, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// Start a service
async fn start_service(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.start(&name).await {
        Ok(()) => {
            tracing::info!("Service {} started", name);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to start service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Stop a service
async fn stop_service(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.stop(&name).await {
        Ok(()) => {
            tracing::info!("Service {} stopped", name);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to stop service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Restart a service
async fn restart_service(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.restart(&name).await {
        Ok(()) => {
            tracing::info!("Service {} restarted", name);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to restart service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Enable a service (auto-start on boot)
async fn enable_service(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.enable(&name).await {
        Ok(()) => {
            tracing::info!("Service {} enabled", name);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to enable service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Disable a service
async fn disable_service(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.disable(&name).await {
        Ok(()) => {
            tracing::info!("Service {} disabled", name);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to disable service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Query parameters for logs
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_lines")]
    lines: u32,
}

fn default_lines() -> u32 {
    100
}

/// Get logs for a service
async fn get_service_logs(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<LogsQuery>,
) -> impl IntoResponse {
    let manager = ServiceManager::new();

    match manager.get_logs(&name, params.lines).await {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => {
            tracing::error!("Failed to get logs for service {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
