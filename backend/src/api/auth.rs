use axum::{
    extract::State,
    http::{header::AUTHORIZATION, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::api::middleware::{AuthErrorResponse, AuthUser};
use crate::services::auth::{extract_bearer_token, generate_jwt, verify_password, AuthError};
use crate::services::session::{create_session, delete_session};
use crate::services::user::{change_password as change_user_password, get_user_by_id, get_user_by_username};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/change-password", post(change_password))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Login endpoint
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Get user by username
    let user = match get_user_by_username(&state.db, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "Invalid credentials".to_string(),
                    code: "INVALID_CREDENTIALS".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Verify password
    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "Invalid credentials".to_string(),
                    code: "INVALID_CREDENTIALS".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    }

    // Generate JWT token
    let token = match generate_jwt(&user, &state.config) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Token generation error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Store session in database
    let expires_at = Utc::now() + Duration::hours(state.config.jwt_expiration_hours as i64);
    if let Err(e) = create_session(&state.db, &user.id, &token, expires_at).await {
        tracing::error!("Session creation error: {}", e);
        // Continue anyway - JWT is still valid
    }

    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Logout endpoint
async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract token from header
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if let Some(token) = extract_bearer_token(header_str) {
                // Delete session from database
                if let Err(e) = delete_session(&state.db, token).await {
                    tracing::error!("Session deletion error: {}", e);
                }
            }
        }
    }

    StatusCode::OK
}

/// Get current user info
async fn me(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    // Get full user info from database
    match get_user_by_id(&state.db, &user.id).await {
        Ok(Some(db_user)) => {
            let user_info = UserInfo {
                id: db_user.id,
                username: db_user.username,
                email: db_user.email,
                is_admin: db_user.is_admin,
            };
            (StatusCode::OK, Json(user_info)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(AuthErrorResponse {
                error: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Change password endpoint
async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Verify current password
    let db_user = match get_user_by_id(&state.db, &user.id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(AuthErrorResponse {
                    error: "User not found".to_string(),
                    code: "USER_NOT_FOUND".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Verify current password
    match verify_password(&payload.current_password, &db_user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "Current password is incorrect".to_string(),
                    code: "INVALID_PASSWORD".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Password verification error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "Internal server error".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    }

    // Change password
    if let Err(e) = change_user_password(&state.db, &user.id, &payload.new_password).await {
        tracing::error!("Password change error: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthErrorResponse {
                error: "Failed to change password".to_string(),
                code: "PASSWORD_CHANGE_FAILED".to_string(),
            }),
        )
            .into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
