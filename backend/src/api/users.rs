use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::{AdminUser, AuthErrorResponse, AuthUser};
use crate::services::user::{
    self, change_password, create_user as create_user_service, delete_user as delete_user_service,
    get_user_by_id, list_users as list_users_service, update_user as update_user_service,
    UserError, UserUpdate,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
        .route("/:id/password", put(change_user_password))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::models::user::User> for UserResponse {
    fn from(user: crate::models::user::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<UserError> for (StatusCode, Json<ErrorResponse>) {
    fn from(err: UserError) -> Self {
        let (status, code) = match &err {
            UserError::NotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            UserError::DuplicateUsername => (StatusCode::CONFLICT, "DUPLICATE_USERNAME"),
            UserError::CannotDeleteSelf => (StatusCode::FORBIDDEN, "CANNOT_DELETE_SELF"),
            UserError::CannotDeleteLastAdmin => (StatusCode::FORBIDDEN, "CANNOT_DELETE_LAST_ADMIN"),
            UserError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            UserError::AuthError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AUTH_ERROR"),
        };

        (
            status,
            Json(ErrorResponse {
                error: err.to_string(),
                code: code.to_string(),
            }),
        )
    }
}

/// List all users (admin only)
async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    match list_users_service(&state.db).await {
        Ok(users) => {
            let response: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Create a new user (admin only)
async fn create_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Validate input
    if payload.username.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Username is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 6 characters".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    match create_user_service(
        &state.db,
        &payload.username,
        &payload.password,
        payload.email,
        payload.is_admin,
    )
    .await
    {
        Ok(user) => {
            let response: UserResponse = user.into();
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Get a specific user (admin or self)
async fn get_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Check if user is accessing their own profile or is admin
    if user.id != id && !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        )
            .into_response();
    }

    match get_user_by_id(&state.db, &id).await {
        Ok(Some(db_user)) => {
            let response: UserResponse = db_user.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Update a user (admin for all fields, self for limited fields)
async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Check permissions
    let is_self = user.id == id;
    let is_admin = user.is_admin;

    if !is_self && !is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        )
            .into_response();
    }

    // Non-admin users can only update their own email
    let updates = if is_admin {
        UserUpdate {
            email: payload.email.map(Some),
            is_admin: payload.is_admin,
            ..Default::default()
        }
    } else {
        UserUpdate {
            email: payload.email.map(Some),
            ..Default::default()
        }
    };

    match update_user_service(&state.db, &id, updates).await {
        Ok(updated_user) => {
            let response: UserResponse = updated_user.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update user: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Delete a user (admin only)
async fn delete_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match delete_user_service(&state.db, &id, &admin.id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete user: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Change a user's password (admin only)
async fn change_user_password(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    if payload.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Password must be at least 6 characters".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    match change_password(&state.db, &id, &payload.password).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to change password: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}
