use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use crate::models::storage::{
    CreatePoolRequest, CreateVolumeRequest, UpdatePoolRequest,
    UpdateVolumeRequest, ResizeVolumeRequest, FsckRequest, WipeDiskRequest, WipeMode,
    CreateSmartScheduleRequest, RunSmartTestRequest, ToggleSmartScheduleRequest,
    UpdateDiskPowerRequest, CreateSnapshotRequest, GrowPoolRequest,
};
use crate::services::storage::StorageService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Disks
        .route("/disks", get(list_disks))
        .route("/disks/:name/smart", get(get_disk_smart))
        .route("/disks/:name/smart/test", post(run_smart_test))
        .route("/disks/:name/smart/history", get(get_smart_test_history))
        .route("/disks/:name/power", get(get_disk_power))
        .route("/disks/:name/power", put(set_disk_power))
        .route("/disks/:name/wipe", post(wipe_disk))
        .route("/candidates", get(get_candidates))
        // SMART schedules
        .route("/smart/schedules", get(list_smart_schedules))
        .route("/smart/schedules", post(create_smart_schedule))
        .route("/smart/schedules/:id", delete(delete_smart_schedule))
        .route("/smart/schedules/:id/toggle", post(toggle_smart_schedule))
        // Pools
        .route("/pools", get(list_pools))
        .route("/pools", post(create_pool))
        .route("/pools/:id", get(get_pool))
        .route("/pools/:id", put(update_pool))
        .route("/pools/:id", delete(delete_pool))
        .route("/pools/:id/health", get(get_pool_health))
        .route("/pools/:id/scrub", post(scrub_pool))
        .route("/pools/:id/grow", post(grow_pool))
        .route("/pools/:id/volumes", post(create_volume))
        // Volumes
        .route("/volumes", get(list_volumes))
        .route("/volumes/:id", get(get_volume))
        .route("/volumes/:id", put(update_volume))
        .route("/volumes/:id", delete(delete_volume))
        .route("/volumes/:id/mount", post(mount_volume))
        .route("/volumes/:id/unmount", post(unmount_volume))
        .route("/volumes/:id/resize", post(resize_volume))
        .route("/volumes/:id/check", post(check_volume))
        .route("/volumes/:id/snapshots", get(list_snapshots))
        .route("/volumes/:id/snapshots", post(create_snapshot))
        .route("/volumes/:id/snapshots/:snap_id", delete(delete_snapshot))
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
    body: Option<Json<WipeDiskRequest>>,
) -> impl IntoResponse {
    let mode = body.and_then(|b| b.mode.clone()).unwrap_or(WipeMode::Quick);
    let service = StorageService::new(state.db.clone());

    match mode {
        WipeMode::Quick => {
            // Synchronous quick wipe (existing behavior)
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
        WipeMode::Zeros | WipeMode::Secure => {
            // Background wipe with progress
            match service.wipe_disk_start(&name) {
                Ok(wipe_status) => {
                    let task_id = wipe_status.task_id.clone();
                    let device_name = name.clone();
                    let task_tx = state.task_tx.clone();
                    let dev_mode = std::env::var("PINAS_DEV_MODE")
                        .map(|v| v.to_lowercase() == "true" || v == "1")
                        .unwrap_or(false);

                    tokio::spawn(async move {
                        StorageService::wipe_disk_execute(device_name, mode, task_id, task_tx, dev_mode).await;
                    });

                    (StatusCode::OK, Json(wipe_status)).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to start wipe for disk {}: {}", name, e);
                    if e.to_string().contains("system disk") {
                        (StatusCode::FORBIDDEN, e.to_string()).into_response()
                    } else {
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                    }
                }
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

/// Get health information for a pool
async fn get_pool_health(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_pool_health(&id).await {
        Ok(health) => Json(health).into_response(),
        Err(e) => {
            tracing::error!("Failed to get pool health {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Start a scrub operation on a pool (RAID verification)
async fn scrub_pool(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.scrub_pool_start(&id).await {
        Ok(scrub_status) => {
            let task_id = scrub_status.task_id.clone();
            let pool_id = id.clone();
            let db = state.db.clone();
            let task_tx = state.task_tx.clone();
            let dev_mode = std::env::var("PINAS_DEV_MODE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            // Execute scrub in background
            tokio::spawn(async move {
                StorageService::scrub_pool_execute(db, pool_id, task_id, task_tx, dev_mode).await;
            });

            (StatusCode::OK, Json(scrub_status)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to start scrub for pool {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
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

/// Update a volume (mount options)
async fn update_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateVolumeRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.update_volume(&id, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to update volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Resize a volume
async fn resize_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ResizeVolumeRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.resize_volume(&id, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to resize volume {}: {}", id, e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Check filesystem on a volume
async fn check_volume(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<FsckRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.fsck_volume_start(&id, request.repair).await {
        Ok(fsck_status) => {
            let task_id = fsck_status.task_id.clone();
            let volume_id = id.clone();
            let db = state.db.clone();
            let task_tx = state.task_tx.clone();
            let repair = request.repair;
            let dev_mode = std::env::var("PINAS_DEV_MODE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            tokio::spawn(async move {
                StorageService::fsck_volume_execute(db, volume_id, task_id, task_tx, repair, dev_mode).await;
            });

            (StatusCode::OK, Json(fsck_status)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to start fsck for volume {}: {}", id, e);
            if e.to_string().contains("unmounted") {
                (StatusCode::CONFLICT, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

// ============ SMART TEST ENDPOINTS ============

/// Run a SMART test on a disk
async fn run_smart_test(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RunSmartTestRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());
    let status = service.smart_test_start(&name, &request.test_type);

    let task_id = status.task_id.clone();
    let device_name = name.clone();
    let test_type = request.test_type.clone();
    let task_tx = state.task_tx.clone();
    let dev_mode = std::env::var("PINAS_DEV_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    tokio::spawn(async move {
        StorageService::smart_test_execute(device_name, test_type, task_id, task_tx, dev_mode).await;
    });

    (StatusCode::OK, Json(status)).into_response()
}

/// Get SMART test history for a disk
async fn get_smart_test_history(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_smart_test_history(&name).await {
        Ok(history) => Json(history).into_response(),
        Err(e) => {
            tracing::error!("Failed to get SMART history for {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// List all SMART test schedules
async fn list_smart_schedules(State(state): State<AppState>) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_smart_schedules().await {
        Ok(schedules) => Json(schedules).into_response(),
        Err(e) => {
            tracing::error!("Failed to list SMART schedules: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create a SMART test schedule
async fn create_smart_schedule(
    State(state): State<AppState>,
    Json(request): Json<CreateSmartScheduleRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.create_smart_schedule(request).await {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create SMART schedule: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Delete a SMART test schedule
async fn delete_smart_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.delete_smart_schedule(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete SMART schedule {}: {}", id, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// Toggle a SMART test schedule
async fn toggle_smart_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ToggleSmartScheduleRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.toggle_smart_schedule(&id, request.enabled).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to toggle SMART schedule {}: {}", id, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

// ============ DISK POWER ENDPOINTS ============

/// Get disk power settings
async fn get_disk_power(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.get_disk_power_settings(&name).await {
        Ok(settings) => Json(settings).into_response(),
        Err(e) => {
            tracing::error!("Failed to get power settings for {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Set disk power settings
async fn set_disk_power(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<UpdateDiskPowerRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.set_disk_power_settings(&name, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to set power settings for {}: {}", name, e);
            if e.to_string().contains("not supported") {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

// ============ SNAPSHOT ENDPOINTS ============

/// List snapshots for a volume
async fn list_snapshots(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.list_snapshots(&id).await {
        Ok(snapshots) => Json(snapshots).into_response(),
        Err(e) => {
            tracing::error!("Failed to list snapshots for volume {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create a snapshot
async fn create_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateSnapshotRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.create_snapshot(&id, request).await {
        Ok(snap_id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": snap_id }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create snapshot for volume {}: {}", id, e);
            if e.to_string().contains("btrfs") || e.to_string().contains("mounted") {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

/// Delete a snapshot
async fn delete_snapshot(
    State(state): State<AppState>,
    Path((id, snap_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.delete_snapshot(&id, &snap_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete snapshot {} for volume {}: {}", snap_id, id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// ============ POOL GROW ENDPOINTS ============

/// Grow a pool by adding disks
async fn grow_pool(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<GrowPoolRequest>,
) -> impl IntoResponse {
    let service = StorageService::new(state.db.clone());

    match service.grow_pool_start(&id, &request).await {
        Ok(grow_status) => {
            let task_id = grow_status.task_id.clone();
            let pool_id = id.clone();
            let db = state.db.clone();
            let task_tx = state.task_tx.clone();
            let new_devices = request.devices.clone();
            let wipe_devices = request.wipe_devices;
            let dev_mode = std::env::var("PINAS_DEV_MODE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            tokio::spawn(async move {
                StorageService::grow_pool_execute(db, pool_id, new_devices, wipe_devices, task_id, task_tx, dev_mode).await;
            });

            (StatusCode::OK, Json(grow_status)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to start grow for pool {}: {}", id, e);
            if e.to_string().contains("Cannot grow") || e.to_string().contains("must be") || e.to_string().contains("requires") {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
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
