use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AdminUser;
use crate::models::service_access::UserServiceAccess;
use crate::services::service_access::ServiceAccessService;
use crate::services::share::ShareService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_service_access))
        .route("/:user_id", get(get_user_service_access))
        .route("/:user_id", put(update_user_service_access))
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct UpdateServiceAccessRequest {
    smb: Option<bool>,
    nfs: Option<bool>,
    ftp: Option<bool>,
}

/// List all users with their service access (admin only)
async fn list_service_access(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> impl IntoResponse {
    let svc = ServiceAccessService::new(state.db.clone());

    match svc.list_all_access().await {
        Ok(access) => (StatusCode::OK, Json(access)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list service access: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to list service access".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get service access for a specific user (admin only)
async fn get_user_service_access(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let svc = ServiceAccessService::new(state.db.clone());

    match svc.get_user_access(&user_id).await {
        Ok(access) => (StatusCode::OK, Json(access)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get service access: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to get service access".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Update service access for a user (admin only)
async fn update_user_service_access(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateServiceAccessRequest>,
) -> impl IntoResponse {
    let svc = ServiceAccessService::new(state.db.clone());
    let share_svc = ShareService::new(state.db.clone());

    // Get username for smbpasswd operations
    let username: Option<String> = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let username = match username {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    code: "USER_NOT_FOUND".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Track if SMB access changed for smbpasswd management
    let old_smb = svc.is_user_smb_enabled(&user_id).await.unwrap_or(false);

    // Update each service if provided
    if let Some(smb) = payload.smb {
        if let Err(e) = svc.set_user_access(&user_id, "smb", smb).await {
            tracing::error!("Failed to set SMB access: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update SMB access".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response();
        }
    }

    if let Some(nfs) = payload.nfs {
        if let Err(e) = svc.set_user_access(&user_id, "nfs", nfs).await {
            tracing::error!("Failed to set NFS access: {}", e);
        }
    }

    if let Some(ftp) = payload.ftp {
        if let Err(e) = svc.set_user_access(&user_id, "ftp", ftp).await {
            tracing::error!("Failed to set FTP access: {}", e);
        }
    }

    // Handle SMB user activation/deactivation
    let new_smb = payload.smb.unwrap_or(old_smb);
    if new_smb != old_smb {
        if new_smb {
            // Enable: activate existing smbpasswd entry
            if let Err(e) = share_svc.enable_samba_user(&username).await {
                tracing::warn!("Failed to enable Samba user {}: {}", username, e);
            }
        } else {
            // Disable: deactivate smbpasswd entry
            if let Err(e) = share_svc.disable_samba_user(&username).await {
                tracing::warn!("Failed to disable Samba user {}: {}", username, e);
            }
        }
        // Regenerate smb.conf to update valid_users
        share_svc.regenerate_and_reload_public().await;
    }

    // Return updated access
    match svc.get_user_access(&user_id).await {
        Ok(access) => (StatusCode::OK, Json(access)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get updated service access: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to get updated access".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
                .into_response()
        }
    }
}
