use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthUser;
use crate::services::service::ServiceManager;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/switch", post(switch_service))
}

#[derive(Debug, Serialize)]
struct DisplayService {
    id: String,
    name: String,
    description: String,
    running: bool,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct DisplayStatus {
    active_service: Option<String>,
    available_services: Vec<DisplayService>,
}

#[derive(Debug, Deserialize)]
struct SwitchRequest {
    service: Option<String>,
}

async fn get_status(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    if state.config.dev_mode {
        return (StatusCode::OK, Json(DisplayStatus {
            active_service: Some("kodi".to_string()),
            available_services: vec![DisplayService {
                id: "kodi".to_string(),
                name: "Kodi".to_string(),
                description: "Media Center".to_string(),
                running: true,
                enabled: true,
            }],
        })).into_response();
    }

    let svc = ServiceManager::new();

    let kodi_status = svc.get_status("kodi").await;
    let (running, enabled) = match kodi_status {
        Ok(s) => (s.running, s.enabled),
        Err(_) => (false, false),
    };

    let active_service = if running { Some("kodi".to_string()) } else { None };

    let status = DisplayStatus {
        active_service,
        available_services: vec![DisplayService {
            id: "kodi".to_string(),
            name: "Kodi".to_string(),
            description: "Media Center".to_string(),
            running,
            enabled,
        }],
    };

    (StatusCode::OK, Json(status)).into_response()
}

async fn switch_service(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<SwitchRequest>,
) -> impl IntoResponse {
    if state.config.dev_mode {
        return (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "active_service": body.service,
        }))).into_response();
    }

    let svc = ServiceManager::new();

    match body.service.as_deref() {
        Some("kodi") => {
            // Stop splash, configure Kodi webserver, start + enable Kodi
            let _ = svc.stop("pinas-splash").await;
            let _ = svc.disable("pinas-splash").await;

            // Run kodi config script to ensure webserver is ready
            let _ = tokio::process::Command::new("/usr/bin/pinas-kodi-config.sh")
                .output()
                .await;

            if let Err(e) = svc.enable("kodi").await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to enable kodi: {}", e)
                }))).into_response();
            }
            if let Err(e) = svc.start("kodi").await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to start kodi: {}", e)
                }))).into_response();
            }

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "active_service": "kodi",
            }))).into_response()
        }
        None => {
            // Stop + disable Kodi, start splash
            if let Err(e) = svc.stop("kodi").await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to stop kodi: {}", e)
                }))).into_response();
            }
            let _ = svc.disable("kodi").await;

            let _ = svc.enable("pinas-splash").await;
            if let Err(e) = svc.start("pinas-splash").await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Failed to start pinas-splash: {}", e)
                }))).into_response();
            }

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "active_service": null,
            }))).into_response()
        }
        Some(unknown) => {
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": format!("Unknown service: {}", unknown)
            }))).into_response()
        }
    }
}
