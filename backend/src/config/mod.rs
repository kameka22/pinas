use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Process-wide configuration, set once by `main` (see `AppConfig::init_global`).
static GLOBAL_CONFIG: OnceLock<AppConfig> = OnceLock::new();

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

    /// TLS enabled (auto-detected: true in production, false in dev_mode)
    #[serde(default)]
    pub tls_enabled: bool,

    /// Path to TLS certificate PEM file (auto-computed)
    #[serde(skip)]
    pub tls_cert_path: PathBuf,

    /// Path to TLS private key PEM file (auto-computed)
    #[serde(skip)]
    pub tls_key_path: PathBuf,

    /// Data directory: secrets, TLS material, package state, update staging.
    /// Defaults to the directory containing the SQLite database.
    #[serde(default)]
    pub data_dir: Option<String>,

    /// Where installed packages live (default: `{data_dir}/apps`)
    #[serde(default)]
    pub packages_dir: Option<String>,

    /// Package download cache (default: `{data_dir}/downloads`)
    #[serde(default)]
    pub downloads_dir: Option<String>,

    /// Symlinked binaries of installed packages (default: `{data_dir}/bin`)
    #[serde(default)]
    pub bin_dir: Option<String>,

    /// Mount root for storage pools
    #[serde(default = "default_pools_path")]
    pub pools_path: String,

    /// App catalog index URL
    #[serde(default = "default_catalog_url")]
    pub catalog_url: String,

    /// GitHub owner/repo hosting the system update releases
    #[serde(default = "default_github_owner")]
    pub github_owner: String,
    #[serde(default = "default_github_repo")]
    pub github_repo: String,

    /// Docker daemon socket (`unix:///var/run/docker.sock` when unset)
    #[serde(default)]
    pub docker_host: Option<String>,
}

fn default_pools_path() -> String {
    "/storage/pools".to_string()
}

fn default_catalog_url() -> String {
    "https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/catalog.json".to_string()
}

fn default_github_owner() -> String {
    "kameka22".to_string()
}

