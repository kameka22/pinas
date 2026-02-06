use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use std::path::Path;

use crate::api::middleware::AuthUser;
use crate::models::storage::VolumeStatus;
use crate::services::home::HomeService;
use crate::services::permission::PermissionService;
use crate::services::storage::StorageService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_locations))
}

/// A browsable location that can be displayed in the File Manager sidebar
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrowsableLocation {
    Home {
        id: String,
        name: String,
        path: String,
        icon: String,
    },
    Share {
        id: String,
        name: String,
        path: String,
        icon: String,
        share_type: String,
        enabled: bool,
    },
    Volume {
        id: String,
        name: String,
        path: String,
        icon: String,
        status: String,
        fs_type: String,
        usage_percent: f32,
        pool_name: String,
    },
}

/// Get all browsable locations for the current user
async fn get_locations(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let mut locations: Vec<BrowsableLocation> = Vec::new();

    // 1. User's home directory
    let home_service = HomeService::new(&state.config);
    let home_path = home_service.get_home_path(&user.username);

    if home_path.exists() {
        locations.push(BrowsableLocation::Home {
            id: format!("home-{}", user.id),
            name: "My Files".to_string(),
            path: home_path.to_string_lossy().to_string(),
            icon: "mdi:folder-home".to_string(),
        });
    } else {
        // Home doesn't exist yet - create it on demand
        if let Ok(path) = home_service.create_home(&user.username).await {
            locations.push(BrowsableLocation::Home {
                id: format!("home-{}", user.id),
                name: "My Files".to_string(),
                path: path.to_string_lossy().to_string(),
                icon: "mdi:folder-home".to_string(),
            });
        }
    }

    // 2. Folders the user has been granted permission to access
    let permission_service = PermissionService::new(state.db.clone());
    match permission_service.list_by_user(&user.id).await {
        Ok(permissions) => {
            let user_home = home_service.get_home_path(&user.username);
            let user_home_str = user_home.to_string_lossy().to_string();

            // Deduplicate: collect unique paths, keeping the first permission ID
            let mut seen_paths: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for perm in &permissions {
                if perm.permission_level().can_read() {
                    let path = perm.path.trim_end_matches('/').to_string();
                    // Keep the first permission ID for each unique path
                    seen_paths.entry(path).or_insert_with(|| perm.id.clone());
                }
            }

            for (path, perm_id) in seen_paths {
                // Skip the user's own home (already shown above)
                if path.trim_end_matches('/') == user_home_str.trim_end_matches('/') {
                    continue;
                }

                // Only add if the path exists on disk
                if !Path::new(&path).exists() {
                    continue;
                }

                // Derive a display name from the path (last component)
                let name = Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                locations.push(BrowsableLocation::Home {
                    id: format!("perm-{}", perm_id),
                    name,
                    path,
                    icon: "mdi:folder-account".to_string(),
                });
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load user permissions: {}", e);
        }
    }

    // 3. Shared folders from database
    match get_enabled_shares(&state).await {
        Ok(shares) => {
            for share in shares {
                // Check if share path exists
                if Path::new(&share.path).exists() {
                    let icon = match share.share_type.as_str() {
                        "smb" => "mdi:folder-network",
                        "nfs" => "mdi:folder-network-outline",
                        _ => "mdi:folder-star",
                    };

                    locations.push(BrowsableLocation::Share {
                        id: format!("share-{}", share.id),
                        name: share.name,
                        path: share.path,
                        icon: icon.to_string(),
                        share_type: share.share_type,
                        enabled: share.enabled,
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load shares: {}", e);
        }
    }

    // 4. Mounted volumes (for admin users or if volume access control is implemented)
    let storage_service = StorageService::new(state.db.clone());
    match storage_service.list_pools().await {
        Ok(pools) => {
            for pool in pools {
                for volume in pool.volumes {
                    // Only show mounted volumes
                    if volume.status != VolumeStatus::Mounted {
                        continue;
                    }

                    // For now, only admin can see all volumes
                    // In the future, implement per-user volume access control
                    if !user.is_admin {
                        continue;
                    }

                    let status_str = match volume.status {
                        VolumeStatus::Mounted => "mounted",
                        VolumeStatus::Unmounted => "unmounted",
                        VolumeStatus::Error => "error",
                        VolumeStatus::Creating => "creating",
                    };

                    locations.push(BrowsableLocation::Volume {
                        id: format!("volume-{}", volume.id),
                        name: volume.name.clone(),
                        path: volume.mount_point.clone(),
                        icon: "mdi:harddisk".to_string(),
                        status: status_str.to_string(),
                        fs_type: volume.fs_type.clone(),
                        usage_percent: volume.usage_percent as f32,
                        pool_name: pool.name.clone(),
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load volumes: {}", e);
        }
    }

    (StatusCode::OK, Json(locations))
}

/// Share info from database
#[derive(Debug)]
struct ShareInfo {
    id: String,
    name: String,
    path: String,
    share_type: String,
    enabled: bool,
}

/// Get enabled shares from database
async fn get_enabled_shares(state: &AppState) -> Result<Vec<ShareInfo>, sqlx::Error> {
    let shares = sqlx::query_as::<_, (String, String, String, String, bool)>(
        "SELECT id, name, path, share_type, enabled FROM shares WHERE enabled = TRUE ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(shares
        .into_iter()
        .map(|(id, name, path, share_type, enabled)| ShareInfo {
            id,
            name,
            path,
            share_type,
            enabled,
        })
        .collect())
}
