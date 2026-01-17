use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sysinfo::Disks;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/disks", get(get_disks))
        .route("/filesystems", get(get_filesystems))
}

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f32,
    pub is_removable: bool,
}

/// Get list of disks
async fn get_disks(State(_state): State<AppState>) -> impl IntoResponse {
    let disks = Disks::new_with_refreshed_list();

    let disk_list: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
                is_removable: disk.is_removable(),
            }
        })
        .collect();

    Json(disk_list)
}

/// Get mounted filesystems
async fn get_filesystems(State(_state): State<AppState>) -> impl IntoResponse {
    // Same as disks for now, but could include more details
    let disks = Disks::new_with_refreshed_list();

    let fs_list: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
                is_removable: disk.is_removable(),
            }
        })
        .collect();

    Json(fs_list)
}
