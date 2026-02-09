use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::api::ws::TaskProgressEvent;

/// GitHub release info
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

/// update.json inside the archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub min_version: Option<String>,
    pub date: String,
    #[serde(rename = "type")]
    pub update_type: String,
    pub reboot_required: bool,
    pub changelog: HashMap<String, String>,
    pub contents: UpdateContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContents {
    #[serde(default)]
    pub backend: bool,
    #[serde(default)]
    pub frontend: bool,
    #[serde(default)]
    pub migrations: bool,
    #[serde(default)]
    pub scripts: bool,
    #[serde(default)]
    pub services: bool,
    #[serde(default)]
    pub system: bool,
}

/// Result of checking for updates
#[derive(Debug, Serialize)]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub update_type: Option<String>,
    pub reboot_required: Option<bool>,
    pub changelog: Option<HashMap<String, String>>,
    pub download_size: Option<u64>,
    pub published_at: Option<String>,
}

/// Info stored after an update is applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppliedInfo {
    pub version: String,
    pub previous_version: String,
    pub changelog: HashMap<String, String>,
}

/// Update history entry
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UpdateHistoryEntry {
    pub id: String,
    pub version: String,
    pub previous_version: String,
    pub update_type: String,
    pub status: String,
    pub changelog: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// Update service handles checking for updates, downloading, and applying them
pub struct UpdateService {
    db: SqlitePool,
    task_tx: broadcast::Sender<TaskProgressEvent>,
    dev_mode: bool,
    data_dir: String,
    github_owner: String,
    github_repo: String,
}

impl UpdateService {
    pub fn new(db: SqlitePool, task_tx: broadcast::Sender<TaskProgressEvent>) -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let data_dir = std::env::var("PINAS_DATA_DIR")
            .unwrap_or_else(|_| "/storage/.pinas".to_string());

        let github_owner = std::env::var("PINAS_GITHUB_OWNER")
            .unwrap_or_else(|_| "kameka22".to_string());

        let github_repo = std::env::var("PINAS_GITHUB_REPO")
            .unwrap_or_else(|_| "pinas".to_string());

        if dev_mode {
            tracing::info!("UpdateService running in dev mode - simulated updates");
        }

        Self {
            db,
            task_tx,
            dev_mode,
            data_dir,
            github_owner,
            github_repo,
        }
    }

    /// Get current version from VERSION file
    fn current_version() -> String {
        include_str!("../../../VERSION").trim().to_string()
    }

    /// Check GitHub Releases for a newer version
    pub async fn check_for_update(&self) -> Result<UpdateCheckResult> {
        let current = Self::current_version();

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.github_owner, self.github_repo
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "PiNAS-Update-Service")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Ok(UpdateCheckResult {
                    available: false,
                    latest_version: current.clone(),
                    current_version: current,
                    update_type: None,
                    reboot_required: None,
                    changelog: None,
                    download_size: None,
                    published_at: None,
                });
            }
            return Err(anyhow!("GitHub API error: HTTP {}", response.status()));
        }

        let release: GitHubRelease = response.json().await?;
        let latest = release.tag_name.trim_start_matches('v').to_string();

        if !is_newer_version(&current, &latest) {
            return Ok(UpdateCheckResult {
                available: false,
                current_version: current,
                latest_version: latest,
                update_type: None,
                reboot_required: None,
                changelog: None,
                download_size: None,
                published_at: release.published_at,
            });
        }

        // Find the update archive asset
        let arch = std::env::consts::ARCH;
        let asset = release.assets.iter().find(|a| {
            a.name.starts_with("pinas-update-") && a.name.ends_with(".tar.gz")
                && (a.name.contains(arch) || !a.name.contains("x86_64") && !a.name.contains("aarch64"))
        });

        let (download_size, update_type) = if let Some(asset) = asset {
            (Some(asset.size), Some(determine_update_type(&current, &latest)))
        } else {
            (None, Some(determine_update_type(&current, &latest)))
        };

        let changelog = release.body.map(|body| {
            HashMap::from([("en".to_string(), body)])
        });

        Ok(UpdateCheckResult {
            available: true,
            current_version: current,
            latest_version: latest,
            update_type,
            reboot_required: None,
            changelog,
            download_size,
            published_at: release.published_at,
        })
    }

    /// Start the update install process. Returns a task_id for tracking.
    pub async fn install_update_start(&self, version: &str) -> Result<String> {
        let current = Self::current_version();
        let task_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let update_type = determine_update_type(&current, version);

        sqlx::query(
            r#"INSERT INTO system_updates (id, version, previous_version, update_type, status, started_at, created_at)
               VALUES (?, ?, ?, ?, 'downloading', ?, ?)"#,
        )
        .bind(&task_id)
        .bind(version)
        .bind(&current)
        .bind(&update_type)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(task_id)
    }

    /// Execute the update (should be called in tokio::spawn after install_update_start)
    pub async fn install_update_execute(&self, task_id: &str) {
        let result = if self.dev_mode {
            self.execute_dev_mode_update(task_id).await
        } else {
            self.execute_real_update(task_id).await
        };

        let now = chrono::Utc::now().to_rfc3339();
        match result {
            Ok(_) => {
                // Status will be set to 'completed' or 'reboot_required' by the inner logic
                tracing::info!("Update {} completed/applied successfully", task_id);
            }
            Err(ref e) => {
                let error_msg = e.to_string();
                tracing::error!("Update {} failed: {}", task_id, error_msg);

                let _ = sqlx::query(
                    "UPDATE system_updates SET status = 'failed', error_message = ?, completed_at = ? WHERE id = ?"
                )
                .bind(&error_msg)
                .bind(&now)
                .bind(task_id)
                .execute(&self.db)
                .await;

                self.broadcast_progress(task_id, "failed", 0, 100, Some("Update failed"), Some(&error_msg));
            }
        }
    }

    /// Simulate update in dev mode
    async fn execute_dev_mode_update(&self, task_id: &str) -> Result<()> {
        let steps = [
            ("Checking for updates...", 10, 1000),
            ("Downloading update archive...", 30, 2000),
            ("Downloading update archive...", 50, 1500),
            ("Extracting update...", 60, 1000),
            ("Validating update manifest...", 65, 500),
            ("Backing up database...", 70, 800),
            ("Applying backend update...", 80, 1000),
            ("Applying frontend update...", 90, 1000),
            ("Finalizing update...", 95, 500),
        ];

        for (desc, progress, delay_ms) in &steps {
            tracing::info!("[DEV MODE] Update step: {} ({}%)", desc, progress);
            self.broadcast_progress(task_id, "running", *progress, 100, Some(desc), None);
            tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
        }

        // Write the update-applied flag
        let info = UpdateAppliedInfo {
            version: "99.0.0".to_string(),
            previous_version: Self::current_version(),
            changelog: HashMap::from([
                ("en".to_string(), "Dev mode simulated update: new features and improvements.".to_string()),
                ("fr".to_string(), "Mise à jour simulée en mode dev : nouvelles fonctionnalités et améliorations.".to_string()),
            ]),
        };

        let flag_path = format!("{}/data/.update-applied", self.data_dir);
        if let Some(parent) = Path::new(&flag_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        let flag_json = serde_json::to_string_pretty(&info)?;
        fs::write(&flag_path, flag_json).await?;

        // Mark as completed
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE system_updates SET status = 'completed', completed_at = ? WHERE id = ?"
        )
        .bind(&now)
        .bind(task_id)
        .execute(&self.db)
        .await?;

        self.broadcast_progress(task_id, "completed", 100, 100, Some("Update complete"), None);
        Ok(())
    }

    /// Execute a real update
    async fn execute_real_update(&self, task_id: &str) -> Result<()> {
        self.broadcast_progress(task_id, "running", 5, 100, Some("Checking for update..."), None);

        // 1. Fetch latest release info
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.github_owner, self.github_repo
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "PiNAS-Update-Service")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("GitHub API error: HTTP {}", response.status()));
        }

        let release: GitHubRelease = response.json().await?;
        let latest = release.tag_name.trim_start_matches('v').to_string();

        // Find the update archive
        let arch = std::env::consts::ARCH;
        let asset = release.assets.iter().find(|a| {
            a.name.starts_with("pinas-update-") && a.name.ends_with(".tar.gz")
                && (a.name.contains(arch) || (!a.name.contains("x86_64") && !a.name.contains("aarch64")))
        }).ok_or_else(|| anyhow!("No update archive found for architecture {}", arch))?;

        // 2. Download archive
        self.broadcast_progress(task_id, "running", 10, 100, Some("Downloading update..."), None);

        let download_response = client
            .get(&asset.browser_download_url)
            .header("User-Agent", "PiNAS-Update-Service")
            .send()
            .await?;

        if !download_response.status().is_success() {
            return Err(anyhow!("Download failed: HTTP {}", download_response.status()));
        }

        let archive_bytes = download_response.bytes().await?;
        self.broadcast_progress(task_id, "running", 40, 100, Some("Download complete"), None);

        // 3. Extract to /tmp/pinas-update/
        let extract_dir = "/tmp/pinas-update";
        if fs::metadata(extract_dir).await.is_ok() {
            fs::remove_dir_all(extract_dir).await?;
        }
        fs::create_dir_all(extract_dir).await?;

        let archive_path = format!("{}/update.tar.gz", extract_dir);
        let mut file = fs::File::create(&archive_path).await?;
        file.write_all(&archive_bytes).await?;
        drop(file);

        self.broadcast_progress(task_id, "running", 50, 100, Some("Extracting update..."), None);

        let extract_dir_clone = extract_dir.to_string();
        let archive_path_clone = archive_path.clone();
        tokio::task::spawn_blocking(move || {
            use flate2::read::GzDecoder;
            use std::fs::File;
            use tar::Archive;

            let file = File::open(&archive_path_clone)?;
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(&extract_dir_clone)?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;

        // 4. Read update.json
        self.broadcast_progress(task_id, "running", 55, 100, Some("Validating update..."), None);

        let manifest_path = format!("{}/update.json", extract_dir);
        let manifest_str = fs::read_to_string(&manifest_path).await
            .map_err(|_| anyhow!("update.json not found in archive"))?;
        let manifest: UpdateManifest = serde_json::from_str(&manifest_str)?;

        // Check min_version
        if let Some(min_ver) = &manifest.min_version {
            let current = Self::current_version();
            if is_newer_version(&current, min_ver) {
                return Err(anyhow!(
                    "Current version {} is below minimum required version {}",
                    current,
                    min_ver
                ));
            }
        }

        // 5. Backup DB
        self.broadcast_progress(task_id, "running", 60, 100, Some("Backing up database..."), None);

        let db_path = format!("{}/pinas.db", self.data_dir);
        let backup_path = format!("{}/pinas.db.bak-{}", self.data_dir, Self::current_version());
        if fs::metadata(&db_path).await.is_ok() {
            fs::copy(&db_path, &backup_path).await?;
        }

        // Update changelog in DB
        let changelog_json = serde_json::to_string(&manifest.changelog)?;
        sqlx::query("UPDATE system_updates SET changelog = ? WHERE id = ?")
            .bind(&changelog_json)
            .bind(task_id)
            .execute(&self.db)
            .await?;

        // 6. Handle system update (major)
        if manifest.contents.system {
            self.broadcast_progress(task_id, "running", 65, 100, Some("Preparing system update..."), None);

            let system_dir = format!("{}/system", extract_dir);
            let update_dir = "/storage/.update";
            fs::create_dir_all(update_dir).await?;

            // Copy SYSTEM and KERNEL to /storage/.update/
            let system_file = format!("{}/SYSTEM", system_dir);
            if fs::metadata(&system_file).await.is_ok() {
                fs::copy(&system_file, format!("{}/SYSTEM", update_dir)).await?;
            }
            let kernel_file = format!("{}/KERNEL", system_dir);
            if fs::metadata(&kernel_file).await.is_ok() {
                fs::copy(&kernel_file, format!("{}/KERNEL", update_dir)).await?;
            }

            // Mark as reboot_required
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE system_updates SET status = 'reboot_required', completed_at = ? WHERE id = ?")
                .bind(&now)
                .bind(task_id)
                .execute(&self.db)
                .await?;
        }

        // 7. Write the apply script and trigger self-restart
        self.broadcast_progress(task_id, "running", 75, 100, Some("Applying update..."), None);

        let mut script_lines = vec![
            "#!/bin/bash".to_string(),
            "set -e".to_string(),
            format!("EXTRACT_DIR={}", extract_dir),
            format!("DATA_DIR={}", self.data_dir),
            "sleep 1".to_string(),
            "systemctl stop pinas || true".to_string(),
        ];

        // Copy backend binary
        if manifest.contents.backend {
            let bin_src = format!("{}/pinas", extract_dir);
            let bin_dest = format!("{}/bin/pinas", self.data_dir);
            script_lines.push(format!("cp '{}' '{}' && chmod 755 '{}'", bin_src, bin_dest, bin_dest));
        }

        // Copy frontend
        if manifest.contents.frontend {
            let www_src = format!("{}/www/.", extract_dir);
            let www_dest = format!("{}/www/", self.data_dir);
            script_lines.push(format!("cp -r '{}' '{}'", www_src, www_dest));
        }

        // Copy migrations
        if manifest.contents.migrations {
            let mig_src = format!("{}/migrations/.", extract_dir);
            // Migrations are embedded in the binary at compile time, but for
            // hot updates we may need to copy new migration files
            let mig_dest = format!("{}/migrations/", self.data_dir);
            script_lines.push(format!("mkdir -p '{}' && cp -r '{}' '{}' 2>/dev/null || true", mig_dest, mig_src, mig_dest));
        }

        // Copy scripts
        if manifest.contents.scripts {
            let scripts_src = format!("{}/scripts/.", extract_dir);
            let scripts_dest = format!("{}/bin/", self.data_dir);
            script_lines.push(format!("cp -r '{}' '{}' 2>/dev/null || true", scripts_src, scripts_dest));
            script_lines.push(format!("chmod +x {}/bin/*.sh 2>/dev/null || true", self.data_dir));
        }

        // Copy services
        if manifest.contents.services {
            let svc_src = format!("{}/services/.", extract_dir);
            script_lines.push(format!("cp -r '{}' '/storage/.config/system.d/' 2>/dev/null || true", svc_src));
            script_lines.push("systemctl daemon-reload".to_string());
        }

        // Write the update-applied flag
        let info = UpdateAppliedInfo {
            version: latest.clone(),
            previous_version: Self::current_version(),
            changelog: manifest.changelog.clone(),
        };
        let flag_json = serde_json::to_string(&info)?;
        let flag_path = format!("{}/data/.update-applied", self.data_dir);
        script_lines.push(format!("mkdir -p '{}/data'", self.data_dir));
        script_lines.push(format!("cat > '{}' << 'ENDFLAG'\n{}\nENDFLAG", flag_path, flag_json));

        // Restart pinas and cleanup
        script_lines.push("systemctl start pinas".to_string());
        script_lines.push(format!("rm -rf '{}' /tmp/pinas-apply-update.sh", extract_dir));

        let script_content = script_lines.join("\n");
        let script_path = "/tmp/pinas-apply-update.sh";
        fs::write(script_path, &script_content).await?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            fs::set_permissions(script_path, perms).await?;
        }

        if !manifest.contents.system {
            // For non-system updates: complete the record now
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE system_updates SET status = 'applying', completed_at = ? WHERE id = ?")
                .bind(&now)
                .bind(task_id)
                .execute(&self.db)
                .await?;
        }

        self.broadcast_progress(task_id, "running", 90, 100, Some("Restarting service..."), None);

        // Launch the apply script in background (nohup)
        tokio::process::Command::new("nohup")
            .arg("bash")
            .arg(script_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        // The service will be restarted by the script.
        // On next startup, the backend will detect .update-applied
        Ok(())
    }

    /// Get update status from DB
    pub async fn get_update_status(&self, task_id: &str) -> Result<Option<UpdateHistoryEntry>> {
        let entry = sqlx::query_as::<_, UpdateHistoryEntry>(
            r#"SELECT id, version, previous_version, update_type, status, changelog,
                      error_message, started_at, completed_at, created_at
               FROM system_updates WHERE id = ?"#,
        )
        .bind(task_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(entry)
    }

    /// Get update history
    pub async fn get_update_history(&self) -> Result<Vec<UpdateHistoryEntry>> {
        let entries = sqlx::query_as::<_, UpdateHistoryEntry>(
            r#"SELECT id, version, previous_version, update_type, status, changelog,
                      error_message, started_at, completed_at, created_at
               FROM system_updates
               ORDER BY created_at DESC
               LIMIT 20"#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(entries)
    }

    /// Check if an update was just applied (read the flag file)
    pub fn read_update_applied_flag(data_dir: &str) -> Option<UpdateAppliedInfo> {
        let flag_path = format!("{}/data/.update-applied", data_dir);
        match std::fs::read_to_string(&flag_path) {
            Ok(content) => {
                match serde_json::from_str::<UpdateAppliedInfo>(&content) {
                    Ok(info) => {
                        tracing::info!("Detected applied update: {} -> {}", info.previous_version, info.version);
                        Some(info)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse update-applied flag: {}", e);
                        None
                    }
                }
            }
            Err(_) => None,
        }
    }

    /// Remove the update-applied flag
    pub async fn dismiss_update_applied(data_dir: &str) -> Result<()> {
        let flag_path = format!("{}/data/.update-applied", data_dir);
        if fs::metadata(&flag_path).await.is_ok() {
            fs::remove_file(&flag_path).await?;
        }
        Ok(())
    }

    /// Broadcast a progress event via WebSocket
    fn broadcast_progress(
        &self,
        task_id: &str,
        status: &str,
        progress: i32,
        total: i32,
        current_step: Option<&str>,
        error_message: Option<&str>,
    ) {
        let progress_percent = if total == 0 { 0 } else { (progress * 100) / total };

        let event = TaskProgressEvent {
            task_id: task_id.to_string(),
            package_id: "system-update".to_string(),
            status: status.to_string(),
            progress,
            total_steps: total,
            progress_percent,
            current_step: current_step.map(|s| s.to_string()),
            error_message: error_message.map(|s| s.to_string()),
        };

        let _ = self.task_tx.send(event);
    }
}

/// Compare version strings (simple numeric comparison)
fn is_newer_version(current: &str, candidate: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let cur = parse(current);
    let cand = parse(candidate);

    for i in 0..std::cmp::max(cur.len(), cand.len()) {
        let c = cur.get(i).copied().unwrap_or(0);
        let n = cand.get(i).copied().unwrap_or(0);
        if n > c {
            return true;
        }
        if n < c {
            return false;
        }
    }
    false
}

/// Determine update type from version comparison
fn determine_update_type(current: &str, new: &str) -> String {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let cur = parse(current);
    let n = parse(new);

    let cur_major = cur.first().copied().unwrap_or(0);
    let new_major = n.first().copied().unwrap_or(0);

    if new_major > cur_major {
        "major".to_string()
    } else {
        let cur_minor = cur.get(1).copied().unwrap_or(0);
        let new_minor = n.get(1).copied().unwrap_or(0);
        if new_minor > cur_minor {
            "minor".to_string()
        } else {
            "patch".to_string()
        }
    }
}
