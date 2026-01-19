use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// App catalog index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub version: String,
    pub updated: String,
    pub apps: Vec<CatalogApp>,
}

/// App entry in catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub icon: Option<String>,
    pub manifest: String, // URL to manifest.json
}

/// Full package manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: HashMap<String, String>, // locale -> description
    pub author: Option<String>,
    pub license: Option<String>,
    pub website: Option<String>,
    pub icon: Option<String>,

    #[serde(default)]
    pub requirements: Requirements,

    pub install: InstallConfig,

    #[serde(default)]
    pub uninstall: UninstallConfig,

    #[serde(default)]
    pub files: HashMap<String, String>, // path -> base64 content

    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,

    /// Frontend configuration for the app window
    #[serde(default)]
    pub frontend: Option<FrontendConfig>,
}

/// Frontend configuration for app window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// MDI icon name (e.g., "mdi:docker")
    pub icon: String,
    /// Tailwind gradient classes (e.g., "from-blue-500 to-blue-600")
    pub gradient: String,
    /// Component name to render (must be pre-registered in frontend)
    pub component: String,
    /// Window configuration
    #[serde(default)]
    pub window: WindowConfig,
    /// Translations per locale
    #[serde(default)]
    pub i18n: HashMap<String, serde_json::Value>,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_min_width")]
    pub min_width: u32,
    #[serde(default = "default_min_height")]
    pub min_height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            min_width: default_min_width(),
            min_height: default_min_height(),
        }
    }
}

fn default_width() -> u32 { 900 }
fn default_height() -> u32 { 600 }
fn default_min_width() -> u32 { 600 }
fn default_min_height() -> u32 { 400 }

/// System requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requirements {
    #[serde(default)]
    pub min_ram: Option<u64>, // MB
    #[serde(default)]
    pub min_disk: Option<u64>, // MB
    #[serde(default)]
    pub arch: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>, // Package IDs
}

/// Installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    #[serde(rename = "type")]
    pub install_type: String, // "binary" or "docker"

    #[serde(default)]
    pub steps: Vec<InstallStep>,

    // Docker-specific
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub container: Option<ContainerConfig>,
}

/// Installation step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum InstallStep {
    Download {
        url: String,
        #[serde(default)]
        sha256: Option<String>,
        dest: String,
    },
    Extract {
        src: String,
        dest: String,
    },
    Copy {
        src: String,
        dest: String,
    },
    Symlink {
        src: String,
        dest: String,
    },
    Chmod {
        path: String,
        mode: String,
    },
    Mkdir {
        path: String,
    },
    Template {
        src: String, // Key in files map
        dest: String,
    },
    WriteFile {
        dest: String,
        content: String, // Base64 encoded
    },
    Exec {
        command: String,
        #[serde(default)]
        ignore_error: bool,
    },
    Delete {
        path: String,
    },
    DockerPull {
        image: String,
    },
    DockerCreate {
        config: ContainerConfig,
    },
    DockerStart {
        container: String,
    },
    DockerStop {
        container: String,
    },
    DockerRm {
        container: String,
    },
}

/// Uninstall configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UninstallConfig {
    #[serde(default)]
    pub steps: Vec<InstallStep>,
}

/// Docker container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub name: String,
    #[serde(default)]
    pub hostname: Option<String>,
    pub image: Option<String>,
    #[serde(default)]
    pub restart: Option<String>, // "no", "always", "unless-stopped", "on-failure"
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    #[serde(default)]
    pub volumes: Vec<VolumeMapping>,
    #[serde(default)]
    pub environment: Vec<EnvVar>,
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub privileged: bool,
}

/// Port mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "tcp".to_string()
}

/// Volume mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host: String,
    pub container: String,
    #[serde(default)]
    pub readonly: bool,
}

/// Environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}
