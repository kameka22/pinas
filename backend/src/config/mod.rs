use serde::Deserialize;
use std::path::Path;

/// Marker value indicating no JWT secret was configured
const DEFAULT_JWT_SECRET_MARKER: &str = "change-me-in-production";

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server bind address (e.g., "0.0.0.0:3000")
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// Database URL (SQLite path)
    #[serde(default = "default_database_url")]
    pub database_url: String,

    /// JWT secret key
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,

    /// JWT token expiration in hours
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_hours: u64,

    /// Root directory for file manager (legacy, use homes_root instead)
    #[serde(default = "default_files_root")]
    pub files_root: String,

    /// Root directory for user home directories
    #[serde(default = "default_homes_root")]
    pub homes_root: String,

    /// Policy for handling home directory when user is deleted: "archive", "delete", "keep"
    #[serde(default = "default_home_on_delete")]
    pub home_on_delete: String,

    /// Directory for static frontend files (optional)
    #[serde(default)]
    pub static_dir: Option<String>,

    /// Development mode - skip actual installations (Docker, downloads, etc.)
    #[serde(default = "default_dev_mode")]
    pub dev_mode: bool,

    /// Kodi JSON-RPC username
    #[serde(default = "default_kodi_username")]
    pub kodi_username: String,

    /// Kodi JSON-RPC password
    #[serde(default = "default_kodi_password")]
    pub kodi_password: String,
}

fn default_bind_address() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_database_url() -> String {
    "sqlite:./data/pinas.db?mode=rwc".to_string()
}

fn default_jwt_secret() -> String {
    DEFAULT_JWT_SECRET_MARKER.to_string()
}

fn default_jwt_expiration() -> u64 {
    24 // 24 hours
}

fn default_files_root() -> String {
    "./data/files".to_string()
}

fn default_homes_root() -> String {
    "./data/homes".to_string()
}

fn default_home_on_delete() -> String {
    "archive".to_string()
}

fn default_dev_mode() -> bool {
    false
}

fn default_kodi_username() -> String {
    "kodi".to_string()
}

fn default_kodi_password() -> String {
    "auto-generate".to_string()
}

/// Marker value indicating no Kodi password was configured
const DEFAULT_KODI_PASSWORD_MARKER: &str = "auto-generate";

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("PINAS"))
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize().unwrap_or_else(|_| AppConfig {
            bind_address: default_bind_address(),
            database_url: default_database_url(),
            jwt_secret: default_jwt_secret(),
            jwt_expiration_hours: default_jwt_expiration(),
            files_root: default_files_root(),
            homes_root: default_homes_root(),
            home_on_delete: default_home_on_delete(),
            static_dir: None,
            dev_mode: default_dev_mode(),
            kodi_username: default_kodi_username(),
            kodi_password: default_kodi_password(),
        });

        // Auto-generate JWT secret if using default marker
        if app_config.jwt_secret == DEFAULT_JWT_SECRET_MARKER {
            app_config.jwt_secret = Self::load_or_generate_jwt_secret(&app_config)?;
        }

        // Auto-generate Kodi password if using default marker
        if app_config.kodi_password == DEFAULT_KODI_PASSWORD_MARKER {
            app_config.kodi_password = Self::load_or_generate_secret(
                &app_config,
                ".kodi_password",
                "Kodi password",
            )?;
        }

        Ok(app_config)
    }

    /// Load JWT secret from persistent file, or generate and save a new one.
    /// This ensures a unique secret per installation that survives restarts.
    fn load_or_generate_jwt_secret(config: &AppConfig) -> anyhow::Result<String> {
        // Determine data directory from database URL or env
        let data_dir = std::env::var("PINAS_DATA_DIR").unwrap_or_else(|_| {
            // Extract directory from database_url (sqlite:./data/pinas.db -> ./data)
            if let Some(path) = config.database_url.strip_prefix("sqlite:") {
                let path = path.split('?').next().unwrap_or(path);
                Path::new(path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string())
            } else {
                ".".to_string()
            }
        });

        let secret_path = Path::new(&data_dir).join(".jwt_secret");

        // Try to read existing secret
        if let Ok(existing) = std::fs::read_to_string(&secret_path) {
            let secret = existing.trim().to_string();
            if secret.len() >= 32 {
                tracing::info!("JWT secret loaded from {}", secret_path.display());
                return Ok(secret);
            }
        }

        // Generate new cryptographically secure secret (64 hex chars = 256 bits)
        use std::io::Write;
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes)
            .map_err(|e| anyhow::anyhow!("Failed to generate random JWT secret: {}", e))?;
        let secret = hex::encode(bytes);

        // Ensure directory exists
        if let Some(parent) = secret_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write secret with restrictive permissions
        let mut file = std::fs::File::create(&secret_path)?;
        file.write_all(secret.as_bytes())?;

        // Set file permissions to 600 (owner read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&secret_path, std::fs::Permissions::from_mode(0o600))?;
        }

        tracing::info!("Generated new JWT secret at {}", secret_path.display());
        Ok(secret)
    }

    /// Generic helper: load a secret from a file, or generate and persist a new one.
    fn load_or_generate_secret(config: &AppConfig, filename: &str, label: &str) -> anyhow::Result<String> {
        let data_dir = std::env::var("PINAS_DATA_DIR").unwrap_or_else(|_| {
            if let Some(path) = config.database_url.strip_prefix("sqlite:") {
                let path = path.split('?').next().unwrap_or(path);
                Path::new(path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string())
            } else {
                ".".to_string()
            }
        });

        let secret_path = Path::new(&data_dir).join(filename);

        // Try to read existing
        if let Ok(existing) = std::fs::read_to_string(&secret_path) {
            let secret = existing.trim().to_string();
            if !secret.is_empty() {
                tracing::info!("{} loaded from {}", label, secret_path.display());
                return Ok(secret);
            }
        }

        // Generate new (16 bytes = 32 hex chars)
        use std::io::Write;
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes)
            .map_err(|e| anyhow::anyhow!("Failed to generate random {}: {}", label, e))?;
        let secret = hex::encode(bytes);

        if let Some(parent) = secret_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(&secret_path)?;
        file.write_all(secret.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&secret_path, std::fs::Permissions::from_mode(0o600))?;
        }

        tracing::info!("Generated new {} at {}", label, secret_path.display());
        Ok(secret)
    }
}
