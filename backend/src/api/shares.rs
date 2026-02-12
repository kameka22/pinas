use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AdminUser;
use crate::models::{SmbGlobalConfig, SmbShareConfig};
use crate::services::share::{ShareError, ShareService};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shares))
        .route("/", post(create_share))
        .route("/:id", get(get_share))
        .route("/:id", put(update_share))
        .route("/:id", delete(delete_share_handler))
        .route("/:id/toggle", post(toggle_share))
        .route("/samba/status", get(get_samba_status))
        .route("/samba/enable", post(enable_samba))
        .route("/samba/disable", post(disable_samba))
        .route("/samba/config", get(get_smb_config))
        .route("/samba/config", put(update_smb_config))
}

// ─── Request / Response Types ─────────────────────────────────────

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub name: String,
    pub path: String,
    #[serde(default = "default_share_type")]
    pub share_type: String,
    pub description: Option<String>,
    pub config: Option<SmbShareConfig>,
}

fn default_share_type() -> String {
    "smb".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateShareRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub config: Option<SmbShareConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

// ─── Error Mapping ────────────────────────────────────────────────

impl From<ShareError> for (StatusCode, Json<ErrorResponse>) {
    fn from(err: ShareError) -> Self {
        let (status, code) = match &err {
            ShareError::NotFound => (StatusCode::NOT_FOUND, "SHARE_NOT_FOUND"),
            ShareError::DuplicateName => (StatusCode::CONFLICT, "DUPLICATE_NAME"),
            ShareError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            ShareError::SystemError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "SYSTEM_ERROR"),
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

// ─── Share CRUD Handlers ──────────────────────────────────────────

/// List all shares
async fn list_shares(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.list_shares().await {
        Ok(shares) => (StatusCode::OK, Json(shares)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list shares: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Create a new share
async fn create_share(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<CreateShareRequest>,
) -> impl IntoResponse {
    // Validate input
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Share name is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.path.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Share path is required".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    if payload.share_type != "smb" && payload.share_type != "nfs" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Share type must be 'smb' or 'nfs'".to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        )
            .into_response();
    }

    let svc = ShareService::new(state.db.clone());

    match svc
        .create_share(
            &payload.name,
            &payload.path,
            &payload.share_type,
            payload.description,
            payload.config,
        )
        .await
    {
        Ok(share) => (StatusCode::CREATED, Json(share)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create share: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Get a specific share
async fn get_share(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.get_share(&id).await {
        Ok(share) => (StatusCode::OK, Json(share)).into_response(),
        Err(e) => {
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Update a share
async fn update_share(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateShareRequest>,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc
        .update_share(
            &id,
            payload.name.as_deref(),
            payload.description,
            payload.config,
        )
        .await
    {
        Ok(share) => (StatusCode::OK, Json(share)).into_response(),
        Err(e) => {
            tracing::error!("Failed to update share: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Delete a share
async fn delete_share_handler(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.delete_share(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete share: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Toggle share enabled/disabled
async fn toggle_share(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Json(payload): Json<ToggleRequest>,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.toggle_share(&id, payload.enabled).await {
        Ok(share) => (StatusCode::OK, Json(share)).into_response(),
        Err(e) => {
            tracing::error!("Failed to toggle share: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

// ─── Samba Service Handlers ───────────────────────────────────────

/// Get Samba service status
async fn get_samba_status(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.get_samba_status().await {
        Ok(status) => (StatusCode::OK, Json(status)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get Samba status: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Enable Samba service
async fn enable_samba(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.enable_samba().await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to enable Samba: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Disable Samba service
async fn disable_samba(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.disable_samba().await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to disable Samba: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Get global SMB configuration
async fn get_smb_config(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.get_smb_config().await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get SMB config: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}

/// Update global SMB configuration
async fn update_smb_config(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<SmbGlobalConfig>,
) -> impl IntoResponse {
    let svc = ShareService::new(state.db.clone());

    match svc.update_smb_config(&payload).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("Failed to update SMB config: {}", e);
            let (status, json) = e.into();
            (status, json).into_response()
        }
    }
}
