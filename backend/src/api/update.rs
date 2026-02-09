use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::services::update::UpdateService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/check", get(check_for_update))
        .route("/install", post(install_update))
        .route("/status", get(get_update_status))
        .route("/history", get(get_update_history))
        .route("/just-updated", get(get_just_updated))
        .route("/dismiss", post(dismiss_update))
}

/// Check if a new update is available
async fn check_for_update(State(state): State<AppState>) -> impl IntoResponse {
    let service = UpdateService::new(state.db.clone(), state.task_tx.clone());

    match service.check_for_update().await {
        Ok(result) => Json(result).into_response(),
        Err(e) => {
            tracing::error!("Failed to check for updates: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Start installing an update
async fn install_update(State(state): State<AppState>) -> impl IntoResponse {
    let service = UpdateService::new(state.db.clone(), state.task_tx.clone());

    // First check what version is available
    let check = match service.check_for_update().await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    if !check.available {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No update available" })),
        )
            .into_response();
    }

    // Start the update
    let task_id = match service.install_update_start(&check.latest_version).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Spawn background execution
    let db = state.db.clone();
    let task_tx = state.task_tx.clone();
    let bg_task_id = task_id.clone();
    tokio::spawn(async move {
        let service = UpdateService::new(db, task_tx);
        service.install_update_execute(&bg_task_id).await;
    });

    Json(serde_json::json!({ "task_id": task_id })).into_response()
}

/// Get status of a running update
async fn get_update_status(State(state): State<AppState>) -> impl IntoResponse {
    let service = UpdateService::new(state.db.clone(), state.task_tx.clone());

    // Get the most recent update entry
    match service.get_update_history().await {
        Ok(entries) => {
            if let Some(entry) = entries.first() {
                Json(entry).into_response()
            } else {
                Json(serde_json::json!({ "status": "none" })).into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to get update status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Get update history
async fn get_update_history(State(state): State<AppState>) -> impl IntoResponse {
    let service = UpdateService::new(state.db.clone(), state.task_tx.clone());

    match service.get_update_history().await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            tracing::error!("Failed to get update history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

/// Response for just-updated endpoint
#[derive(Serialize)]
struct JustUpdatedResponse {
    just_updated: bool,
    version: Option<String>,
    previous_version: Option<String>,
    changelog: Option<std::collections::HashMap<String, String>>,
}

/// Check if the system was just updated (after service restart)
async fn get_just_updated(State(state): State<AppState>) -> impl IntoResponse {
    let info = state.just_updated.lock().await;

    if let Some(ref update_info) = *info {
        Json(JustUpdatedResponse {
            just_updated: true,
            version: Some(update_info.version.clone()),
            previous_version: Some(update_info.previous_version.clone()),
            changelog: Some(update_info.changelog.clone()),
        })
        .into_response()
    } else {
        Json(JustUpdatedResponse {
            just_updated: false,
            version: None,
            previous_version: None,
            changelog: None,
        })
        .into_response()
    }
}

/// Dismiss the update notification (removes the flag)
async fn dismiss_update(State(state): State<AppState>) -> impl IntoResponse {
    // Clear the in-memory state
    {
        let mut info = state.just_updated.lock().await;
        *info = None;
    }

    // Remove the flag file
    let data_dir = std::env::var("PINAS_DATA_DIR")
        .unwrap_or_else(|_| "/storage/.pinas".to_string());

    match UpdateService::dismiss_update_applied(&data_dir).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to dismiss update: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
