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

/// Package task for tracking installation progress (DB row)
#[derive(Debug, Clone, Deserialize, sqlx::FromRow)]
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

impl PackageTask {
    /// Compute progress percentage (0-100)
    pub fn progress_percent(&self) -> i32 {
        if self.total_steps == 0 {
            return 0;
        }
        if self.status == "completed" {
            return 100;
        }
        ((self.progress as f32 / self.total_steps as f32) * 100.0) as i32
    }
}

/// Custom Serialize to include computed progress_percent
impl Serialize for PackageTask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PackageTask", 12)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("package_id", &self.package_id)?;
        s.serialize_field("task_type", &self.task_type)?;
        s.serialize_field("status", &self.status)?;
        s.serialize_field("progress", &self.progress)?;
        s.serialize_field("total_steps", &self.total_steps)?;
        s.serialize_field("progress_percent", &self.progress_percent())?;
        s.serialize_field("current_step", &self.current_step)?;
        s.serialize_field("error_message", &self.error_message)?;
        s.serialize_field("started_at", &self.started_at)?;
        s.serialize_field("completed_at", &self.completed_at)?;
        s.serialize_field("created_at", &self.created_at)?;
        Ok(s.end()?)
    }
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
    /// Component-specific configuration (e.g., port, path for IframeApp)
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

/// Window configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfigResponse {
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
}
