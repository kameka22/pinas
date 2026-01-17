use serde::Deserialize;

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

    /// Root directory for file manager
    #[serde(default = "default_files_root")]
    pub files_root: String,
}

fn default_bind_address() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_database_url() -> String {
    "sqlite:./data/pinas.db?mode=rwc".to_string()
}

fn default_jwt_secret() -> String {
    "change-me-in-production".to_string()
}

fn default_jwt_expiration() -> u64 {
    24 // 24 hours
}

fn default_files_root() -> String {
    "./data/files".to_string()
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("PINAS"))
            .build()?;

        let app_config: AppConfig = config.try_deserialize().unwrap_or_else(|_| AppConfig {
            bind_address: default_bind_address(),
            database_url: default_database_url(),
            jwt_secret: default_jwt_secret(),
            jwt_expiration_hours: default_jwt_expiration(),
            files_root: default_files_root(),
        });

        Ok(app_config)
    }
}
