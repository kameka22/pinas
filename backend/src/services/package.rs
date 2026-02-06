use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::api::ws::TaskProgressEvent;
use crate::models::manifest::{InstallStep, PackageManifest};
use crate::models::package::{InstalledPackage, PackageTask};
use crate::services::docker::DockerService;

/// Package service handles installation, updates, and removal of packages
pub struct PackageService {
    db: SqlitePool,
    catalog_url: String,
    data_dir: String,
    packages_dir: String,
    downloads_dir: String,
    bin_dir: String,
    docker_service: DockerService,
    dev_mode: bool,
    task_tx: broadcast::Sender<TaskProgressEvent>,
}

impl PackageService {
    pub async fn new(db: SqlitePool, task_tx: broadcast::Sender<TaskProgressEvent>) -> Self {
        let data_dir = std::env::var("PINAS_DATA_DIR")
            .unwrap_or_else(|_| "/storage/.pinas".to_string());

        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::info!("PackageService running in dev mode - fake installation with simulated steps");
        }

        Self {
            db,
            catalog_url: std::env::var("PINAS_CATALOG_URL")
                .unwrap_or_else(|_| "https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/catalog.json".to_string()),
            data_dir: data_dir.clone(),
            packages_dir: std::env::var("PINAS_PACKAGES_DIR")
                .unwrap_or_else(|_| format!("{}/apps", data_dir)),
            downloads_dir: std::env::var("PINAS_DOWNLOADS_DIR")
                .unwrap_or_else(|_| format!("{}/downloads", data_dir)),
            bin_dir: std::env::var("PINAS_BIN_DIR")
                .unwrap_or_else(|_| format!("{}/bin", data_dir)),
            docker_service: DockerService::new().await,
            dev_mode,
            task_tx,
        }
    }

    /// Get variable substitutions for manifest paths
    fn get_substitutions(&self) -> HashMap<String, String> {
        let arch = std::env::consts::ARCH;
        let docker_arch = match arch {
            "aarch64" => "aarch64",
            "x86_64" => "x86_64",
            _ => arch,
        };

        let mut vars = HashMap::new();
        vars.insert("DATA_DIR".to_string(), self.data_dir.clone());
        vars.insert("PACKAGES_DIR".to_string(), self.packages_dir.clone());
        vars.insert("DOWNLOADS_DIR".to_string(), self.downloads_dir.clone());
        vars.insert("BIN_DIR".to_string(), self.bin_dir.clone());
        vars.insert("ARCH".to_string(), docker_arch.to_string());
        vars
    }

    /// Substitute variables in a string
    fn substitute_vars(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in self.get_substitutions() {
            result = result.replace(&format!("${{{}}}", key), &value);
        }
        result
    }

    /// Substitute variables in an InstallStep
    fn substitute_step(&self, step: &InstallStep) -> InstallStep {
        match step {
            InstallStep::Download { url, sha256, sha256_aarch64, sha256_x86_64, dest } => {
                // Select the appropriate SHA256 based on current architecture
                let arch = std::env::consts::ARCH;
                let selected_sha256 = match arch {
                    "aarch64" => sha256_aarch64.clone().or_else(|| sha256.clone()),
                    "x86_64" => sha256_x86_64.clone().or_else(|| sha256.clone()),
                    _ => sha256.clone(),
                };
                InstallStep::Download {
                    url: self.substitute_vars(url),
                    sha256: selected_sha256,
                    sha256_aarch64: None, // Clear arch-specific after selection
                    sha256_x86_64: None,
                    dest: self.substitute_vars(dest),
                }
            }
            InstallStep::Extract { src, dest } => InstallStep::Extract {
                src: self.substitute_vars(src),
                dest: self.substitute_vars(dest),
            },
            InstallStep::Copy { src, dest } => InstallStep::Copy {
                src: self.substitute_vars(src),
                dest: self.substitute_vars(dest),
            },
            InstallStep::Symlink { src, dest } => InstallStep::Symlink {
                src: self.substitute_vars(src),
                dest: self.substitute_vars(dest),
            },
            InstallStep::Chmod { path, mode } => InstallStep::Chmod {
                path: self.substitute_vars(path),
                mode: mode.clone(),
            },
            InstallStep::Mkdir { path } => InstallStep::Mkdir {
                path: self.substitute_vars(path),
            },
            InstallStep::Template { src, dest } => InstallStep::Template {
                src: src.clone(),
                dest: self.substitute_vars(dest),
            },
            InstallStep::WriteFile { dest, content } => InstallStep::WriteFile {
                dest: self.substitute_vars(dest),
                content: content.clone(),
            },
            InstallStep::Exec { command, ignore_error } => InstallStep::Exec {
                command: self.substitute_vars(command),
                ignore_error: *ignore_error,
            },
            InstallStep::Delete { path } => InstallStep::Delete {
                path: self.substitute_vars(path),
            },
            InstallStep::DockerPull { image } => InstallStep::DockerPull {
                image: self.substitute_vars(image),
            },
            InstallStep::DockerCreate { config } => {
                let mut new_config = config.clone();
                // Substitute volume paths
                for vol in &mut new_config.volumes {
                    vol.host = self.substitute_vars(&vol.host);
                }
                InstallStep::DockerCreate { config: new_config }
            },
            InstallStep::DockerStart { container } => InstallStep::DockerStart {
                container: self.substitute_vars(container),
            },
            InstallStep::DockerStop { container } => InstallStep::DockerStop {
                container: self.substitute_vars(container),
            },
            InstallStep::DockerRm { container } => InstallStep::DockerRm {
                container: self.substitute_vars(container),
            },
        }
    }

    /// Ensure required directories exist
    pub async fn init_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.packages_dir).await?;
        fs::create_dir_all(&self.downloads_dir).await?;
        fs::create_dir_all(&self.bin_dir).await?;
        Ok(())
    }

    /// Get Docker service, trying to reconnect if not available
    async fn get_docker_service(&self) -> Result<DockerService> {
        if self.docker_service.is_available() {
            return Ok(DockerService::new().await);
        }
        // Try to create a new connection (useful after Docker was just installed)
        let mut docker = DockerService::new().await;
        if docker.is_available() {
            return Ok(docker);
        }
        // Wait up to 30 seconds for Docker to become available
        if docker.wait_for_docker(30).await {
            return Ok(docker);
        }
        Err(anyhow!("Docker is not available"))
    }

    /// List all installed packages
    pub async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let packages = sqlx::query_as::<_, InstalledPackage>(
            r#"SELECT id, name, version, package_type, manifest_url, manifest_data,
                      status, error_message, installed_at, updated_at,
                      frontend_config, has_window
               FROM installed_packages
               ORDER BY name"#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(packages)
    }

    /// Get a specific installed package
    pub async fn get_installed(&self, package_id: &str) -> Result<Option<InstalledPackage>> {
        let package = sqlx::query_as::<_, InstalledPackage>(
            r#"SELECT id, name, version, package_type, manifest_url, manifest_data,
                      status, error_message, installed_at, updated_at,
                      frontend_config, has_window
               FROM installed_packages WHERE id = ?"#
        )
        .bind(package_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(package)
    }

    /// Check if a package is installed
    pub async fn is_installed(&self, package_id: &str) -> Result<bool> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM installed_packages WHERE id = ? AND status = 'installed'"
        )
        .bind(package_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count > 0)
    }

    /// Clean up any failed or incomplete installation
    async fn cleanup_failed_installation(&self, package_id: &str) -> Result<()> {
        // Check if there's a failed/installing record
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM installed_packages WHERE id = ?"
        )
        .bind(package_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(status) = status {
            if status == "error" || status == "installing" {
                tracing::info!("Cleaning up failed installation for package: {} (status: {})", package_id, status);

                // Delete related records
                sqlx::query("DELETE FROM package_files WHERE package_id = ?")
                    .bind(package_id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("DELETE FROM docker_containers WHERE package_id = ?")
                    .bind(package_id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("DELETE FROM app_translations WHERE package_id = ?")
                    .bind(package_id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("DELETE FROM package_tasks WHERE package_id = ?")
                    .bind(package_id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("DELETE FROM installed_packages WHERE id = ?")
                    .bind(package_id)
                    .execute(&self.db)
                    .await?;

                tracing::info!("Cleanup completed for package: {}", package_id);
            }
        }

        Ok(())
    }

    /// Install a package from manifest
    pub async fn install(&self, manifest: &PackageManifest, manifest_url: Option<&str>) -> Result<String> {
        // Check if already installed successfully
        if self.is_installed(&manifest.id).await? {
            return Err(anyhow!("Package {} is already installed", manifest.id));
        }

        // Clean up any failed/incomplete previous installation
        self.cleanup_failed_installation(&manifest.id).await?;

        // Check dependencies
        for dep in &manifest.requirements.dependencies {
            if !self.is_installed(dep).await? {
                return Err(anyhow!("Missing dependency: {}", dep));
            }
        }

        // Create task for progress tracking
        let task_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let total_steps = manifest.install.steps.len() as i32;

        sqlx::query(
            r#"INSERT INTO package_tasks (id, package_id, task_type, status, progress, total_steps, created_at, started_at)
               VALUES (?, ?, 'install', 'running', 0, ?, ?, ?)"#
        )
        .bind(&task_id)
        .bind(&manifest.id)
        .bind(total_steps)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        // Prepare frontend config
        let frontend_config_json = manifest.frontend.as_ref()
            .map(|fc| serde_json::to_string(fc))
            .transpose()?;
        let has_window = manifest.frontend.is_some();

        // Create package record
        let manifest_json = serde_json::to_string(manifest)?;
        sqlx::query(
            r#"INSERT INTO installed_packages (id, name, version, package_type, manifest_url, manifest_data, status, installed_at, updated_at, frontend_config, has_window)
               VALUES (?, ?, ?, ?, ?, ?, 'installing', ?, ?, ?, ?)"#
        )
        .bind(&manifest.id)
        .bind(&manifest.name)
        .bind(&manifest.version)
        .bind(&manifest.install.install_type)
        .bind(manifest_url)
        .bind(&manifest_json)
        .bind(&now)
        .bind(&now)
        .bind(&frontend_config_json)
        .bind(has_window)
        .execute(&self.db)
        .await?;

        // Execute installation steps (simulated in dev mode)
        let result = if self.dev_mode {
            self.execute_dev_mode_steps(&manifest.id, &task_id).await
        } else {
            self.execute_install_steps(manifest, &task_id).await
        };

        // Update status based on result
        match result {
            Ok(_) => {
                let now = chrono::Utc::now().to_rfc3339();
                sqlx::query("UPDATE installed_packages SET status = 'installed', updated_at = ? WHERE id = ?")
                    .bind(&now)
                    .bind(&manifest.id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("UPDATE package_tasks SET status = 'completed', completed_at = ? WHERE id = ?")
                    .bind(&now)
                    .bind(&task_id)
                    .execute(&self.db)
                    .await?;

                // Store translations if frontend config has i18n
                if let Some(frontend) = &manifest.frontend {
                    for (locale, translations) in &frontend.i18n {
                        let trans_json = serde_json::to_string(translations)?;
                        sqlx::query(
                            r#"INSERT OR REPLACE INTO app_translations (package_id, locale, translations, created_at, updated_at)
                               VALUES (?, ?, ?, ?, ?)"#
                        )
                        .bind(&manifest.id)
                        .bind(locale)
                        .bind(&trans_json)
                        .bind(&now)
                        .bind(&now)
                        .execute(&self.db)
                        .await?;
                    }
                }
            }
            Err(ref e) => {
                let now = chrono::Utc::now().to_rfc3339();
                let error_msg = e.to_string();
                sqlx::query("UPDATE installed_packages SET status = 'error', error_message = ?, updated_at = ? WHERE id = ?")
                    .bind(&error_msg)
                    .bind(&now)
                    .bind(&manifest.id)
                    .execute(&self.db)
                    .await?;

                sqlx::query("UPDATE package_tasks SET status = 'failed', error_message = ?, completed_at = ? WHERE id = ?")
                    .bind(&error_msg)
                    .bind(&now)
                    .bind(&task_id)
                    .execute(&self.db)
                    .await?;
            }
        }

        result?;
        Ok(task_id)
    }

    /// Broadcast a task progress event via WebSocket
    fn broadcast_progress(
        &self,
        task_id: &str,
        package_id: &str,
        status: &str,
        progress: i32,
        total_steps: i32,
        current_step: Option<&str>,
    ) {
        let progress_percent = if total_steps == 0 {
            0
        } else {
            ((progress as f32 / total_steps as f32) * 100.0) as i32
        };

        let event = TaskProgressEvent {
            task_id: task_id.to_string(),
            package_id: package_id.to_string(),
            status: status.to_string(),
            progress,
            total_steps,
            progress_percent,
            current_step: current_step.map(|s| s.to_string()),
            error_message: None,
        };

        // Ignore send errors (no subscribers)
        let _ = self.task_tx.send(event);
    }

    /// Get a human-readable description for an install step
    fn step_description(step: &InstallStep) -> String {
        match step {
            InstallStep::Download { url, .. } => {
                let filename = url.rsplit('/').next().unwrap_or("file");
                format!("Downloading {}", filename)
            }
            InstallStep::Extract { src, .. } => {
                let filename = src.rsplit('/').next().unwrap_or("archive");
                format!("Extracting {}", filename)
            }
            InstallStep::Copy { dest, .. } => {
                let filename = dest.rsplit('/').next().unwrap_or("file");
                format!("Copying {}", filename)
            }
            InstallStep::Symlink { dest, .. } => {
                let filename = dest.rsplit('/').next().unwrap_or("link");
                format!("Creating link {}", filename)
            }
            InstallStep::Chmod { path, .. } => {
                let filename = path.rsplit('/').next().unwrap_or("file");
                format!("Setting permissions on {}", filename)
            }
            InstallStep::Mkdir { path } => {
                let dirname = path.rsplit('/').next().unwrap_or("directory");
                format!("Creating directory {}", dirname)
            }
            InstallStep::Template { dest, .. } | InstallStep::WriteFile { dest, .. } => {
                let filename = dest.rsplit('/').next().unwrap_or("file");
                format!("Writing {}", filename)
            }
            InstallStep::Exec { command, .. } => {
                // Show a simplified version of the command
                if command.contains("systemctl start") {
                    "Starting service".to_string()
                } else if command.contains("systemctl enable") {
                    "Enabling service".to_string()
                } else if command.contains("systemctl daemon-reload") {
                    "Reloading services".to_string()
                } else {
                    let short = if command.len() > 40 { &command[..40] } else { command };
                    format!("Running: {}", short)
                }
            }
            InstallStep::Delete { path } => {
                let filename = path.rsplit('/').next().unwrap_or("file");
                format!("Removing {}", filename)
            }
            InstallStep::DockerPull { image } => format!("Pulling image {}", image),
            InstallStep::DockerCreate { config } => format!("Creating container {}", config.name),
            InstallStep::DockerStart { container } => format!("Starting container {}", container),
            InstallStep::DockerStop { container } => format!("Stopping container {}", container),
            InstallStep::DockerRm { container } => format!("Removing container {}", container),
        }
    }

    /// Execute installation steps
    async fn execute_install_steps(&self, manifest: &PackageManifest, task_id: &str) -> Result<()> {
        let total = manifest.install.steps.len() as i32;

        for (i, step) in manifest.install.steps.iter().enumerate() {
            let substituted_step = self.substitute_step(step);
            let step_desc = Self::step_description(&substituted_step);
            tracing::info!("Executing step {}/{}: {}", i + 1, total, step_desc);

            // Update progress in DB
            sqlx::query("UPDATE package_tasks SET progress = ?, current_step = ? WHERE id = ?")
                .bind(i as i32)
                .bind(&step_desc)
                .bind(task_id)
                .execute(&self.db)
                .await?;

            // Broadcast progress via WebSocket
            self.broadcast_progress(task_id, &manifest.id, "running", i as i32, total, Some(&step_desc));

            self.execute_step(&substituted_step, manifest, &manifest.id).await
                .with_context(|| format!("Failed at step {}: {:?}", i + 1, substituted_step))?;
        }

        // Broadcast completion
        self.broadcast_progress(task_id, &manifest.id, "completed", total, total, None);

        Ok(())
    }

    /// Execute fake installation steps in dev mode
    async fn execute_dev_mode_steps(&self, package_id: &str, task_id: &str) -> Result<()> {
        let fake_steps = [
            ("Downloading package...", 2000),
            ("Extracting files...", 1500),
            ("Configuring...", 1000),
            ("Installing services...", 1500),
            ("Finalizing...", 1000),
        ];
        let total = fake_steps.len() as i32;

        // Update total_steps in DB
        sqlx::query("UPDATE package_tasks SET total_steps = ? WHERE id = ?")
            .bind(total)
            .bind(task_id)
            .execute(&self.db)
            .await?;

        for (i, (desc, delay_ms)) in fake_steps.iter().enumerate() {
            tracing::info!("[DEV MODE] Step {}/{}: {}", i + 1, total, desc);

            // Update DB
            sqlx::query("UPDATE package_tasks SET progress = ?, current_step = ? WHERE id = ?")
                .bind(i as i32)
                .bind(*desc)
                .bind(task_id)
                .execute(&self.db)
                .await?;

            // Broadcast via WebSocket
            self.broadcast_progress(task_id, package_id, "running", i as i32, total, Some(desc));

            // Simulate work
            tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
        }

        // Broadcast completion
        self.broadcast_progress(task_id, package_id, "completed", total, total, None);

        Ok(())
    }

    /// Execute a single installation step
    async fn execute_step(&self, step: &InstallStep, manifest: &PackageManifest, package_id: &str) -> Result<()> {
        match step {
            InstallStep::Download { url, sha256, dest, .. } => {
                self.download_file(url, dest, sha256.as_deref()).await?;
                // Track downloaded file
                self.track_file(package_id, dest, "data").await?;
            }
            InstallStep::Extract { src, dest } => {
                self.extract_archive(src, dest).await?;
                // Track extracted directory
                self.track_file(package_id, dest, "data").await?;
            }
            InstallStep::Copy { src, dest } => {
                fs::copy(src, dest).await?;
                // Track copied file
                self.track_file(package_id, dest, "data").await?;
            }
            InstallStep::Symlink { src, dest } => {
                // Remove existing symlink if present
                if fs::symlink_metadata(dest).await.is_ok() {
                    fs::remove_file(dest).await?;
                }
                tokio::fs::symlink(src, dest).await?;
                // Track symlink
                self.track_file(package_id, dest, "symlink").await?;
            }
            InstallStep::Chmod { path, mode } => {
                let mode_val = u32::from_str_radix(mode, 8)?;
                let mut perms = fs::metadata(path).await?.permissions();
                std::os::unix::fs::PermissionsExt::set_mode(&mut perms, mode_val);
                fs::set_permissions(path, perms).await?;
            }
            InstallStep::Mkdir { path } => {
                fs::create_dir_all(path).await?;
                // Track created directory
                self.track_file(package_id, path, "data").await?;
            }
            InstallStep::Template { src, dest } => {
                if let Some(content) = manifest.files.get(src) {
                    let decoded = base64_decode(content)?;
                    let parent = Path::new(dest).parent();
                    if let Some(p) = parent {
                        fs::create_dir_all(p).await?;
                    }
                    fs::write(dest, decoded).await?;
                    // Determine file type based on destination
                    let file_type = if dest.ends_with(".service") { "service" } else { "config" };
                    self.track_file(package_id, dest, file_type).await?;
                } else {
                    return Err(anyhow!("Template file not found in manifest: {}", src));
                }
            }
            InstallStep::WriteFile { dest, content } => {
                let decoded = base64_decode(content)?;
                let parent = Path::new(dest).parent();
                if let Some(p) = parent {
                    fs::create_dir_all(p).await?;
                }
                fs::write(dest, decoded).await?;
                // Track written file
                self.track_file(package_id, dest, "config").await?;
            }
            InstallStep::Exec { command, ignore_error } => {
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .status()
                    .await?;

                if !status.success() && !ignore_error {
                    return Err(anyhow!("Command failed: {}", command));
                }
            }
            InstallStep::Delete { path } => {
                if fs::metadata(path).await.is_ok() {
                    if fs::metadata(path).await?.is_dir() {
                        fs::remove_dir_all(path).await?;
                    } else {
                        fs::remove_file(path).await?;
                    }
                }
            }
            // Docker steps - get a fresh connection each time (allows reconnect after Docker install)
            InstallStep::DockerPull { image } => {
                tracing::info!("Pulling Docker image: {}", image);
                let docker = self.get_docker_service().await?;
                docker.pull_image(image).await?;
            }
            InstallStep::DockerCreate { config } => {
                tracing::info!("Creating Docker container: {}", config.name);
                let docker = self.get_docker_service().await?;
                let container_id = docker.create_container(config).await?;
                // Track the container in database
                let image = config.image.as_deref().unwrap_or("unknown");
                let config_json = serde_json::to_string(config).unwrap_or_default();
                self.track_container(package_id, &container_id, &config.name, image, &config_json).await?;
            }
            InstallStep::DockerStart { container } => {
                tracing::info!("Starting Docker container: {}", container);
                let docker = self.get_docker_service().await?;
                docker.start_container(container).await?;
                // Update container status
                self.update_container_status(container, "running").await?;
            }
            InstallStep::DockerStop { container } => {
                tracing::info!("Stopping Docker container: {}", container);
                let docker = self.get_docker_service().await;
                if let Ok(docker) = docker {
                    if let Err(e) = docker.stop_container(container).await {
                        tracing::warn!("Failed to stop container {}: {}", container, e);
                    }
                }
                // Update container status
                let _ = self.update_container_status(container, "stopped").await;
            }
            InstallStep::DockerRm { container } => {
                tracing::info!("Removing Docker container: {}", container);
                let docker = self.get_docker_service().await;
                if let Ok(docker) = docker {
                    if let Err(e) = docker.remove_container(container, true).await {
                        tracing::warn!("Failed to remove container {}: {}", container, e);
                    }
                }
                // Remove container from tracking
                let _ = self.untrack_container(container).await;
            }
        }

        Ok(())
    }

    /// Execute a single uninstall step (no tracking needed)
    async fn execute_uninstall_step(&self, step: &InstallStep) -> Result<()> {
        match step {
            InstallStep::Exec { command, ignore_error } => {
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .status()
                    .await?;

                if !status.success() && !ignore_error {
                    return Err(anyhow!("Command failed: {}", command));
                }
            }
            InstallStep::Delete { path } => {
                if fs::metadata(path).await.is_ok() {
                    if fs::metadata(path).await?.is_dir() {
                        fs::remove_dir_all(path).await?;
                    } else {
                        fs::remove_file(path).await?;
                    }
                }
            }
            InstallStep::DockerStop { container } => {
                tracing::info!("Stopping Docker container: {}", container);
                if let Ok(docker) = self.get_docker_service().await {
                    if let Err(e) = docker.stop_container(container).await {
                        tracing::warn!("Failed to stop container {}: {}", container, e);
                    }
                }
            }
            InstallStep::DockerRm { container } => {
                tracing::info!("Removing Docker container: {}", container);
                if let Ok(docker) = self.get_docker_service().await {
                    if let Err(e) = docker.remove_container(container, true).await {
                        tracing::warn!("Failed to remove container {}: {}", container, e);
                    }
                }
                // Also remove from tracking
                let _ = self.untrack_container(container).await;
            }
            _ => {
                tracing::warn!("Unhandled uninstall step type: {:?}", step);
            }
        }
        Ok(())
    }

    /// Download a file with optional SHA256 verification
    async fn download_file(&self, url: &str, dest: &str, sha256: Option<&str>) -> Result<()> {
        tracing::info!("Downloading {} to {}", url, dest);

        // Ensure parent directory exists
        if let Some(parent) = Path::new(dest).parent() {
            fs::create_dir_all(parent).await?;
        }

        // Download file
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(anyhow!("Download failed: HTTP {}", response.status()));
        }

        let bytes = response.bytes().await?;

        // Verify SHA256 if provided
        if let Some(expected_hash) = sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual_hash = hex::encode(hasher.finalize());

            if actual_hash != expected_hash.to_lowercase() {
                return Err(anyhow!(
                    "SHA256 mismatch: expected {}, got {}",
                    expected_hash,
                    actual_hash
                ));
            }
            tracing::info!("SHA256 verified: {}", actual_hash);
        }

        // Write to file
        let mut file = fs::File::create(dest).await?;
        file.write_all(&bytes).await?;

        tracing::info!("Downloaded {} bytes to {}", bytes.len(), dest);
        Ok(())
    }

    /// Extract a tar.gz archive
    async fn extract_archive(&self, src: &str, dest: &str) -> Result<()> {
        tracing::info!("Extracting {} to {}", src, dest);

        fs::create_dir_all(dest).await?;

        let src_path = src.to_string();
        let dest_path = dest.to_string();

        // Run extraction in blocking task
        tokio::task::spawn_blocking(move || {
            use flate2::read::GzDecoder;
            use std::fs::File;
            use tar::Archive;

            let file = File::open(&src_path)?;
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(&dest_path)?;

            Ok::<_, anyhow::Error>(())
        })
        .await??;

        tracing::info!("Extraction complete");
        Ok(())
    }

    /// Uninstall a package
    /// If delete_data is true, all tracked files including data directories will be deleted
    /// If delete_data is false, only config/service files will be deleted, data is preserved
    pub async fn uninstall(&self, package_id: &str, delete_data: bool) -> Result<()> {
        tracing::info!("Uninstalling package: {}, delete_data: {}", package_id, delete_data);

        let package = self.get_installed(package_id).await?
            .ok_or_else(|| anyhow!("Package not found: {}", package_id))?;

        // Parse manifest to get uninstall steps
        if let Some(manifest_data) = &package.manifest_data {
            let manifest: PackageManifest = serde_json::from_str(manifest_data)?;

            // Execute uninstall steps (pass empty package_id since we're uninstalling)
            for step in &manifest.uninstall.steps {
                // Apply variable substitution
                let substituted_step = self.substitute_step(step);
                if let Err(e) = self.execute_uninstall_step(&substituted_step).await {
                    tracing::warn!("Uninstall step failed (continuing): {}", e);
                }
            }
        }

        // Also stop and remove any tracked containers for this package
        let containers = self.get_package_containers(package_id).await?;
        for container_name in containers {
            tracing::info!("Cleaning up container: {}", container_name);
            if let Err(e) = self.docker_service.stop_container(&container_name).await {
                tracing::warn!("Failed to stop container {}: {}", container_name, e);
            }
            if let Err(e) = self.docker_service.remove_container(&container_name, true).await {
                tracing::warn!("Failed to remove container {}: {}", container_name, e);
            }
        }

        // Delete tracked files based on delete_data option
        // file_type can be: "data", "config", "service", "symlink"
        let files: Vec<(String, String)> = sqlx::query_as(
            "SELECT path, file_type FROM package_files WHERE package_id = ? ORDER BY id DESC"
        )
        .bind(package_id)
        .fetch_all(&self.db)
        .await?;

        for (file_path, file_type) in &files {
            // Skip data files if delete_data is false
            if !delete_data && file_type == "data" {
                tracing::info!("Preserving data file: {}", file_path);
                continue;
            }

            // Try to remove (could be file or directory)
            if let Ok(metadata) = fs::metadata(&file_path).await {
                if metadata.is_dir() {
                    if let Err(e) = fs::remove_dir_all(&file_path).await {
                        tracing::warn!("Failed to remove directory {}: {}", file_path, e);
                    } else {
                        tracing::info!("Removed directory: {}", file_path);
                    }
                } else {
                    if let Err(e) = fs::remove_file(&file_path).await {
                        tracing::warn!("Failed to remove file {}: {}", file_path, e);
                    } else {
                        tracing::info!("Removed file: {}", file_path);
                    }
                }
            }
        }

        // Remove from database (keep data entries if not deleting data for potential reinstall)
        if delete_data {
            sqlx::query("DELETE FROM package_files WHERE package_id = ?")
                .bind(package_id)
                .execute(&self.db)
                .await?;
        } else {
            sqlx::query("DELETE FROM package_files WHERE package_id = ? AND file_type != 'data'")
                .bind(package_id)
                .execute(&self.db)
                .await?;
        }

        sqlx::query("DELETE FROM app_translations WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.db)
            .await?;

        sqlx::query("DELETE FROM docker_containers WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.db)
            .await?;

        sqlx::query("DELETE FROM installed_packages WHERE id = ?")
            .bind(package_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get task status
    pub async fn get_task(&self, task_id: &str) -> Result<Option<PackageTask>> {
        let task = sqlx::query_as::<_, PackageTask>(
            r#"SELECT id, package_id, task_type, status, progress, total_steps,
                      current_step, error_message, started_at, completed_at, created_at
               FROM package_tasks WHERE id = ?"#
        )
        .bind(task_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(task)
    }

    /// Track a file created during installation
    pub async fn track_file(&self, package_id: &str, path: &str, file_type: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO package_files (package_id, path, file_type, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(package_id)
        .bind(path)
        .bind(file_type)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Track a Docker container created during installation
    pub async fn track_container(&self, package_id: &str, container_id: &str, name: &str, image: &str, config: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO docker_containers (id, package_id, name, image, status, config, created_at, updated_at)
               VALUES (?, ?, ?, ?, 'created', ?, ?, ?)"#
        )
        .bind(container_id)
        .bind(package_id)
        .bind(name)
        .bind(image)
        .bind(config)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Update container status in database
    pub async fn update_container_status(&self, container_name: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE docker_containers SET status = ?, updated_at = ? WHERE name = ?")
            .bind(status)
            .bind(&now)
            .bind(container_name)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Remove container tracking from database
    pub async fn untrack_container(&self, container_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM docker_containers WHERE name = ?")
            .bind(container_name)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Get containers for a package
    pub async fn get_package_containers(&self, package_id: &str) -> Result<Vec<String>> {
        let containers: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM docker_containers WHERE package_id = ?"
        )
        .bind(package_id)
        .fetch_all(&self.db)
        .await?;
        Ok(containers)
    }
}

/// Decode base64 string
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input).map_err(|e| anyhow!("Base64 decode error: {}", e))
}
