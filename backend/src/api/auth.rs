use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
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
    pub is_admin: bool,
}

/// Login endpoint
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // TODO: Implement actual authentication
    // For now, return a mock response

    if payload.username == "admin" && payload.password == "admin" {
        let response = LoginResponse {
            token: "mock-jwt-token".to_string(),
            user: UserInfo {
                id: "1".to_string(),
                username: payload.username,
                is_admin: true,
            },
        };
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }
}

/// Logout endpoint
async fn logout() -> impl IntoResponse {
    StatusCode::OK
}

/// Get current user info
async fn me() -> impl IntoResponse {
    // TODO: Extract user from JWT token
    let user = UserInfo {
        id: "1".to_string(),
        username: "admin".to_string(),
        is_admin: true,
    };
    Json(user)
}
