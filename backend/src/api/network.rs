use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Serialize;

use crate::api::middleware::AdminUser;
use crate::services::network::{DnsConfig, NetworkService, UpdateHostnameRequest, UpdateInterfaceRequest};
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
        .route("/interface", put(update_interface))
        .route("/dns", put(update_dns))
        .route("/hostname", put(update_hostname))
}

/// Get network status (interfaces, DNS, gateway, hostname)
async fn get_status(State(_state): State<AppState>) -> impl IntoResponse {
    let service = NetworkService::new();

    match service.get_status().await {
        Ok(status) => Json(status).into_response(),
        Err(e) => {
            tracing::error!("Failed to get network status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Update network interface configuration (admin only, DHCP/static)
async fn update_interface(
    State(_state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<UpdateInterfaceRequest>,
) -> impl IntoResponse {
    let service = NetworkService::new();

    match service.update_interface(&payload).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to update interface: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Update DNS configuration (admin only)
async fn update_dns(
    State(_state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<DnsConfig>,
) -> impl IntoResponse {
    let service = NetworkService::new();

    match service.update_dns(&payload).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to update DNS: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Update hostname (admin only)
async fn update_hostname(
    State(_state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<UpdateHostnameRequest>,
) -> impl IntoResponse {
    let service = NetworkService::new();

    match service.update_hostname(&payload.hostname).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to update hostname: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}
