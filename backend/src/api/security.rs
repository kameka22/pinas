use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::services::password_policy::PasswordPolicy;
use crate::services::security::{SecurityService, SecuritySettings};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_status).put(put_settings))
        .route("/login-attempts", get(login_attempts))
        .route("/tls/reset", post(reset_tls))
        .route("/restart-service", post(restart_service))
        .route("/password-policy", get(get_policy).put(put_policy))
}

async fn get_status(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(SecurityService::new(state.db.clone()).status(state.tls_enabled).await?))
}

async fn put_settings(State(state): State<AppState>, Json(settings): Json<SecuritySettings>) -> Result<impl IntoResponse, ApiError> {
    if !(1..=24 * 30).contains(&settings.session_hours) {
        return Err(ApiError::bad_request("session_hours must be between 1 and 720"));
    }
    if settings.tls_enabled && state.config.dev_mode {
        return Err(ApiError::bad_request("TLS cannot be enabled in dev mode"));
    }
    let svc = SecurityService::new(state.db.clone());
    svc.save(&settings).await?;
    Ok(Json(svc.status(state.tls_enabled).await?))
}

#[derive(Deserialize)]
struct AttemptsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    100
}

async fn login_attempts(State(state): State<AppState>, Query(q): Query<AttemptsQuery>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(SecurityService::new(state.db.clone()).login_attempts(q.limit.clamp(1, 1000)).await?))
}

/// Remove the self-signed certificate; a new one is generated at the next start
async fn reset_tls(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    SecurityService::new(state.db.clone()).reset_tls_certificate().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Restart the PiNAS service itself (applies TLS changes); the UI reconnects
async fn restart_service(State(state): State<AppState>) -> impl IntoResponse {
    if state.config.dev_mode {
        tracing::info!("[DEV MODE] Would restart pinas.service");
        return StatusCode::ACCEPTED;
    }
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let _ = tokio::process::Command::new("systemctl").args(["restart", "pinas.service"]).status().await;
    });
    StatusCode::ACCEPTED
}

async fn get_policy(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(PasswordPolicy::load(&state.db).await?))
}

async fn put_policy(
    State(state): State<AppState>,
    Json(policy): Json<PasswordPolicy>,
) -> Result<impl IntoResponse, ApiError> {
    if !(4..=128).contains(&policy.min_length) {
        return Err(ApiError::bad_request("min_length must be between 4 and 128"));
    }
    policy.save(&state.db).await?;
    Ok(Json(PasswordPolicy::load(&state.db).await?))
}
