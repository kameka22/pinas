use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Package type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Binary,
    Docker,
    Service,
}

impl std::fmt::Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageType::Binary => write!(f, "binary"),
            PackageType::Docker => write!(f, "docker"),
            PackageType::Service => write!(f, "service"),
        }
    }
}

/// Package status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum PackageStatus {
    Installing,
    Installed,
    Updating,
    Removing,
    Error,
}

impl std::fmt::Display for PackageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageStatus::Installing => write!(f, "installing"),
            PackageStatus::Installed => write!(f, "installed"),
            PackageStatus::Updating => write!(f, "updating"),
            PackageStatus::Removing => write!(f, "removing"),
            PackageStatus::Error => write!(f, "error"),
        }
    }
}

/// Installed package record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InstalledPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub package_type: String,
    pub manifest_url: Option<String>,
    pub manifest_data: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub installed_at: String,
    pub updated_at: String,
    pub frontend_config: Option<String>, // JSON FrontendConfig
    pub has_window: bool,
}

/// Package file record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub id: i64,
    pub package_id: String,
    pub path: String,
    pub file_type: String,
    pub created_at: String,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Package task for tracking installation progress
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageTask {
    pub id: String,
    pub package_id: String,
    pub task_type: String,
    pub status: String,
    pub progress: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// App translation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTranslation {
    pub id: i64,
    pub package_id: String,
    pub locale: String,
    pub translations: String, // JSON
    pub created_at: String,
    pub updated_at: String,
}

/// App registry entry for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistryEntry {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub gradient: String,
    pub component: String,
    pub window: WindowConfigResponse,
}

/// Window configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfigResponse {
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
}
