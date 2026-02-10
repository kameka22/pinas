use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

use crate::config::AppConfig;

#[derive(Debug, Error)]
pub enum HomeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Home directory already exists")]
    AlreadyExists,
    #[error("Home directory not found")]
    NotFound,
    #[error("Invalid username")]
    InvalidUsername,
}

/// Service for managing user home directories
pub struct HomeService {
    homes_root: PathBuf,
    delete_policy: String,
}

impl HomeService {
    /// Create a new HomeService from application config
    pub fn new(config: &AppConfig) -> Self {
        Self {
            homes_root: PathBuf::from(&config.homes_root),
            delete_policy: config.home_on_delete.clone(),
        }
    }

    /// Get the path to a user's home directory
    pub fn get_home_path(&self, username: &str) -> PathBuf {
        self.homes_root.join(username)
    }

    /// Check if a user's home directory exists
    pub async fn home_exists(&self, username: &str) -> bool {
        self.get_home_path(username).exists()
    }

    /// Create a home directory for a user with default subdirectories
    pub async fn create_home(&self, username: &str) -> Result<PathBuf, HomeError> {
        // Validate username (no path separators, not empty, not hidden)
        if username.is_empty()
            || username.contains('/')
            || username.contains('\\')
            || username.starts_with('.')
        {
            return Err(HomeError::InvalidUsername);
        }

        let home_path = self.get_home_path(username);

        // Check if already exists
        if home_path.exists() {
            tracing::warn!("Home directory already exists for user: {}", username);
            return Ok(home_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = home_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create home directory
        fs::create_dir_all(&home_path).await?;

        // Create default subdirectories
        let subdirs = ["Documents", "Downloads", "Photos", "Music", "Videos"];
        for subdir in subdirs {
            let subdir_path = home_path.join(subdir);
            if let Err(e) = fs::create_dir_all(&subdir_path).await {
                tracing::warn!("Failed to create subdirectory {}: {}", subdir, e);
            }
        }

        tracing::info!("Created home directory for user: {} at {:?}", username, home_path);
        Ok(home_path)
    }

    /// Handle home directory when a user is deleted
    /// Policy can be: "archive", "delete", or "keep"
    pub async fn handle_user_deletion(&self, username: &str) -> Result<(), HomeError> {
        let home_path = self.get_home_path(username);

        if !home_path.exists() {
            tracing::debug!("No home directory to handle for deleted user: {}", username);
            return Ok(());
        }

        match self.delete_policy.as_str() {
            "archive" => {
                // Rename to {username}_deleted_{timestamp}
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let archive_name = format!("{}_deleted_{}", username, timestamp);
                let archive_path = self.homes_root.join(archive_name);

                fs::rename(&home_path, &archive_path).await?;
                tracing::info!(
                    "Archived home directory for deleted user: {} -> {:?}",
                    username,
                    archive_path
                );
            }
            "delete" => {
                // Permanently delete
                fs::remove_dir_all(&home_path).await?;
                tracing::info!("Deleted home directory for user: {}", username);
            }
            "keep" => {
                // Leave as-is (orphaned)
                tracing::info!(
                    "Keeping orphaned home directory for deleted user: {} at {:?}",
                    username,
                    home_path
                );
            }
            _ => {
                // Unknown policy, default to archive
                tracing::warn!(
                    "Unknown home_on_delete policy: {}, defaulting to archive",
                    self.delete_policy
                );
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let archive_name = format!("{}_deleted_{}", username, timestamp);
                let archive_path = self.homes_root.join(archive_name);
                fs::rename(&home_path, &archive_path).await?;
            }
        }

        Ok(())
    }

    /// Get the homes root directory
    pub fn get_homes_root(&self) -> &PathBuf {
        &self.homes_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_config(homes_root: &str, policy: &str) -> AppConfig {
        AppConfig {
            bind_address: "0.0.0.0:3000".to_string(),
            database_url: "sqlite::memory:".to_string(),
            jwt_secret: "test".to_string(),
            jwt_expiration_hours: 24,
            files_root: "./data/files".to_string(),
            homes_root: homes_root.to_string(),
            home_on_delete: policy.to_string(),
            static_dir: None,
            dev_mode: true,
            kodi_username: "kodi".to_string(),
            kodi_password: "test".to_string(),
            tls_enabled: false,
            tls_cert_path: std::path::PathBuf::new(),
            tls_key_path: std::path::PathBuf::new(),
        }
    }

    #[tokio::test]
    async fn test_create_home() {
        let temp = tempdir().unwrap();
        let config = test_config(temp.path().to_str().unwrap(), "archive");
        let service = HomeService::new(&config);

        let result = service.create_home("testuser").await;
        assert!(result.is_ok());

        let home_path = result.unwrap();
        assert!(home_path.exists());
        assert!(home_path.join("Documents").exists());
        assert!(home_path.join("Downloads").exists());
    }

    #[tokio::test]
    async fn test_invalid_username() {
        let temp = tempdir().unwrap();
        let config = test_config(temp.path().to_str().unwrap(), "archive");
        let service = HomeService::new(&config);

        assert!(service.create_home("").await.is_err());
        assert!(service.create_home("../escape").await.is_err());
        assert!(service.create_home(".hidden").await.is_err());
    }
}
