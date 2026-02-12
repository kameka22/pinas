use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tokio::process::Command as AsyncCommand;

use crate::models::share::{
    SambaStatus, ShareInfo, SharePermissionEntry, SmbGlobalConfig, SmbShareConfig, Share,
};

/// Share service errors
#[derive(Debug, Error)]
pub enum ShareError {
    #[error("Share not found")]
    NotFound,

    #[error("Share name already exists")]
    DuplicateName,

    #[error("Invalid share name: {0}")]
    InvalidName(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Samba service is not enabled")]
    SambaNotEnabled,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Request to create a new share
#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub name: String,
    pub path: String,
    #[serde(default = "default_share_type")]
    pub share_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub guest_ok: bool,
    #[serde(default = "default_true")]
    pub browseable: bool,
    #[serde(default)]
    pub read_only: bool,
}

fn default_share_type() -> String {
    "smb".to_string()
}
fn default_true() -> bool {
    true
}

/// Request to update an existing share
#[derive(Debug, Deserialize)]
pub struct UpdateShareRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub guest_ok: Option<bool>,
    pub browseable: Option<bool>,
    pub read_only: Option<bool>,
}

/// Request to toggle a share
#[derive(Debug, Deserialize)]
pub struct ToggleShareRequest {
    pub enabled: bool,
}

// Dev mode state
static DEV_SAMBA_ENABLED: AtomicBool = AtomicBool::new(false);

const SMB_CONF_PATH: &str = "/etc/samba/smb.conf";

/// Share and Samba service manager
pub struct ShareService {
    db: SqlitePool,
    dev_mode: bool,
    shares_root: String,
}

impl ShareService {
    pub fn new(db: SqlitePool) -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let shares_root = std::env::var("PINAS_SHARES_ROOT")
            .unwrap_or_else(|_| "/storage/shares".to_string());

        if dev_mode {
            tracing::debug!("ShareService running in dev mode");
        }