fn default_github_repo() -> String {
    "pinas".to_string()
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
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
            tls_enabled: false,
            tls_cert_path: PathBuf::new(),
            tls_key_path: PathBuf::new(),
            data_dir: None,
            packages_dir: None,
            downloads_dir: None,
            bin_dir: None,
            pools_path: default_pools_path(),
            catalog_url: default_catalog_url(),
            github_owner: default_github_owner(),
            github_repo: default_github_repo(),
            docker_host: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("PINAS"))
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize().unwrap_or_default();

        // Resolve data_dir once so every consumer sees the same absolute answer
        if app_config.data_dir.is_none() {
            app_config.data_dir = Some(Self::get_data_dir(&app_config));
        }

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

        // TLS: only enabled if explicitly set via PINAS_TLS_ENABLED=true (already
        // deserialized into `tls_enabled`; forced off in dev mode)
        if app_config.dev_mode && app_config.tls_enabled {
            app_config.tls_enabled = false;
            tracing::info!("TLS disabled (dev_mode)");
        }

        // Setup TLS cert/key paths and generate if needed
        if app_config.tls_enabled {
            let data_dir = app_config.data_dir();
            let tls_dir = Path::new(&data_dir).join(".tls");
            app_config.tls_cert_path = tls_dir.join("cert.pem");
            app_config.tls_key_path = tls_dir.join("key.pem");

            Self::load_or_generate_tls_cert(&app_config)?;
        }

        Ok(app_config)
    }

    /// Install this configuration as the process-wide instance. Must be called once,
    /// before any service is constructed.
    pub fn init_global(config: AppConfig) {
        if GLOBAL_CONFIG.set(config).is_err() {
            tracing::warn!("AppConfig::init_global called twice; keeping the first value");
        }
    }

    /// Process-wide configuration. Falls back to loading from the environment
    /// (tests, tools) when `init_global` has not been called.
    pub fn global() -> &'static AppConfig {
        GLOBAL_CONFIG.get_or_init(|| {
            AppConfig::load().unwrap_or_else(|e| {
                tracing::warn!("AppConfig::global: load failed ({}), using defaults", e);
                let mut cfg = AppConfig::default();
                cfg.data_dir = Some(Self::get_data_dir(&cfg));
                cfg
            })
        })
    }

    /// Data directory (secrets, TLS, packages, update staging)
    pub fn data_dir(&self) -> String {
        self.data_dir.clone().unwrap_or_else(|| Self::get_data_dir(self))
    }

    /// Where installed packages live
    pub fn packages_dir(&self) -> String {
        self.packages_dir.clone().unwrap_or_else(|| format!("{}/apps", self.data_dir()))
    }

    /// Package download cache
    pub fn downloads_dir(&self) -> String {
        self.downloads_dir.clone().unwrap_or_else(|| format!("{}/downloads", self.data_dir()))
    }

    /// Symlinked binaries of installed packages
    pub fn bin_dir(&self) -> String {
        self.bin_dir.clone().unwrap_or_else(|| format!("{}/bin", self.data_dir()))
    }

    /// Docker daemon endpoint
    pub fn docker_host(&self) -> String {
        self.docker_host.clone().unwrap_or_else(|| "unix:///var/run/docker.sock".to_string())
    }

    /// Load JWT secret from persistent file, or generate and save a new one.
    /// This ensures a unique secret per installation that survives restarts.
    fn load_or_generate_jwt_secret(config: &AppConfig) -> anyhow::Result<String> {
        let data_dir = config.data_dir();

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
        getrandom::fill(&mut bytes)
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
        let data_dir = config.data_dir();

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
        getrandom::fill(&mut bytes)
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

    /// Derive the data directory from the database URL
    /// (`sqlite:./data/pinas.db` -> `./data`) when `PINAS_DATA_DIR` is not set.
    fn get_data_dir(config: &AppConfig) -> String {
        if let Some(dir) = &config.data_dir {
            return dir.clone();
        }
        if let Some(path) = config.database_url.strip_prefix("sqlite:") {
            let path = path.split('?').next().unwrap_or(path);
            Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            ".".to_string()
        }
    }

    /// Load existing TLS certificate or generate a self-signed one
    fn load_or_generate_tls_cert(config: &AppConfig) -> anyhow::Result<()> {
        let cert_path = &config.tls_cert_path;
        let key_path = &config.tls_key_path;

        // Check if both files exist and are non-empty
        if cert_path.exists() && key_path.exists() {
            if std::fs::metadata(cert_path)?.len() > 0
                && std::fs::metadata(key_path)?.len() > 0
            {
                tracing::info!("TLS certificate loaded from {}", cert_path.display());
                return Ok(());
            }
        }

        tracing::info!("Generating self-signed TLS certificate...");

        // Ensure TLS directory exists
        if let Some(parent) = cert_path.parent() {
            std::fs::create_dir_all(parent)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
            }
        }

        // Get hostname for SAN
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "pinas".to_string());

        // Generate self-signed certificate using rcgen
        let mut params = rcgen::CertificateParams::new(vec![
            hostname.clone(),
            "localhost".to_string(),
        ])?;

        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            rcgen::DnValue::Utf8String(hostname),
        );

        // Add IP SANs
        params.subject_alt_names.push(
            rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        );

        // 10 year validity
        params.not_before = rcgen::date_time_ymd(2025, 1, 1);
        params.not_after = rcgen::date_time_ymd(2035, 1, 1);

        // Generate ECDSA P-256 key pair (algorithm is inferred by rcgen from key type)
        let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        let cert = params.self_signed(&key_pair)?;

        // Write certificate
        std::fs::write(cert_path, cert.pem())?;

        // Write private key with restrictive permissions
        std::fs::write(key_path, key_pair.serialize_pem())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        tracing::info!("Self-signed TLS certificate generated at {}", cert_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_is_derived_from_database_url() {
        let cfg = AppConfig {
            database_url: "sqlite:/var/lib/pinas/pinas.db?mode=rwc".to_string(),
            ..AppConfig::default()
        };
        assert_eq!(cfg.data_dir(), "/var/lib/pinas");
        assert_eq!(cfg.packages_dir(), "/var/lib/pinas/apps");
        assert_eq!(cfg.downloads_dir(), "/var/lib/pinas/downloads");
        assert_eq!(cfg.bin_dir(), "/var/lib/pinas/bin");
    }

    #[test]
    fn explicit_paths_win_over_derived_ones() {
        let cfg = AppConfig {
            data_dir: Some("/storage/.pinas/data".to_string()),
            packages_dir: Some("/storage/.pinas/packages".to_string()),
            ..AppConfig::default()
        };
        assert_eq!(cfg.data_dir(), "/storage/.pinas/data");
        assert_eq!(cfg.packages_dir(), "/storage/.pinas/packages");
        assert_eq!(cfg.bin_dir(), "/storage/.pinas/data/bin");
        assert_eq!(cfg.docker_host(), "unix:///var/run/docker.sock");
    }

    #[test]
    fn defaults_are_production_sensible() {
        let cfg = AppConfig::default();
        assert!(!cfg.dev_mode);
        assert!(!cfg.tls_enabled);
        assert_eq!(cfg.pools_path, "/storage/pools");
        assert_eq!(cfg.github_owner, "kameka22");
        assert!(cfg.catalog_url.starts_with("https://"));
    }
}
