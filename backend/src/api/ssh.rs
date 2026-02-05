use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::ssh::SshService;
use crate::AppState;

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiSuccess {
    success: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/enable", post(enable_ssh))
        .route("/disable", post(disable_ssh))
        .route("/password", post(change_password))
}

/// Get SSH status
async fn get_status(State(_state): State<AppState>) -> impl IntoResponse {
    let service = SshService::new();

    match service.get_status().await {
        Ok(status) => Json(status).into_response(),
        Err(e) => {
            tracing::error!("Failed to get SSH status: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { message: e.to_string() })).into_response()
        }
    }
}

/// Enable SSH
async fn enable_ssh(State(_state): State<AppState>) -> impl IntoResponse {
    let service = SshService::new();

    match service.enable().await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to enable SSH: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { message: e.to_string() })).into_response()
        }
    }
}

/// Disable SSH
async fn disable_ssh(State(_state): State<AppState>) -> impl IntoResponse {
    let service = SshService::new();

    match service.disable().await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to disable SSH: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { message: e.to_string() })).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
}

/// Change SSH password
async fn change_password(
    State(_state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let service = SshService::new();

    match service.change_password(&payload.password).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to change SSH password: {}", e);
            (StatusCode::BAD_REQUEST, Json(ApiError { message: e.to_string() })).into_response()
        }
    }
}
