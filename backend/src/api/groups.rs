use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AdminUser;
use crate::services::group::{
    self, add_member, count_group_members, create_group as create_group_service,
    delete_group as delete_group_service, get_group_by_id, get_group_members,
    list_groups as list_groups_service, remove_member, update_group as update_group_service,
    GroupError,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_groups))
        .route("/", post(create_group))
        .route("/:id", get(get_group))
        .route("/:id", put(update_group))
        .route("/:id", delete(delete_group))
        .route("/:id/members", get(list_group_members))
        .route("/:id/members", post(add_group_member))
        .route("/:id/members/:user_id", delete(remove_group_member))
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub member_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<GroupError> for (StatusCode, Json<ErrorResponse>) {
    fn from(err: GroupError) -> Self {
        let (status, code) = match &err {
            GroupError::NotFound => (StatusCode::NOT_FOUND, "GROUP_NOT_FOUND"),
            GroupError::DuplicateName => (StatusCode::CONFLICT, "DUPLICATE_NAME"),
            GroupError::CannotDeleteSystemGroup => {
                (StatusCode::FORBIDDEN, "CANNOT_DELETE_SYSTEM_GROUP")
            }
            GroupError::NotAMember => (StatusCode::NOT_FOUND, "NOT_A_MEMBER"),
            GroupError::AlreadyMember => (StatusCode::CONFLICT, "ALREADY_MEMBER"),
            GroupError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
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

/// List all groups (admin only)
async fn list_groups(State(state): State<AppState>, _admin: AdminUser) -> impl IntoResponse {
    match list_groups_service(&state.db).await {
        Ok(groups) => {
            let mut responses = Vec::new();
            for group in groups {
                let member_count = count_group_members(&state.db, &group.id)
                    .await
                    .unwrap_or(0);
                responses.push(GroupResponse {
                    id: group.id,
                    name: group.name,
                    description: group.description,
                    is_system: group.is_system,
                    member_count,
                    created_at: group.created_at,
                    updated_at: group.updated_at,
                });
            }
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list groups: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Create a new group (admin only)
async fn create_group(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Group name is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    match create_group_service(&state.db, &payload.name, payload.description).await {
        Ok(group) => {
            let response = GroupResponse {
                id: group.id,
                name: group.name,
                description: group.description,
                is_system: group.is_system,
                member_count: 0,
                created_at: group.created_at,
                updated_at: group.updated_at,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create group: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Get a specific group (admin only)
async fn get_group(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match get_group_by_id(&state.db, &id).await {
        Ok(Some(group)) => {
            let member_count = count_group_members(&state.db, &group.id)
                .await
                .unwrap_or(0);
            let response = GroupResponse {
                id: group.id,
                name: group.name,
                description: group.description,
                is_system: group.is_system,
                member_count,
                created_at: group.created_at,
                updated_at: group.updated_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Group not found".to_string(),
                code: "GROUP_NOT_FOUND".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Update a group (admin only)
async fn update_group(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    match update_group_service(
        &state.db,
        &id,
        payload.name,
        payload.description.map(Some),
    )
    .await
    {
        Ok(group) => {
            let member_count = count_group_members(&state.db, &group.id)
                .await
                .unwrap_or(0);
            let response = GroupResponse {
                id: group.id,
                name: group.name,
                description: group.description,
                is_system: group.is_system,
                member_count,
                created_at: group.created_at,
                updated_at: group.updated_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update group: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Delete a group (admin only)
async fn delete_group(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match delete_group_service(&state.db, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete group: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// List members of a group (admin only)
async fn list_group_members(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Check if group exists
    match get_group_by_id(&state.db, &id).await {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Group not found".to_string(),
                    code: "GROUP_NOT_FOUND".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            let (status, json) = e.into();
            return (status, json).into_response();
        }
        Ok(Some(_)) => {}
    }

    match get_group_members(&state.db, &id).await {
        Ok(users) => {
            let response: Vec<MemberResponse> = users
                .into_iter()
                .map(|u| MemberResponse {
                    id: u.id,
                    username: u.username,
                    email: u.email,
                    is_admin: u.is_admin,
                })
                .collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list group members: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Add a member to a group (admin only)
async fn add_group_member(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<AddMemberRequest>,
) -> impl IntoResponse {
    match add_member(&state.db, &id, &payload.user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to add member: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Remove a member from a group (admin only)
async fn remove_group_member(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path((group_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match remove_member(&state.db, &group_id, &user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to remove member: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}
