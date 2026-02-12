use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthUser;
use crate::models::permission::{FolderPermissions, PermissionEntry, PermissionLevel};
use crate::services::permission::PermissionService;
use crate::AppState;

/// Create the permissions router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_permissions))
        .route("/", post(create_permission))
        .route("/folders", get(list_folders))
        .route("/folder", get(get_folder_permissions))
        .route("/user/:user_id", get(get_user_permissions))
        .route("/:id", put(update_permission))
        .route("/:id", delete(delete_permission))
}

/// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Request to create a permission
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub path: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub permission: String,
}

/// Request to update a permission
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    pub permission: String,
}

/// Query for folder permissions
#[derive(Debug, Deserialize)]
pub struct FolderQuery {
    pub path: String,
}

/// Permission response
#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: String,
    pub path: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub permission: String,
}

/// List all permissions (admin only)
async fn list_permissions(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    // Only admins can view all permissions
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let service = PermissionService::new(state.db.clone());

    match service.list_all_grouped().await {
        Ok(permissions) => Json(permissions).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// List folders with permissions configured
async fn list_folders(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let service = PermissionService::new(state.db.clone());

    match service.list_configured_folders().await {
        Ok(folders) => Json(folders).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Get permissions for a specific folder
async fn get_folder_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let service = PermissionService::new(state.db.clone());

    match service.list_by_folder(&query.path).await {
        Ok(permissions) => Json(FolderPermissions {
            path: query.path,
            permissions,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Get permissions for a specific user
async fn get_user_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(target_user_id): Path<String>,
) -> impl IntoResponse {
    // Users can view their own permissions, admins can view anyone's
    if !user.is_admin && user.id != target_user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
            }),
        )
            .into_response();
    }

    let service = PermissionService::new(state.db.clone());

    match service.list_by_user(&target_user_id).await {
        Ok(permissions) => {
            let response: Vec<PermissionResponse> = permissions
                .into_iter()
                .map(|p| PermissionResponse {
                    id: p.id,
                    path: p.path,
                    user_id: p.user_id,
                    group_id: p.group_id,
                    permission: p.permission,
                })
                .collect();
            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Create a new permission
async fn create_permission(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreatePermissionRequest>,
) -> impl IntoResponse {
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let permission_level = match PermissionLevel::from_str(&payload.permission) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid permission level. Use 'none', 'read', or 'write'".to_string(),
                }),
            )
                .into_response();
        }
    };

    let service = PermissionService::new(state.db.clone());

    match service
        .upsert(
            &payload.path,
            payload.user_id.as_deref(),
            payload.group_id.as_deref(),
            permission_level,
        )
        .await
    {
        Ok(perm) => (
            StatusCode::CREATED,
            Json(PermissionResponse {
                id: perm.id,
                path: perm.path,
                user_id: perm.user_id,
                group_id: perm.group_id,
                permission: perm.permission,
            }),
        )
            .into_response(),
        Err(e) => {
            let status = match &e {
                crate::services::permission::PermissionError::InvalidPermission => {
                    StatusCode::BAD_REQUEST
                }
                crate::services::permission::PermissionError::AlreadyExists => {
                    StatusCode::CONFLICT
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse { error: e.to_string() })).into_response()
        }
    }
}

/// Update a permission
async fn update_permission(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePermissionRequest>,
) -> impl IntoResponse {
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let permission_level = match PermissionLevel::from_str(&payload.permission) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid permission level. Use 'none', 'read', or 'write'".to_string(),
                }),
            )
                .into_response();
        }
    };

    let service = PermissionService::new(state.db.clone());

    match service.update(&id, permission_level).await {
        Ok(perm) => Json(PermissionResponse {
            id: perm.id,
            path: perm.path,
            user_id: perm.user_id,
            group_id: perm.group_id,
            permission: perm.permission,
        })
        .into_response(),
        Err(e) => {
            let status = match &e {
                crate::services::permission::PermissionError::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse { error: e.to_string() })).into_response()
        }
    }
}

/// Delete a permission
async fn delete_permission(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    let service = PermissionService::new(state.db.clone());

    match service.delete(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = match &e {
                crate::services::permission::PermissionError::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse { error: e.to_string() })).into_response()
        }
    }
}
