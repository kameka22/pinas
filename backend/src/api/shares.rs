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
        .route("/", get(list_shares))
        .route("/", post(create_share))
        .route("/:id", get(get_share))
        .route("/:id", put(update_share))
        .route("/:id", delete(delete_share))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: ShareType,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareType {
    Smb,
    Nfs,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub name: String,
    pub path: String,
    pub share_type: ShareType,
    pub description: Option<String>,
}

/// List all shares
async fn list_shares(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Get from database
    let shares = vec![
        Share {
            id: "1".to_string(),
            name: "Public".to_string(),
            path: "/srv/public".to_string(),
            share_type: ShareType::Smb,
            enabled: true,
            description: Some("Public share".to_string()),
        },
        Share {
            id: "2".to_string(),
            name: "Media".to_string(),
            path: "/srv/media".to_string(),
            share_type: ShareType::Nfs,
            enabled: true,
            description: Some("Media files".to_string()),
        },
    ];

    Json(shares)
}

/// Create a new share
async fn create_share(
    State(_state): State<AppState>,
    Json(payload): Json<CreateShareRequest>,
) -> impl IntoResponse {
    // TODO: Save to database and generate config
    let share = Share {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        path: payload.path,
        share_type: payload.share_type,
        enabled: true,
        description: payload.description,
    };

    (StatusCode::CREATED, Json(share))
}

/// Get a specific share
async fn get_share(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Get from database
    let share = Share {
        id,
        name: "Public".to_string(),
        path: "/srv/public".to_string(),
        share_type: ShareType::Smb,
        enabled: true,
        description: Some("Public share".to_string()),
    };

    Json(share)
}

/// Update a share
async fn update_share(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateShareRequest>,
) -> impl IntoResponse {
    // TODO: Update in database
    let share = Share {
        id,
        name: payload.name,
        path: payload.path,
        share_type: payload.share_type,
        enabled: true,
        description: payload.description,
    };

    Json(share)
}

/// Delete a share
async fn delete_share(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Delete from database
    StatusCode::NO_CONTENT
}
