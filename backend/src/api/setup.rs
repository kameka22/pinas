use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::services::auth::generate_jwt;
use crate::services::group::{add_member, get_group_by_name};
use crate::services::session::create_session;
use crate::services::user::{create_user, has_any_users};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_setup_status))
        .route("/complete", post(complete_setup))
}

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub is_complete: bool,
    pub needs_setup: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompleteSetupRequest {
    pub machine_name: String,
    pub admin_username: String,
    pub admin_password: String,
}

#[derive(Debug, Serialize)]
pub struct CompleteSetupResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// Get setup status
async fn get_setup_status(State(state): State<AppState>) -> impl IntoResponse {
    match has_any_users(&state.db).await {
        Ok(has_users) => {
            let response = SetupStatusResponse {
                is_complete: has_users,
                needs_setup: !has_users,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check setup status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check setup status".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Complete setup - creates the initial admin user
async fn complete_setup(
    State(state): State<AppState>,
    Json(payload): Json<CompleteSetupRequest>,
) -> impl IntoResponse {
    // Check if setup is already complete
    match has_any_users(&state.db).await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Setup has already been completed".to_string(),
                    code: "SETUP_ALREADY_COMPLETE".to_string(),
                }),
            )
                .into_response();
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!("Failed to check setup status: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check setup status".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    }

    // Validate input
    if payload.admin_username.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.admin_username.len() < 3 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username must be at least 3 characters".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.admin_password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 6 characters".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.machine_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Machine name is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    // Create the admin user
    let user = match create_user(
        &state.db,
        &payload.admin_username,
        &payload.admin_password,
        Some(format!("{}@pinas.local", payload.admin_username)),
        true,
    )
    .await
    {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to create admin user: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create admin user".to_string(),
                    code: "USER_CREATION_FAILED".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Add admin user to administrators group
    if let Ok(Some(admin_group)) = get_group_by_name(&state.db, "administrators").await {
        if let Err(e) = add_member(&state.db, &admin_group.id, &user.id).await {
            tracing::warn!("Failed to add admin to administrators group: {}", e);
            // Continue anyway - user is still admin
        }
    }

    // TODO: Save machine_name to settings table (for future use)
    // For now, we just log it
    tracing::info!("Setup complete. Machine name: {}", payload.machine_name);

    // Generate JWT token for auto-login
    let token = match generate_jwt(&user, &state.config) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to generate token: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to generate authentication token".to_string(),
                    code: "TOKEN_GENERATION_FAILED".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Create session
    let expires_at = Utc::now() + Duration::hours(state.config.jwt_expiration_hours as i64);
    if let Err(e) = create_session(&state.db, &user.id, &token, expires_at).await {
        tracing::error!("Failed to create session: {}", e);
        // Continue anyway - token is still valid
    }

    let response = CompleteSetupResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}
