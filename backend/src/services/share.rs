use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use tokio::process::Command as AsyncCommand;

use std::collections::HashSet;

use crate::models::{
    PermissionEntry, SambaStatus, Share, ShareInfo, SmbGlobalConfig, SmbShareConfig,
};
use crate::services::permission::PermissionService;
use crate::services::service_access::ServiceAccessService;

const SAMBA_CONF_PATH: &str = "/storage/.pinas/data/samba/smb.conf";
const SAMBA_PRIVATE_DIR: &str = "/storage/.pinas/data/samba";

/// Share service errors
#[derive(Debug, Error)]
pub enum ShareError {
    #[error("Share not found")]
    NotFound,

    #[error("Share name already exists")]
    DuplicateName,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("System error: {0}")]
    SystemError(String),
}

// Dev mode state
static DEV_SAMBA_ENABLED: AtomicBool = AtomicBool::new(false);

/// Share and Samba service manager
pub struct ShareService {
    db: SqlitePool,
    dev_mode: bool,
}

impl ShareService {
    pub fn new(db: SqlitePool) -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        Self { db, dev_mode }
    }

    // ─── CRUD Operations ─────────────────────────────────────────────

    /// List all shares with enriched info
    pub async fn list_shares(&self) -> Result<Vec<ShareInfo>, ShareError> {
        let shares = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares ORDER BY name ASC",
        )
        .fetch_all(&self.db)
        .await?;

        let perm_svc = PermissionService::new(self.db.clone());
        let mut result = Vec::with_capacity(shares.len());

        for share in shares {
            let permissions = perm_svc.list_by_folder(&share.path).await.unwrap_or_default();
            result.push(ShareInfo::from_share(share, permissions));
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

        let perm_svc = PermissionService::new(self.db.clone());
        let permissions = perm_svc.list_by_folder(&share.path).await.unwrap_or_default();

        Ok(ShareInfo::from_share(share, permissions))
    }

    /// Create a new share
    pub async fn create_share(
        &self,
        name: &str,
        path: &str,
        share_type: &str,
        description: Option<String>,
        config: Option<SmbShareConfig>,
    ) -> Result<ShareInfo, ShareError> {
        // Check for duplicate name
        let existing = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        if existing.is_some() {
            return Err(ShareError::DuplicateName);
        }

        let mut share = Share::new(
            name.to_string(),
            path.to_string(),
            share_type.to_string(),
            description,
        );

        if let Some(cfg) = &config {
            share.set_smb_config(cfg);
        }

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

        // Create physical directory
        self.ensure_share_directory(path).await;

        // Regenerate smb.conf and reload
        if share.share_type == "smb" {
            self.regenerate_and_reload().await;
        }

        let perm_svc = PermissionService::new(self.db.clone());
        let permissions = perm_svc.list_by_folder(&share.path).await.unwrap_or_default();

        Ok(ShareInfo::from_share(share, permissions))
    }

    /// Update an existing share
    pub async fn update_share(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<Option<String>>,
        config: Option<SmbShareConfig>,
    ) -> Result<ShareInfo, ShareError> {
        let existing = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        // Check for duplicate name if changing
        if let Some(new_name) = name {
            if new_name != existing.name {
                let dup = sqlx::query_as::<_, Share>(
                    "SELECT * FROM shares WHERE name = ? AND id != ?",
                )
                .bind(new_name)
                .bind(id)
                .fetch_optional(&self.db)
                .await?;

                if dup.is_some() {
                    return Err(ShareError::DuplicateName);
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let final_name = name.unwrap_or(&existing.name);
        let final_description = match description {
            Some(d) => d,
            None => existing.description.clone(),
        };
        let final_config = match config {
            Some(cfg) => serde_json::to_string(&cfg).ok(),
            None => existing.config.clone(),
        };

        sqlx::query(
            r#"
            UPDATE shares
            SET name = ?, description = ?, config = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(final_name)
        .bind(&final_description)
        .bind(&final_config)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;

        if existing.share_type == "smb" {
            self.regenerate_and_reload().await;
        }

        self.get_share(id).await
    }

    /// Delete a share (removes from DB and permissions, keeps the directory)
    pub async fn delete_share(&self, id: &str) -> Result<(), ShareError> {
        let share = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        // Delete associated folder permissions
        sqlx::query("DELETE FROM folder_permissions WHERE path = ?")
            .bind(&share.path)
            .execute(&self.db)
            .await?;

        // Delete the share record
        sqlx::query("DELETE FROM shares WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        if share.share_type == "smb" {
            self.regenerate_and_reload().await;
        }

        Ok(())
    }

    /// Toggle share enabled/disabled
    pub async fn toggle_share(&self, id: &str, enabled: bool) -> Result<ShareInfo, ShareError> {
        let existing = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(ShareError::NotFound)?;

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE shares SET enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(&now)
            .bind(id)
            .execute(&self.db)
            .await?;

        if existing.share_type == "smb" {
            self.regenerate_and_reload().await;
        }

        self.get_share(id).await
    }

    // ─── Samba Service Control ───────────────────────────────────────

    /// Get Samba service status
    pub async fn get_samba_status(&self) -> Result<SambaStatus, ShareError> {
        let share_count = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM shares WHERE share_type = 'smb' AND enabled = TRUE",
        )
        .fetch_one(&self.db)
        .await?
        .0 as u32;

        if self.dev_mode {
            let enabled = DEV_SAMBA_ENABLED.load(Ordering::Relaxed);
            return Ok(SambaStatus {
                enabled,
                running: enabled,
                share_count,
                connected_users: 0,
                version: Some("4.x.x (dev mode)".to_string()),
            });
        }

        let running = self.is_service_active("pinas-smbd").await;
        let enabled = self.is_service_enabled("pinas-smbd").await || running;
        let version = self.get_samba_version().await;
        let connected_users = self.count_samba_connections().await;

        Ok(SambaStatus {
            enabled,
            running,
            share_count,
            connected_users,
            version,
        })
    }

    /// Enable Samba services (smbd + nmbd)
    pub async fn enable_samba(&self) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Enabling Samba services (smbd, nmbd)");
            DEV_SAMBA_ENABLED.store(true, Ordering::Relaxed);
            return Ok(());
        }

        // Regenerate smb.conf before enabling services
        self.regenerate_and_reload().await;

        for svc in &["pinas-smbd", "pinas-nmbd"] {
            let output = AsyncCommand::new("systemctl")
                .args(["enable", "--now", svc])
                .output()
                .await
                .map_err(|e| ShareError::SystemError(e.to_string()))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to enable {}: {}", svc, stderr);
            }
        }

        tracing::info!("PiNAS Samba services enabled and started");
        Ok(())
    }

    /// Disable Samba services
    pub async fn disable_samba(&self) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Disabling Samba services (smbd, nmbd)");
            DEV_SAMBA_ENABLED.store(false, Ordering::Relaxed);
            return Ok(());
        }

        for svc in &["pinas-smbd", "pinas-nmbd"] {
            let output = AsyncCommand::new("systemctl")
                .args(["disable", "--now", svc])
                .output()
                .await
                .map_err(|e| ShareError::SystemError(e.to_string()))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to disable {}: {}", svc, stderr);
            }
        }

        tracing::info!("PiNAS Samba services disabled");
        Ok(())
    }

    // ─── SMB Global Configuration ───────────────────────────────────

    /// Read global SMB config from settings table
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

    /// Update global SMB config in settings table
    pub async fn update_smb_config(&self, config: &SmbGlobalConfig) -> Result<(), ShareError> {
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
                INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
                "#,
            )
            .bind(key)
            .bind(value)
            .bind(&now)
            .execute(&self.db)
            .await?;
        }

        self.regenerate_and_reload().await;

        Ok(())
    }

    // ─── Samba User Sync ─────────────────────────────────────────────

    /// Sync a user to Samba (smbpasswd -a -s)
    pub async fn sync_samba_user(&self, username: &str, password: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would sync Samba user: {}", username);
            return Ok(());
        }

        // Ensure system user exists (required by smbpasswd)
        self.ensure_system_user(username).await?;

        let mut child = AsyncCommand::new("smbpasswd")
            .args(["-c", SAMBA_CONF_PATH, "-a", "-s", username])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ShareError::SystemError(format!("Failed to spawn smbpasswd: {}", e)))?;

        // smbpasswd expects password twice on stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            let input = format!("{}\n{}\n", password, password);
            stdin.write_all(input.as_bytes()).await
                .map_err(|e| ShareError::SystemError(format!("Failed to write to smbpasswd: {}", e)))?;
        }

        let output = child.wait_with_output().await
            .map_err(|e| ShareError::SystemError(format!("smbpasswd failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("smbpasswd sync failed for {}: {}", username, stderr);
        } else {
            tracing::info!("Samba user synced: {}", username);
        }

        Ok(())
    }

    /// Remove a user from Samba (smbpasswd -x)
    pub async fn remove_samba_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would remove Samba user: {}", username);
            return Ok(());
        }

        let output = AsyncCommand::new("smbpasswd")
            .args(["-c", SAMBA_CONF_PATH, "-x", username])
            .output()
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to run smbpasswd: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("smbpasswd remove failed for {}: {}", username, stderr);
        } else {
            tracing::info!("Samba user removed: {}", username);
        }

        // Remove system user from /etc/passwd
        self.remove_system_user(username).await?;

        Ok(())
    }

    /// Enable a Samba user account (smbpasswd -e)
    pub async fn enable_samba_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would enable Samba user: {}", username);
            return Ok(());
        }

        // Ensure system user exists (may have been lost on reboot)
        self.ensure_system_user(username).await?;

        let output = AsyncCommand::new("smbpasswd")
            .args(["-c", SAMBA_CONF_PATH, "-e", username])
            .output()
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to run smbpasswd -e: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("smbpasswd enable failed for {}: {}", username, stderr);
        } else {
            tracing::info!("Samba user enabled: {}", username);
        }

        Ok(())
    }

    /// Disable a Samba user account (smbpasswd -d)
    pub async fn disable_samba_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would disable Samba user: {}", username);
            return Ok(());
        }

        let output = AsyncCommand::new("smbpasswd")
            .args(["-c", SAMBA_CONF_PATH, "-d", username])
            .output()
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to run smbpasswd -d: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("smbpasswd disable failed for {}: {}", username, stderr);
        } else {
            tracing::info!("Samba user disabled: {}", username);
        }

        Ok(())
    }

    /// Ensure system user exists in /etc/passwd for Samba auth.
    /// LibreELEC /etc/passwd is tmpfs — resets to root-only on reboot.
    pub async fn ensure_system_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            return Ok(());
        }
        if username == "root" {
            return Ok(());
        }

        // Check if already exists
        let check = AsyncCommand::new("id")
            .args(["-u", username])
            .output()
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to run id: {}", e)))?;
        if check.status.success() {
            return Ok(());
        }

        // Create: no home (-M), no shell (-s /bin/false)
        let output = AsyncCommand::new("useradd")
            .args(["-M", "-s", "/bin/false", username])
            .output()
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to run useradd: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() != Some(9) {
                // 9 = already exists
                return Err(ShareError::SystemError(format!(
                    "useradd failed for {}: {}",
                    username, stderr
                )));
            }
        }
        tracing::info!("System user created: {}", username);
        Ok(())
    }

    /// Remove system user from /etc/passwd
    pub async fn remove_system_user(&self, username: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            return Ok(());
        }
        if username == "root" {
            return Ok(());
        }

        let _ = AsyncCommand::new("userdel").arg(username).output().await;
        Ok(())
    }

    /// Initialize Samba on backend startup
    /// Ensures LibreELEC default Samba is disabled and PiNAS smb.conf is generated
    pub async fn initialize_samba(&self) {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Samba initialization skipped");
            return;
        }

        // Ensure LibreELEC default Samba is disabled
        let disabled_marker = "/storage/.cache/services/samba.disabled";
        if tokio::fs::metadata(disabled_marker).await.is_err() {
            tracing::info!("Creating samba.disabled marker to prevent LibreELEC default Samba");
            if let Err(e) = tokio::fs::create_dir_all("/storage/.cache/services").await {
                tracing::warn!("Failed to create services dir: {}", e);
            }
            if let Err(e) = tokio::fs::write(disabled_marker, "").await {
                tracing::warn!("Failed to create samba.disabled: {}", e);
            }
        }

        // Stop LibreELEC default Samba services
        for svc in &["smbd", "nmbd", "samba-config"] {
            let _ = AsyncCommand::new("systemctl")
                .args(["stop", svc])
                .output()
                .await;
        }

        // Create PiNAS Samba config directory
        if let Err(e) = tokio::fs::create_dir_all("/storage/.pinas/data/samba").await {
            tracing::warn!("Failed to create samba dir: {}", e);
        }

        // Create Samba log directory (may not exist on LibreELEC tmpfs)
        if let Err(e) = tokio::fs::create_dir_all("/var/log/samba").await {
            tracing::warn!("Failed to create samba log dir: {}", e);
        }

        // Regenerate smb.conf from database state
        self.regenerate_and_reload().await;

        // Resync system users from PiNAS DB (tmpfs /etc/passwd resets on reboot,
        // but passdb.tdb in /storage is persistent — no password resync needed)
        let users: Vec<(String,)> = sqlx::query_as("SELECT username FROM users")
            .fetch_all(&self.db)
            .await
            .unwrap_or_default();

        for (username,) in &users {
            if let Err(e) = self.ensure_system_user(username).await {
                tracing::warn!("Failed to ensure system user {} at boot: {}", username, e);
            }
        }
        tracing::info!("Resynced {} system users for Samba", users.len());

        tracing::info!("PiNAS Samba initialization complete");
    }

    // ─── smb.conf Generation ─────────────────────────────────────────

    /// Generate smb.conf content from DB state
    async fn generate_smb_conf(&self) -> Result<String, ShareError> {
        let global_config = self.get_smb_config().await?;

        let shares = sqlx::query_as::<_, Share>(
            "SELECT * FROM shares WHERE share_type = 'smb' AND enabled = TRUE ORDER BY name",
        )
        .fetch_all(&self.db)
        .await?;

        let perm_svc = PermissionService::new(self.db.clone());
        let sa_svc = ServiceAccessService::new(self.db.clone());
        let smb_authorized: HashSet<String> = sa_svc
            .get_smb_authorized_usernames()
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();

        // Get hostname for netbios name
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "pinas".to_string());

        let mut conf = String::new();

        // Global section
        conf.push_str("[global]\n");
        conf.push_str(&format!("   workgroup = {}\n", global_config.workgroup));
        conf.push_str(&format!("   netbios name = {}\n", hostname));
        conf.push_str(&format!("   server string = {}\n", global_config.server_string));
        conf.push_str(&format!("   min protocol = {}\n", global_config.min_protocol));
        conf.push_str(&format!("   max protocol = {}\n", global_config.max_protocol));
        conf.push_str("   security = user\n");
        conf.push_str("   map to guest = never\n");
        conf.push_str("   log file = /var/log/samba/%m.log\n");
        conf.push_str("   max log size = 1000\n");
        conf.push_str("   logging = file\n");
        conf.push_str("   passdb backend = tdbsam\n");
        conf.push_str(&format!("   private dir = {}\n", SAMBA_PRIVATE_DIR));
        conf.push_str("   obey pam restrictions = no\n");
        conf.push_str("   use sendfile = yes\n");
        conf.push('\n');

        // Per-share sections
        for share in &shares {
            let smb_cfg = share.smb_config();
            let permissions = perm_svc.list_by_folder(&share.path).await.unwrap_or_default();

            conf.push_str(&format!("[{}]\n", share.name));
            conf.push_str(&format!("   path = {}\n", share.path));

            if let Some(ref desc) = share.description {
                conf.push_str(&format!("   comment = {}\n", desc));
            }

            conf.push_str(&format!("   browseable = {}\n", if smb_cfg.browseable { "yes" } else { "no" }));
            conf.push_str(&format!("   read only = {}\n", if smb_cfg.read_only { "yes" } else { "no" }));
            conf.push_str(&format!("   guest ok = {}\n", if smb_cfg.guest_ok { "yes" } else { "no" }));
            conf.push_str(&format!("   create mask = {}\n", smb_cfg.create_mask));
            conf.push_str(&format!("   force create mode = {}\n", smb_cfg.create_mask));
            conf.push_str(&format!("   directory mask = {}\n", smb_cfg.directory_mask));
            conf.push_str(&format!("   force directory mode = {}\n", smb_cfg.directory_mask));
            conf.push_str("   force user = root\n");
            conf.push_str("   force group = root\n");
            conf.push_str("   inherit acls = yes\n");
            conf.push_str("   inherit permissions = yes\n");
            conf.push_str("   ea support = yes\n");
            conf.push_str("   store dos attributes = yes\n");
            conf.push_str("   hide special files = yes\n");
            conf.push_str("   hide dot files = yes\n");

            if let Some(ref veto) = smb_cfg.veto_files {
                conf.push_str(&format!("   veto files = {}\n", veto));
            }

            // Build VFS objects list dynamically
            let mut vfs_objects: Vec<&str> = Vec::new();

            if smb_cfg.fruit_enabled {
                vfs_objects.push("fruit");
                vfs_objects.push("streams_xattr");
            }

            if smb_cfg.recycle_bin {
                vfs_objects.push("recycle");
            }

            if smb_cfg.audit_enabled {
                vfs_objects.push("full_audit");
            }

            if !vfs_objects.is_empty() {
                conf.push_str(&format!("   vfs objects = {}\n", vfs_objects.join(" ")));
            }

            // VFS module configurations
            if smb_cfg.fruit_enabled {
                conf.push_str("   fruit:encoding = private\n");
                conf.push_str("   fruit:metadata = stream\n");
                conf.push_str("   fruit:resource = file\n");
                conf.push_str("   fruit:locking = none\n");
                conf.push_str("   fruit:delete_empty_adfiles = yes\n");
                conf.push_str("   fruit:wipe_intentionally_left_blank_rfork = yes\n");
                conf.push_str("   fruit:veto_appledouble = no\n");
                conf.push_str("   fruit:time machine = yes\n");
            }

            if smb_cfg.recycle_bin {
                conf.push_str("   recycle:repository = .recycle/%U\n");
                conf.push_str("   recycle:keeptree = yes\n");
                conf.push_str("   recycle:versions = yes\n");
                conf.push_str("   recycle:touch = yes\n");
                conf.push_str("   recycle:directory_mode = 0777\n");
                conf.push_str("   recycle:subdir_mode = 0700\n");
                conf.push_str("   recycle:exclude = *.tmp,*.TMP,*.temp,*.o,~$*\n");
                conf.push_str("   recycle:exclude_dir = /tmp,/cache,/TEMP\n");
            }

            if smb_cfg.audit_enabled {
                conf.push_str("   full_audit:prefix = %u|%I|%m|%S\n");
                conf.push_str("   full_audit:success = mkdirat renameat unlinkat pwrite\n");
                conf.push_str("   full_audit:failure = connect\n");
                conf.push_str("   full_audit:facility = local7\n");
                conf.push_str("   full_audit:priority = NOTICE\n");
            }

            if let Some(ref encrypt) = smb_cfg.smb_encrypt {
                if encrypt != "off" && !encrypt.is_empty() {
                    conf.push_str(&format!("   smb encrypt = {}\n", encrypt));
                }
            }

            if let Some(ref hosts) = smb_cfg.hosts_allow {
                if !hosts.is_empty() {
                    conf.push_str(&format!("   hosts allow = {}\n", hosts));
                }
            }

            if let Some(ref hosts) = smb_cfg.hosts_deny {
                if !hosts.is_empty() {
                    conf.push_str(&format!("   hosts deny = {}\n", hosts));
                }
            }

            if let Some(ref extra) = smb_cfg.extra_options {
                if !extra.is_empty() {
                    for line in extra.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            conf.push_str(&format!("   {}\n", trimmed));
                        }
                    }
                }
            }

            // Resolve permissions: build valid users, read list, write list
            let mut valid_users = Vec::new();
            let mut read_list = Vec::new();
            let mut write_list = Vec::new();

            for entry in &permissions {
                let principal = if let Some(ref uname) = entry.username {
                    uname.clone()
                } else if let Some(ref gname) = entry.group_name {
                    format!("@{}", gname)
                } else {
                    continue;
                };

                match entry.permission.as_str() {
                    "read" => {
                        valid_users.push(principal.clone());
                        read_list.push(principal);
                    }
                    "write" => {
                        valid_users.push(principal.clone());
                        write_list.push(principal);
                    }
                    _ => {} // "none" = no access
                }
            }

            // Filter lists to only include users with SMB access enabled
            // Groups (prefixed with @) pass through as they are checked separately
            let filter_by_smb = |users: Vec<String>| -> Vec<String> {
                users
                    .into_iter()
                    .filter(|u| u.starts_with('@') || smb_authorized.contains(u))
                    .collect()
            };

            let valid_users = filter_by_smb(valid_users);
            let read_list = filter_by_smb(read_list);
            let write_list = filter_by_smb(write_list);

            if !smb_cfg.guest_ok && !valid_users.is_empty() {
                conf.push_str(&format!("   valid users = {}\n", valid_users.join(" ")));
            }
            if !read_list.is_empty() {
                conf.push_str(&format!("   read list = {}\n", read_list.join(" ")));
            }
            if !write_list.is_empty() {
                conf.push_str(&format!("   write list = {}\n", write_list.join(" ")));
            }

            conf.push('\n');
        }

        Ok(conf)
    }

    /// Write smb.conf atomically (temp file + rename)
    async fn write_smb_conf(&self, content: &str) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would write smb.conf:\n{}", content);
            return Ok(());
        }

        let conf_path = SAMBA_CONF_PATH;
        let tmp_path = format!("{}.tmp", SAMBA_CONF_PATH);

        tokio::fs::write(&tmp_path, content)
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to write temp smb.conf: {}", e)))?;

        tokio::fs::rename(&tmp_path, conf_path)
            .await
            .map_err(|e| ShareError::SystemError(format!("Failed to rename smb.conf: {}", e)))?;

        tracing::info!("smb.conf updated");
        Ok(())
    }

    /// Reload Samba configuration
    async fn reload_samba(&self) -> Result<(), ShareError> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would reload Samba configuration");
            return Ok(());
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
                // Fallback: restart smbd
                tracing::info!("smbcontrol failed, falling back to systemctl restart");
                let output = AsyncCommand::new("systemctl")
                    .args(["restart", "pinas-smbd"])
                    .output()
                    .await
                    .map_err(|e| ShareError::SystemError(format!("Failed to restart pinas-smbd: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("Failed to restart pinas-smbd: {}", stderr);
                }
            }
        }

        Ok(())
    }

    /// Public method to regenerate smb.conf and reload Samba
    pub async fn regenerate_and_reload_public(&self) {
        self.regenerate_and_reload().await;
    }

    /// Validate smb.conf using testparm
    async fn validate_smb_conf(&self) -> bool {
        if self.dev_mode {
            return true;
        }

        let output = AsyncCommand::new("testparm")
            .args(["-s", "--suppress-prompt", SAMBA_CONF_PATH])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => true,
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                tracing::warn!("smb.conf validation failed: {}", stderr);
                false
            }
            Err(e) => {
                // testparm not available — skip validation
                tracing::debug!("testparm not available, skipping validation: {}", e);
                true
            }
        }
    }

    /// Regenerate smb.conf and reload Samba (convenience method)
    async fn regenerate_and_reload(&self) {
        match self.generate_smb_conf().await {
            Ok(content) => {
                if let Err(e) = self.write_smb_conf(&content).await {
                    tracing::warn!("Failed to write smb.conf: {}", e);
                    return;
                }
                if !self.validate_smb_conf().await {
                    tracing::error!("Generated smb.conf is invalid, skipping reload");
                    return;
                }
                if let Err(e) = self.reload_samba().await {
                    tracing::warn!("Failed to reload Samba: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to generate smb.conf: {}", e);
            }
        }
    }

    // ─── Helper Methods ──────────────────────────────────────────────

    /// Create share directory if it doesn't exist
    async fn ensure_share_directory(&self, path: &str) {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would create directory: {}", path);
            return;
        }

        if let Err(e) = tokio::fs::create_dir_all(path).await {
            tracing::warn!("Failed to create share directory {}: {}", path, e);
        }
    }

    /// Check if a systemd service is active
    async fn is_service_active(&self, name: &str) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-active", name])
            .output()
            .await;

        output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false)
    }

    /// Check if a systemd service is enabled
    async fn is_service_enabled(&self, name: &str) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-enabled", name])
            .output()
            .await;

        output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
            .unwrap_or(false)
    }

    /// Get Samba version string
    async fn get_samba_version(&self) -> Option<String> {
        let output = AsyncCommand::new("smbd")
            .args(["--version"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Count active Samba connections
    async fn count_samba_connections(&self) -> u32 {
        let output = AsyncCommand::new("smbstatus")
            .args(["--brief", "--numeric"])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                // Count non-header lines (skip first 4 lines of smbstatus output)
                stdout.lines().skip(4).filter(|l| !l.trim().is_empty()).count() as u32
            }
            _ => 0,
        }
    }
}
