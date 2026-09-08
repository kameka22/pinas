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

    /// Parse the JSON config field into NfsShareConfig (for `share_type == "nfs"`)
    pub fn nfs_config(&self) -> NfsShareConfig {
        self.config
            .as_ref()
            .and_then(|c| serde_json::from_str(c).ok())
            .unwrap_or_default()
    }

    pub fn set_nfs_config(&mut self, cfg: &NfsShareConfig) {
        self.config = serde_json::to_string(cfg).ok();
    }
}

/// NFS export configuration stored as JSON in shares.config
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NfsShareConfig {
    /// Allowed clients: IP, CIDR, hostname or `*`
    #[serde(default = "default_clients")]
    pub clients: Vec<String>,
    #[serde(default)]
    pub read_only: bool,
    /// `sync` (safe) vs `async` (fast)
    #[serde(default = "default_true")]
    pub sync: bool,
    /// root_squash | no_root_squash | all_squash
    #[serde(default = "default_squash")]
    pub squash: String,
    #[serde(default)]
    pub subtree_check: bool,
}

fn default_clients() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_squash() -> String {
    "root_squash".to_string()
}

impl Default for NfsShareConfig {
    fn default() -> Self {
        Self { clients: default_clients(), read_only: false, sync: true, squash: default_squash(), subtree_check: false }
    }
}

impl NfsShareConfig {
    /// Validate client specs and the squash mode
    pub fn validate(&self) -> Result<(), String> {
        if self.clients.is_empty() {
            return Err("At least one client is required".to_string());
        }
        for c in &self.clients {
            let ok = c == "*"
                || c.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '/' | ':' | '*' | '_'));
            if !ok || c.contains("..") {
                return Err(format!("Invalid client specification: {}", c));
            }
        }
        if !matches!(self.squash.as_str(), "root_squash" | "no_root_squash" | "all_squash") {
            return Err(format!("Invalid squash mode: {}", self.squash));
        }
        Ok(())
    }

    /// Options string as understood by exportfs / /etc/exports
    pub fn export_options(&self) -> String {
        let mut opts = vec![
            if self.read_only { "ro" } else { "rw" },
            if self.sync { "sync" } else { "async" },
            self.squash.as_str(),
            if self.subtree_check { "subtree_check" } else { "no_subtree_check" },
        ];
        // anonymous writes land as root-owned files otherwise (everything is root on LibreELEC)
        if self.squash == "all_squash" {
            opts.push("anonuid=0,anongid=0");
        }
        opts.join(",")
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
    #[serde(default)]
    pub fruit_enabled: bool,
    #[serde(default)]
    pub smb_encrypt: Option<String>,
    #[serde(default)]
    pub hosts_allow: Option<String>,
    #[serde(default)]
    pub hosts_deny: Option<String>,
    #[serde(default)]
    pub audit_enabled: bool,
    #[serde(default)]
    pub extra_options: Option<String>,
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
            fruit_enabled: false,
            smb_encrypt: None,
            hosts_allow: None,
            hosts_deny: None,
            audit_enabled: false,
            extra_options: None,
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

/// NFS server status
#[derive(Debug, Clone, Serialize)]
pub struct NfsStatus {
    pub enabled: bool,
    pub running: bool,
    pub export_count: u32,
    /// rpc.nfsd binary and kernel nfsd filesystem present on this image
    pub nfsd_available: bool,
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
    /// Present for NFS exports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfs: Option<NfsShareConfig>,
    pub permissions: Vec<PermissionEntry>,
    pub created_at: String,
    pub updated_at: String,
}

impl ShareInfo {
    pub fn from_share(share: Share, permissions: Vec<PermissionEntry>) -> Self {
        let config = share.smb_config();
        let nfs = if share.share_type == "nfs" { Some(share.nfs_config()) } else { None };
        Self {
            id: share.id,
            name: share.name,
            path: share.path,
            share_type: share.share_type,
            enabled: share.enabled,
            description: share.description,
            config,
            nfs,
            permissions,
            created_at: share.created_at,
            updated_at: share.updated_at,
        }
    }
}

#[cfg(test)]
mod nfs_tests {
    use super::*;

    #[test]
    fn export_options_follow_the_config() {
        let d = NfsShareConfig::default();
        assert_eq!(d.export_options(), "rw,sync,root_squash,no_subtree_check");
        let ro = NfsShareConfig { read_only: true, sync: false, squash: "all_squash".into(), subtree_check: true, ..Default::default() };
        assert_eq!(ro.export_options(), "ro,async,all_squash,subtree_check,anonuid=0,anongid=0");
    }

    #[test]
    fn client_specs_and_squash_are_validated() {
        assert!(NfsShareConfig::default().validate().is_ok());
        assert!(NfsShareConfig { clients: vec!["192.168.1.0/24".into(), "nas-client".into()], ..Default::default() }.validate().is_ok());
        assert!(NfsShareConfig { clients: vec![], ..Default::default() }.validate().is_err());
        assert!(NfsShareConfig { clients: vec!["10.0.0.1 rw)".into()], ..Default::default() }.validate().is_err());
        assert!(NfsShareConfig { squash: "maybe".into(), ..Default::default() }.validate().is_err());
    }

    #[test]
    fn share_round_trips_nfs_config() {
        let mut share = Share::new("media".into(), "/storage/shares/media".into(), "nfs".into(), None);
        let cfg = NfsShareConfig { clients: vec!["192.168.1.0/24".into()], read_only: true, ..Default::default() };
        share.set_nfs_config(&cfg);
        assert_eq!(share.nfs_config(), cfg);
        let info = ShareInfo::from_share(share, vec![]);
        assert_eq!(info.nfs, Some(cfg));
    }
}
