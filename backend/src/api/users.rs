use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub is_admin: Option<bool>,
    pub password: Option<String>,
}

/// List all users
async fn list_users(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Get from database
    let users = vec![
        User {
            id: "1".to_string(),
            username: "admin".to_string(),
            email: Some("admin@pinas.local".to_string()),
            is_admin: true,
            created_at: "2025-01-14T00:00:00Z".to_string(),
        },
    ];

    Json(users)
}

/// Create a new user
async fn create_user(
    State(_state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // TODO: Save to database with hashed password
    let user = User {
        id: uuid::Uuid::new_v4().to_string(),
        username: payload.username,
        email: payload.email,
        is_admin: payload.is_admin,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::CREATED, Json(user))
}

/// Get a specific user
async fn get_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Get from database
    let user = User {
        id,
        username: "admin".to_string(),
        email: Some("admin@pinas.local".to_string()),
        is_admin: true,
        created_at: "2025-01-14T00:00:00Z".to_string(),
    };

    Json(user)
}

/// Update a user
async fn update_user(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(_payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // TODO: Update in database
    let user = User {
        id,
        username: "admin".to_string(),
        email: Some("admin@pinas.local".to_string()),
        is_admin: true,
        created_at: "2025-01-14T00:00:00Z".to_string(),
    };

    Json(user)
}

/// Delete a user
async fn delete_user(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Delete from database
    StatusCode::NO_CONTENT
}
