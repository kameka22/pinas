use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::PermissionEntry;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Share {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String, // "smb" or "nfs"
    pub enabled: bool,
    pub description: Option<String>,
    pub config: Option<String>, // JSON config
    pub created_at: String,
    pub updated_at: String,
}

impl Share {
    pub fn new(name: String, path: String, share_type: String, description: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            path,
            share_type,
            enabled: true,
            description,
            config: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Parse the JSON config field into SmbShareConfig
    pub fn smb_config(&self) -> SmbShareConfig {
        self.config
            .as_ref()
            .and_then(|c| serde_json::from_str(c).ok())
            .unwrap_or_default()
    }

    /// Serialize SmbShareConfig into the JSON config field
    pub fn set_smb_config(&mut self, cfg: &SmbShareConfig) {
        self.config = serde_json::to_string(cfg).ok();
    }
}

/// SMB-specific configuration stored as JSON in shares.config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbShareConfig {
    #[serde(default)]
    pub guest_ok: bool,
    #[serde(default = "default_true")]
    pub browseable: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default = "default_create_mask")]
    pub create_mask: String,
    #[serde(default = "default_directory_mask")]
    pub directory_mask: String,
    #[serde(default)]
    pub veto_files: Option<String>,
    #[serde(default)]
    pub recycle_bin: bool,
}

fn default_true() -> bool {
    true
}

fn default_create_mask() -> String {
    "0644".to_string()
}

fn default_directory_mask() -> String {
    "0755".to_string()
}

impl Default for SmbShareConfig {
    fn default() -> Self {
        Self {
            guest_ok: false,
            browseable: true,
            read_only: false,
            create_mask: default_create_mask(),
            directory_mask: default_directory_mask(),
            veto_files: None,
            recycle_bin: false,
        }
    }
}

/// Global Samba configuration (stored in settings table as smb.* keys)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmbGlobalConfig {
    #[serde(default = "default_workgroup")]
    pub workgroup: String,
    #[serde(default = "default_server_string")]
    pub server_string: String,
    #[serde(default = "default_min_protocol")]
    pub min_protocol: String,
    #[serde(default = "default_max_protocol")]
    pub max_protocol: String,
}

fn default_workgroup() -> String {
    "WORKGROUP".to_string()
}

fn default_server_string() -> String {
    "PiNAS".to_string()
}

fn default_min_protocol() -> String {
    "SMB2".to_string()
}

fn default_max_protocol() -> String {
    "SMB3".to_string()
}

impl Default for SmbGlobalConfig {
    fn default() -> Self {
        Self {
            workgroup: default_workgroup(),
            server_string: default_server_string(),
            min_protocol: default_min_protocol(),
            max_protocol: default_max_protocol(),
        }
    }
}

/// Samba service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SambaStatus {
    pub enabled: bool,
    pub running: bool,
    pub share_count: u32,
    pub connected_users: u32,
    pub version: Option<String>,
}

/// Enriched share info for API responses
#[derive(Debug, Clone, Serialize)]
pub struct ShareInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub share_type: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub config: SmbShareConfig,
    pub permissions: Vec<PermissionEntry>,
    pub created_at: String,
    pub updated_at: String,
}

impl ShareInfo {
    pub fn from_share(share: Share, permissions: Vec<PermissionEntry>) -> Self {
        let config = share.smb_config();
        Self {
            id: share.id,
            name: share.name,
            path: share.path,
            share_type: share.share_type,
            enabled: share.enabled,
            description: share.description,
            config,
            permissions,
            created_at: share.created_at,
            updated_at: share.updated_at,
        }
    }
}