        Self {
            db,
            dev_mode,
            shares_root,
        }
    }

    // ─── CRUD Operations ─────────────────────────────────────────────

    /// List all shares with their parsed config and permissions
    pub async fn list_shares(&self) -> Result<Vec<ShareInfo>, ShareError> {
        let shares = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::with_capacity(shares.len());
        for share in shares {
            let permissions = self.get_share_permissions(&share.path).await?;
            let mut info = ShareInfo::from(share);
            info.permissions = permissions;
            result.push(info);
        }

        Ok(result)
    }

    /// Get a single share by ID
    pub async fn get_share(&self, id: &str) -> Result<ShareInfo, ShareError> {
        let share = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        let permissions = self.get_share_permissions(&share.path).await?;
        let mut info = ShareInfo::from(share);
        info.permissions = permissions;

        Ok(info)
    }

    /// Create a new share
    pub async fn create_share(&self, req: CreateShareRequest) -> Result<ShareInfo, ShareError> {
        // Validate name
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(ShareError::InvalidName("Share name cannot be empty".to_string()));
        }
        if name.len() > 80 {
            return Err(ShareError::InvalidName("Share name too long (max 80 chars)".to_string()));
        }

        // Validate path
        let path = req.path.trim().to_string();
        if path.is_empty() || !path.starts_with('/') {
            return Err(ShareError::InvalidPath("Path must be absolute".to_string()));
        }

        // Check for duplicate name
        let existing = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE name = ?",
        )
        .bind(&name)
        .fetch_optional(&self.db)
        .await?;

        if existing.is_some() {
            return Err(ShareError::DuplicateName);
        }

        // Build SMB config
        let smb_config = SmbShareConfig {
            guest_ok: req.guest_ok,
            browseable: req.browseable,
            read_only: req.read_only,
            ..Default::default()
        };

        let mut share = Share::new(name, path.clone(), req.share_type, req.description);
        share.set_smb_config(&smb_config);

        // Insert into DB
        sqlx::query(
            r#"
            INSERT INTO shares (id, name, path, share_type, enabled, description, config, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&share.id)
        .bind(&share.name)
        .bind(&share.path)
        .bind(&share.share_type)
        .bind(share.enabled)
        .bind(&share.description)
        .bind(&share.config)
        .bind(&share.created_at)
        .bind(&share.updated_at)
        .execute(&self.db)
        .await?;

        // Create directory if it doesn't exist
        self.ensure_directory(&path).await;

        // Regenerate smb.conf and reload
        self.apply_samba_config().await;

        let info = ShareInfo::from(share);
        Ok(info)
    }

    /// Update an existing share
    pub async fn update_share(&self, id: &str, req: UpdateShareRequest) -> Result<ShareInfo, ShareError> {
        let mut share = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        // Check for duplicate name if renaming
        if let Some(ref new_name) = req.name {
            let trimmed = new_name.trim();
            if trimmed.is_empty() {
                return Err(ShareError::InvalidName("Share name cannot be empty".to_string()));
            }
            let existing = sqlx::query_as::<_, Share>(
                "SELECT * FROM shares WHERE name = ? AND id != ?",
            )
            .bind(trimmed)
            .bind(id)
            .fetch_optional(&self.db)
            .await?;

            if existing.is_some() {
                return Err(ShareError::DuplicateName);
            }
            share.name = trimmed.to_string();
        }

        if let Some(desc) = req.description {
            share.description = Some(desc);
        }

        // Update SMB config
        let mut smb_config = share.smb_config();
        if let Some(v) = req.guest_ok {
            smb_config.guest_ok = v;
        }
        if let Some(v) = req.browseable {
            smb_config.browseable = v;
        }
        if let Some(v) = req.read_only {
            smb_config.read_only = v;
        }
        share.set_smb_config(&smb_config);

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE shares SET name = ?, description = ?, config = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&share.name)
        .bind(&share.description)
        .bind(&share.config)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;

        share.updated_at = now;

        // Regenerate smb.conf and reload
        self.apply_samba_config().await;

        let permissions = self.get_share_permissions(&share.path).await?;
        let mut info = ShareInfo::from(share);
        info.permissions = permissions;
        Ok(info)
    }

    /// Delete a share (removes from DB but NOT the directory)
    pub async fn delete_share(&self, id: &str) -> Result<(), ShareError> {
        let share = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        // Delete folder permissions for this share's path
        sqlx::query("DELETE FROM folder_permissions WHERE path = ?")
            .bind(&share.path)
            .execute(&self.db)
            .await?;

        // Delete the share
        sqlx::query("DELETE FROM shares WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Regenerate smb.conf and reload
        self.apply_samba_config().await;

        tracing::info!("Deleted share '{}' (directory {} preserved)", share.name, share.path);
        Ok(())
    }

    /// Toggle share enabled/disabled
    pub async fn toggle_share(&self, id: &str, enabled: bool) -> Result<ShareInfo, ShareError> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE shares SET enabled = ?, updated_at = ? WHERE id = ?",
        )
        .bind(enabled)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ShareError::NotFound);
        }

        // Regenerate smb.conf and reload
        self.apply_samba_config().await;

        self.get_share(id).await
    }

    // ─── Samba Service Control ───────────────────────────────────────

    /// Get Samba service status
    pub async fn get_samba_status(&self) -> Result<SambaStatus, ShareError> {
        if self.dev_mode {
            let enabled = DEV_SAMBA_ENABLED.load(Ordering::Relaxed);
            let share_count = if enabled {
                sqlx::query_as::<_, (i64,)>(
                    "SELECT COUNT(*) FROM shares WHERE enabled = TRUE AND share_type = 'smb'",
                )
                .fetch_one(&self.db)
                .await
                .map(|(c,)| c as u32)
                .unwrap_or(0)
            } else {
                0
            };

            return Ok(SambaStatus {
                enabled,
                running: enabled,
                share_count,
                connected_users: 0,
                version: Some("4.x (dev mode)".to_string()),
            });
        }

        let running = self.is_samba_running().await;
        let enabled = self.is_samba_enabled().await || running;

        let share_count = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM shares WHERE enabled = TRUE AND share_type = 'smb'",
        )
        .fetch_one(&self.db)
        .await
        .map(|(c,)| c as u32)
        .unwrap_or(0);

        let connected_users = self.get_connected_users().await.unwrap_or(0);
        let version = self.get_samba_version().await;

        Ok(SambaStatus {
            enabled,
            running,
            share_count,
            connected_users,
            version,
        })
    }

    /// Enable Samba service
    pub async fn enable_samba(&self) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Enabling Samba service");
            DEV_SAMBA_ENABLED.store(true, Ordering::Relaxed);
            // Generate config for the first time
            self.apply_samba_config().await;
            return Ok(());
        }

        let output = AsyncCommand::new("systemctl")
            .args(["enable", "smbd", "nmbd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to enable Samba: {}", stderr);
        }

        let output = AsyncCommand::new("systemctl")
            .args(["start", "smbd", "nmbd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to start Samba: {}", stderr);
        }

        // Generate config
        self.apply_samba_config().await;

        tracing::info!("Samba service enabled and started");
        Ok(())
    }

    /// Disable Samba service
    pub async fn disable_samba(&self) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Disabling Samba service");
            DEV_SAMBA_ENABLED.store(false, Ordering::Relaxed);
            return Ok(());
        }

        let output = AsyncCommand::new("systemctl")
            .args(["stop", "smbd", "nmbd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to stop Samba: {}", stderr);
        }

        let output = AsyncCommand::new("systemctl")
            .args(["disable", "smbd", "nmbd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to disable Samba: {}", stderr);
        }

        tracing::info!("Samba service disabled");
        Ok(())
    }

    // ─── Global SMB Configuration ───────────────────────────────────

    /// Get global SMB configuration from settings table
    pub async fn get_smb_config(&self) -> Result<SmbGlobalConfig, ShareError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM settings WHERE key LIKE 'smb.%'",
        )
        .fetch_all(&self.db)
        .await?;

        let mut config = SmbGlobalConfig::default();
        for (key, value) in rows {
            match key.as_str() {
                "smb.workgroup" => config.workgroup = value,
                "smb.server_string" => config.server_string = value,
                "smb.min_protocol" => config.min_protocol = value,
                "smb.max_protocol" => config.max_protocol = value,
                _ => {}
            }
        }

        Ok(config)
    }

    /// Update global SMB configuration
    pub async fn update_smb_config(&self, config: SmbGlobalConfig) -> Result<(), ShareError> {
        let now = chrono::Utc::now().to_rfc3339();

        let pairs = [
            ("smb.workgroup", &config.workgroup),
            ("smb.server_string", &config.server_string),
            ("smb.min_protocol", &config.min_protocol),
            ("smb.max_protocol", &config.max_protocol),
        ];

        for (key, value) in &pairs {
            sqlx::query(
                r#"
                INSERT INTO settings (key, value, updated_at)
                VALUES (?, ?, ?)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
                "#,
            )
            .bind(key)
            .bind(value)
            .bind(&now)
            .execute(&self.db)
            .await?;
        }

        // Regenerate smb.conf and reload
        self.apply_samba_config().await;

        tracing::info!("Updated SMB global config: workgroup={}", config.workgroup);
        Ok(())
    }

    // ─── Samba User Sync ─────────────────────────────────────────────

    /// Sync a PiNAS user to Samba (add or update password)
    pub async fn sync_samba_user(&self, username: &str, password: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would sync Samba user: {}", username);
            return Ok(());
        }

        // smbpasswd -a -s reads password from stdin (two lines: password + confirm)
        let mut child = AsyncCommand::new("smbpasswd")
            .args(["-a", "-s", username])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let input = format!("{}\n{}\n", password, password);
            stdin.write_all(input.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to sync Samba user '{}': {}", username, stderr);
            return Err(ShareError::Internal(format!(
                "Failed to sync Samba user: {}",
                stderr
            )));
        }

        tracing::info!("Synced Samba user: {}", username);
        Ok(())
    }

    /// Remove a Samba user
    pub async fn remove_samba_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would remove Samba user: {}", username);
            return Ok(());
        }

        let output = AsyncCommand::new("smbpasswd")
            .args(["-x", username])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to remove Samba user '{}': {}", username, stderr);
            // Non-fatal: user might not exist in Samba
        }

        tracing::info!("Removed Samba user: {}", username);
        Ok(())
    }

    // ─── smb.conf Generation ────────────────────────────────────────

    /// Generate the complete smb.conf content from database state
    pub async fn generate_smb_conf(&self) -> Result<String, ShareError> {
        let global = self.get_smb_config().await?;

        let shares = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE enabled = TRUE AND share_type = 'smb' ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;

        let mut conf = String::with_capacity(2048);

        // Global section
        conf.push_str("# Generated by PiNAS - DO NOT EDIT MANUALLY\n");
        conf.push_str("# Changes will be overwritten when shares are modified\n\n");
        conf.push_str("[global]\n");
        conf.push_str(&format!("   workgroup = {}\n", global.workgroup));
        conf.push_str(&format!("   server string = {}\n", global.server_string));
        conf.push_str(&format!("   min protocol = {}\n", global.min_protocol));
        conf.push_str(&format!("   max protocol = {}\n", global.max_protocol));
        conf.push_str("   security = user\n");
        conf.push_str("   map to guest = Bad User\n");
        conf.push_str("   dns proxy = no\n");
        conf.push_str("   server role = standalone server\n");
        conf.push_str("\n");
        // Performance tuning for Raspberry Pi
        conf.push_str("   # Performance\n");
        conf.push_str("   socket options = TCP_NODELAY IPTOS_LOWDELAY\n");
        conf.push_str("   read raw = yes\n");
        conf.push_str("   write raw = yes\n");
        conf.push_str("   use sendfile = yes\n");
        conf.push_str("   aio read size = 16384\n");
        conf.push_str("   aio write size = 16384\n");
        conf.push_str("\n");

        // Per-share sections
        for share in &shares {
            let smb_config = share.smb_config();
            let (read_users, write_users) = self.resolve_share_users(&share.path).await?;

            conf.push_str(&format!("[{}]\n", share.name));
            conf.push_str(&format!("   path = {}\n", share.path));
            conf.push_str(&format!(
                "   browseable = {}\n",
                if smb_config.browseable { "yes" } else { "no" }
            ));
            conf.push_str(&format!(
                "   guest ok = {}\n",
                if smb_config.guest_ok { "yes" } else { "no" }
            ));
            conf.push_str(&format!(
                "   read only = {}\n",
                if smb_config.read_only { "yes" } else { "no" }
            ));

            // Build valid users list (all users with any permission)
            let mut all_valid: Vec<String> = Vec::new();
            all_valid.extend(read_users.iter().cloned());
            all_valid.extend(write_users.iter().cloned());
            all_valid.sort();
            all_valid.dedup();

            if !all_valid.is_empty() && !smb_config.guest_ok {
                conf.push_str(&format!("   valid users = {}\n", all_valid.join(" ")));
            }

            if !read_users.is_empty() {
                conf.push_str(&format!("   read list = {}\n", read_users.join(" ")));
            }
            if !write_users.is_empty() {
                conf.push_str(&format!("   write list = {}\n", write_users.join(" ")));
            }

            conf.push_str(&format!("   create mask = {}\n", smb_config.create_mask));
            conf.push_str(&format!("   directory mask = {}\n", smb_config.directory_mask));

            if !smb_config.veto_files.is_empty() {
                conf.push_str(&format!("   veto files = {}\n", smb_config.veto_files));
                conf.push_str("   delete veto files = yes\n");
            }

            if smb_config.recycle_bin {
                conf.push_str("   vfs objects = recycle\n");
                conf.push_str("   recycle:repository = .recycle\n");
                conf.push_str("   recycle:keeptree = yes\n");
                conf.push_str("   recycle:versions = yes\n");
            }

            if let Some(ref desc) = share.description {
                conf.push_str(&format!("   comment = {}\n", desc));
            }

            conf.push('\n');
        }

        Ok(conf)
    }

    /// Write smb.conf to disk and reload Samba
    async fn apply_samba_config(&self) {
        match self.generate_smb_conf().await {
            Ok(conf) => {
                if self.dev_mode {
                    tracing::info!("[DEV MODE] Would write smb.conf:\n{}", conf);
                } else {
                    if let Err(e) = self.write_smb_conf(&conf).await {
                        tracing::error!("Failed to write smb.conf: {}", e);
                        return;
                    }
                    self.reload_samba().await;
                }
            }
            Err(e) => {
                tracing::error!("Failed to generate smb.conf: {}", e);
            }
        }
    }

    /// Write smb.conf atomically (temp file + rename)
    async fn write_smb_conf(&self, content: &str) -> Result<(), ShareError> {
        let conf_path = std::path::Path::new(SMB_CONF_PATH);
        let tmp_path = conf_path.with_extension("tmp");

        tokio::fs::write(&tmp_path, content).await?;
        tokio::fs::rename(&tmp_path, conf_path).await?;

        tracing::info!("Wrote smb.conf ({} bytes)", content.len());
        Ok(())
    }

    /// Reload Samba configuration
    async fn reload_samba(&self) {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would reload Samba");
            return;
        }

        // Try smbcontrol first (graceful reload)
        let output = AsyncCommand::new("smbcontrol")
            .args(["all", "reload-config"])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                tracing::info!("Samba configuration reloaded via smbcontrol");
            }
            _ => {
                // Fallback: restart the service
                tracing::info!("smbcontrol failed, restarting smbd...");
                let _ = AsyncCommand::new("systemctl")
                    .args(["restart", "smbd"])
                    .output()
                    .await;
            }
        }
    }

    // ─── Internal Helpers ────────────────────────────────────────────

    /// Resolve users with read/write permissions for a share path
    async fn resolve_share_users(
        &self,
        path: &str,
    ) -> Result<(Vec<String>, Vec<String>), ShareError> {
        let rows: Vec<(Option<String>, Option<String>, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                r#"
                SELECT
                    fp.user_id,
                    u.username,
                    fp.group_id,
                    g.name as group_name,
                    fp.permission
                FROM folder_permissions fp
                LEFT JOIN users u ON fp.user_id = u.id
                LEFT JOIN user_groups g ON fp.group_id = g.id
                WHERE fp.path = ?
                "#,
            )
            .bind(path)
            .fetch_all(&self.db)
            .await?;

        let mut read_users: Vec<String> = Vec::new();
        let mut write_users: Vec<String> = Vec::new();

        for (user_id, username, group_id, group_name, permission) in rows {
            let name = if let Some(uname) = username {
                uname
            } else if let Some(gname) = group_name {
                // Samba group syntax: @groupname
                format!("@{}", gname)
            } else {
                continue;
            };

            match permission.as_str() {
                "read" => read_users.push(name),
                "write" => write_users.push(name),
                _ => {} // "none" or unknown
            }
        }

        Ok((read_users, write_users))
    }

    /// Get permissions for a share path (for API response)
    async fn get_share_permissions(
        &self,
        path: &str,
    ) -> Result<Vec<SharePermissionEntry>, ShareError> {
        let rows: Vec<(Option<String>, Option<String>, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                r#"
                SELECT
                    fp.user_id,
                    u.username,
                    fp.group_id,
                    g.name as group_name,
                    fp.permission
                FROM folder_permissions fp
                LEFT JOIN users u ON fp.user_id = u.id
                LEFT JOIN user_groups g ON fp.group_id = g.id
                WHERE fp.path = ?
                ORDER BY u.username, g.name
                "#,
            )
            .bind(path)
            .fetch_all(&self.db)
            .await?;

        let entries: Vec<SharePermissionEntry> = rows
            .into_iter()
            .map(
                |(user_id, username, group_id, group_name, permission)| SharePermissionEntry {
                    user_id,
                    username,
                    group_id,
                    group_name,
                    permission,
                },
            )
            .collect();

        Ok(entries)
    }

    /// Ensure a directory exists (create if needed)
    async fn ensure_directory(&self, path: &str) {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would create directory: {}", path);
            return;
        }

        match tokio::fs::create_dir_all(path).await {
            Ok(()) => tracing::info!("Created directory: {}", path),
            Err(e) => tracing::warn!("Failed to create directory '{}': {}", path, e),
        }
    }

    /// Check if smbd is running
    async fn is_samba_running(&self) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-active", "smbd"])
            .output()
            .await;

        if let Ok(o) = output {
            return String::from_utf8_lossy(&o.stdout).trim() == "active";
        }
        false
    }

    /// Check if smbd is enabled at boot
    async fn is_samba_enabled(&self) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-enabled", "smbd"])
            .output()
            .await;

        if let Ok(o) = output {
            return String::from_utf8_lossy(&o.stdout).trim() == "enabled";
        }
        false
    }

    /// Get number of connected SMB users
    async fn get_connected_users(&self) -> Result<u32, ShareError> {
        let output = AsyncCommand::new("smbstatus")
            .args(["--brief", "--no-header"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(0);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|l| !l.trim().is_empty()).count();
        Ok(count as u32)
    }

    /// Get Samba version string
    async fn get_samba_version(&self) -> Option<String> {
        let output = AsyncCommand::new("smbd")
            .args(["--version"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(version)
        } else {
            None
        }
    }
}
