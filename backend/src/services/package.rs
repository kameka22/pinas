use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

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
}

impl PackageService {
    pub async fn new(db: SqlitePool) -> Self {
        let data_dir = std::env::var("PINAS_DATA_DIR")
            .unwrap_or_else(|_| "/storage/.pinas".to_string());

        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::info!("PackageService running in dev mode - installation steps will be skipped");
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
            InstallStep::Download { url, sha256, dest } => InstallStep::Download {
                url: self.substitute_vars(url),
                sha256: sha256.clone(),
                dest: self.substitute_vars(dest),
            },
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

    /// Install a package from manifest
    pub async fn install(&self, manifest: &PackageManifest, manifest_url: Option<&str>) -> Result<String> {
        // Check if already installed
        if self.is_installed(&manifest.id).await? {
            return Err(anyhow!("Package {} is already installed", manifest.id));
        }

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

        // Execute installation steps
        let result = self.execute_install_steps(manifest, &task_id).await;

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

    /// Execute installation steps
    async fn execute_install_steps(&self, manifest: &PackageManifest, task_id: &str) -> Result<()> {
        for (i, step) in manifest.install.steps.iter().enumerate() {
            // Apply variable substitution
            let substituted_step = self.substitute_step(step);
            let step_desc = format!("{:?}", substituted_step);
            tracing::info!("Executing step {}/{}: {}", i + 1, manifest.install.steps.len(), step_desc);

            // Update progress
            sqlx::query("UPDATE package_tasks SET progress = ?, current_step = ? WHERE id = ?")
                .bind(i as i32)
                .bind(&step_desc)
                .bind(task_id)
                .execute(&self.db)
                .await?;

            self.execute_step(&substituted_step, manifest).await
                .with_context(|| format!("Failed at step {}: {:?}", i + 1, substituted_step))?;
        }

        Ok(())
    }

    /// Execute a single installation step
    async fn execute_step(&self, step: &InstallStep, manifest: &PackageManifest) -> Result<()> {
        match step {
            InstallStep::Download { url, sha256, dest } => {
                self.download_file(url, dest, sha256.as_deref()).await?;
            }
            InstallStep::Extract { src, dest } => {
                self.extract_archive(src, dest).await?;
            }
            InstallStep::Copy { src, dest } => {
                fs::copy(src, dest).await?;
            }
            InstallStep::Symlink { src, dest } => {
                // Remove existing symlink if present
                if fs::symlink_metadata(dest).await.is_ok() {
                    fs::remove_file(dest).await?;
                }
                tokio::fs::symlink(src, dest).await?;
            }
            InstallStep::Chmod { path, mode } => {
                let mode_val = u32::from_str_radix(mode, 8)?;
                let mut perms = fs::metadata(path).await?.permissions();
                std::os::unix::fs::PermissionsExt::set_mode(&mut perms, mode_val);
                fs::set_permissions(path, perms).await?;
            }
            InstallStep::Mkdir { path } => {
                fs::create_dir_all(path).await?;
            }
            InstallStep::Template { src, dest } => {
                if let Some(content) = manifest.files.get(src) {
                    let decoded = base64_decode(content)?;
                    let parent = Path::new(dest).parent();
                    if let Some(p) = parent {
                        fs::create_dir_all(p).await?;
                    }
                    fs::write(dest, decoded).await?;
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
            // Docker steps
            InstallStep::DockerPull { image } => {
                tracing::info!("Pulling Docker image: {}", image);
                self.docker_service.pull_image(image).await?;
            }
            InstallStep::DockerCreate { config } => {
                tracing::info!("Creating Docker container: {}", config.name);
                self.docker_service.create_container(config).await?;
            }
            InstallStep::DockerStart { container } => {
                tracing::info!("Starting Docker container: {}", container);
                self.docker_service.start_container(container).await?;
            }
            InstallStep::DockerStop { container } => {
                tracing::info!("Stopping Docker container: {}", container);
                if let Err(e) = self.docker_service.stop_container(container).await {
                    tracing::warn!("Failed to stop container {}: {}", container, e);
                }
            }
            InstallStep::DockerRm { container } => {
                tracing::info!("Removing Docker container: {}", container);
                if let Err(e) = self.docker_service.remove_container(container, true).await {
                    tracing::warn!("Failed to remove container {}: {}", container, e);
                }
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
    pub async fn uninstall(&self, package_id: &str) -> Result<()> {
        let package = self.get_installed(package_id).await?
            .ok_or_else(|| anyhow!("Package not found: {}", package_id))?;

        // Parse manifest to get uninstall steps
        if let Some(manifest_data) = &package.manifest_data {
            let manifest: PackageManifest = serde_json::from_str(manifest_data)?;

            // Execute uninstall steps
            for step in &manifest.uninstall.steps {
                if let Err(e) = self.execute_step(step, &manifest).await {
                    tracing::warn!("Uninstall step failed (continuing): {}", e);
                }
            }
        }

        // Delete tracked files
        let files: Vec<String> = sqlx::query_scalar(
            "SELECT path FROM package_files WHERE package_id = ? ORDER BY id DESC"
        )
        .bind(package_id)
        .fetch_all(&self.db)
        .await?;

        for file_path in files {
            if let Err(e) = fs::remove_file(&file_path).await {
                tracing::warn!("Failed to remove file {}: {}", file_path, e);
            }
        }

        // Remove from database
        sqlx::query("DELETE FROM package_files WHERE package_id = ?")
            .bind(package_id)
            .execute(&self.db)
            .await?;

        sqlx::query("DELETE FROM app_translations WHERE package_id = ?")
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
}

/// Decode base64 string
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input).map_err(|e| anyhow!("Base64 decode error: {}", e))
}
