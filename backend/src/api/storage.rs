use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;

use crate::models::storage::{CreatePoolRequest, CreateVolumeRequest, UpdatePoolRequest};
use crate::services::storage::StorageService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Disks
        .route("/disks", get(list_disks))
        .route("/disks/:name/smart", get(get_disk_smart))
        .route("/disks/:name/wipe", post(wipe_disk))
        .route("/candidates", get(get_candidates))
        // Pools
        .route("/pools", get(list_pools))
        .route("/pools", post(create_pool))
        .route("/pools/:id", get(get_pool))
        .route("/pools/:id", put(update_pool))
        .route("/pools/:id", delete(delete_pool))
        .route("/pools/:id/scrub", post(scrub_pool))
        .route("/pools/:id/volumes", post(create_volume))
        // Volumes
        .route("/volumes", get(list_volumes))
        .route("/volumes/:id", get(get_volume))
        .route("/volumes/:id", delete(delete_volume))
        .route("/volumes/:id/mount", post(mount_volume))
        .route("/volumes/:id/unmount", post(unmount_volume))
        // Legacy compatibility
        .route("/filesystems", get(list_filesystems))
}

// ============ DISK ENDPOINTS ============

/// List all physical disks with partitions
async fn list_disks(State(state): State<AppState>) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_disks().await {
        Ok(disks) => Json(disks).into_response(),
        Err(e) => {
            tracing::error!("Failed to list disks: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get S.M.A.R.T. information for a disk
async fn get_disk_smart(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_smart_info(&name).await {
        Ok(info) => Json(info).into_response(),
        Err(e) => {
            tracing::error!("Failed to get S.M.A.R.T. info for {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Wipe a disk (destroy all data)
async fn wipe_disk(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.wipe_disk(&name).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to wipe disk {}: {}", name, e);
            if e.to_string().contains("system disk") {
                (StatusCode::FORBIDDEN, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

/// Get disks available for creating pools
async fn get_candidates(State(state): State<AppState>) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_candidates().await {
        Ok(candidates) => Json(candidates).into_response(),
        Err(e) => {
            tracing::error!("Failed to get candidates: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// ============ POOL ENDPOINTS ============

/// List all storage pools
async fn list_pools(State(state): State<AppState>) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_pools().await {
        Ok(pools) => Json(pools).into_response(),
        Err(e) => {
            tracing::error!("Failed to list pools: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create a new storage pool
async fn create_pool(
    State(state): State<AppState>,
    Json(request): Json<CreatePoolRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.create_pool(request).await {
        Ok(pool_id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": pool_id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create pool: {}", e);
            if e.to_string().contains("system device") {
                (StatusCode::FORBIDDEN, e.to_string()).into_response()
            } else {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
        }
    }
}

/// Get a single pool
async fn get_pool(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_pool(&id).await {
        Ok(Some(pool)) => Json(pool).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Failed to get pool {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Update a pool
async fn update_pool(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdatePoolRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.update_pool(&id, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to update pool {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Delete a pool
async fn delete_pool(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.delete_pool(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete pool {}: {}", id, e);
            if e.to_string().contains("mounted volumes") {
                (StatusCode::CONFLICT, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

/// Start a scrub operation on a pool (RAID verification)
async fn scrub_pool(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement scrub for btrfs/mdadm
    tracing::info!("Scrub requested for pool {}", id);
    (StatusCode::NOT_IMPLEMENTED, "Scrub not yet implemented").into_response()
}

// ============ VOLUME ENDPOINTS ============

/// List all volumes
async fn list_volumes(State(state): State<AppState>) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_volumes().await {
        Ok(volumes) => Json(volumes).into_response(),
        Err(e) => {
            tracing::error!("Failed to list volumes: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get a single volume
async fn get_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_volumes().await {
        Ok(volumes) => {
            if let Some(volume) = volumes.into_iter().find(|v| v.id == id) {
                Json(volume).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to get volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create a volume in a pool
async fn create_volume(
    State(state): State<AppState>,
    Path(pool_id): Path<String>,
    Json(request): Json<CreateVolumeRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.create_volume(&pool_id, request).await {
        Ok(volume_id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": volume_id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create volume in pool {}: {}", pool_id, e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Delete a volume
async fn delete_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.delete_volume(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Mount a volume
async fn mount_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.mount_volume(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to mount volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Unmount a volume
async fn unmount_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.unmount_volume(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to unmount volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// ============ LEGACY ENDPOINTS ============

/// List mounted filesystems (legacy compatibility)
async fn list_filesystems(State(state): State<AppState>) -> impl IntoResponse {
    // Return volumes as filesystems for backward compatibility
    let service = StorageService::new(state.db.clone());

    match service.list_volumes().await {
        Ok(volumes) => Json(volumes).into_response(),
        Err(e) => {
            tracing::error!("Failed to list filesystems: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
