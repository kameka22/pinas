use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;

/// Kodi JSON-RPC service for controlling Kodi
pub struct KodiService {
    dev_mode: bool,
    // Fake state for dev mode
    fake_volume: AtomicU8,
    fake_playing: AtomicBool,
    fake_current_title: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KodiStatus {
    pub connected: bool,
    pub version: String,
    pub name: String,
    pub playing: bool,
    pub paused: bool,
    pub current_title: Option<String>,
    pub current_type: Option<String>, // movie, episode, music, etc.
    pub volume: u8,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KodiInfo {
    pub version: String,
    pub build: String,
    pub name: String,
    pub uptime: String,
    pub cpu_temp: Option<f32>,
    pub free_space: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSource {
    pub id: String,
    pub name: String,
    pub path: String,
    pub source_type: String, // video, music, pictures, files
    pub protocol: Option<String>, // smb, nfs, local
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KodiAddon {
    pub id: String,
    pub name: String,
    pub version: String,
    pub addon_type: String,
    pub enabled: bool,
    pub installed: bool,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KodiSetting {
    pub id: String,
    pub label: String,
    pub category: String,
    pub value: serde_json::Value,
    pub setting_type: String, // boolean, integer, string, list
    pub options: Option<Vec<SettingOption>>,
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    pub label: String,
    pub value: serde_json::Value,
}

impl KodiService {
    pub fn new(dev_mode: bool) -> Self {
        Self {
            dev_mode,
            fake_volume: AtomicU8::new(75),
            fake_playing: AtomicBool::new(false),
            fake_current_title: Mutex::new(None),
        }
    }

    /// Get Kodi connection status and basic info
    pub async fn get_status(&self) -> Result<KodiStatus, KodiError> {
        if self.dev_mode {
            return Ok(KodiStatus {
                connected: true,
                version: "21.0".to_string(),
                name: "LibreELEC".to_string(),
                playing: self.fake_playing.load(Ordering::Relaxed),
                paused: false,
                current_title: self.fake_current_title.lock().unwrap().clone(),
                current_type: if self.fake_playing.load(Ordering::Relaxed) {
                    Some("movie".to_string())
                } else {
                    None
                },
                volume: self.fake_volume.load(Ordering::Relaxed),
                muted: false,
            });
        }

        // Real implementation would use JSON-RPC
        self.json_rpc_get_status().await
    }

    /// Get detailed Kodi system info
    pub async fn get_info(&self) -> Result<KodiInfo, KodiError> {
        if self.dev_mode {
            return Ok(KodiInfo {
                version: "21.0 Omega".to_string(),
                build: "21.0-Omega".to_string(),
                name: "LibreELEC".to_string(),
                uptime: "2 days, 5:23:41".to_string(),
                cpu_temp: Some(52.0),
                free_space: "128.5 GB".to_string(),
            });
        }

        self.json_rpc_get_info().await
    }

    // === Playback Controls ===

    pub async fn play_pause(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            let current = self.fake_playing.load(Ordering::Relaxed);
            self.fake_playing.store(!current, Ordering::Relaxed);
            if !current {
                *self.fake_current_title.lock().unwrap() = Some("Big Buck Bunny (2008)".to_string());
            }
            tracing::info!("[DEV] Kodi play/pause toggled");
            return Ok(());
        }

        self.json_rpc_call("Player.PlayPause", serde_json::json!({ "playerid": 1 })).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            self.fake_playing.store(false, Ordering::Relaxed);
            *self.fake_current_title.lock().unwrap() = None;
            tracing::info!("[DEV] Kodi playback stopped");
            return Ok(());
        }

        self.json_rpc_call("Player.Stop", serde_json::json!({ "playerid": 1 })).await?;
        Ok(())
    }

    pub async fn get_volume(&self) -> Result<u8, KodiError> {
        if self.dev_mode {
            return Ok(self.fake_volume.load(Ordering::Relaxed));
        }

        let result = self.json_rpc_call(
            "Application.GetProperties",
            serde_json::json!({ "properties": ["volume"] })
        ).await?;

        Ok(result["volume"].as_u64().unwrap_or(50) as u8)
    }

    pub async fn set_volume(&self, volume: u8) -> Result<(), KodiError> {
        let volume = volume.min(100);

        if self.dev_mode {
            self.fake_volume.store(volume, Ordering::Relaxed);
            tracing::info!("[DEV] Kodi volume set to {}", volume);
            return Ok(());
        }

        self.json_rpc_call("Application.SetVolume", serde_json::json!({ "volume": volume })).await?;
        Ok(())
    }

    pub async fn input_action(&self, action: &str) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi input action: {}", action);
            return Ok(());
        }

        let method = match action {
            "up" => "Input.Up",
            "down" => "Input.Down",
            "left" => "Input.Left",
            "right" => "Input.Right",
            "select" => "Input.Select",
            "back" => "Input.Back",
            "home" => "Input.Home",
            "info" => "Input.Info",
            "context" => "Input.ContextMenu",
            _ => return Err(KodiError::InvalidAction(action.to_string())),
        };

        self.json_rpc_call(method, serde_json::json!({})).await?;
        Ok(())
    }

    // === Media Sources ===

    pub async fn get_sources(&self, source_type: Option<&str>) -> Result<Vec<MediaSource>, KodiError> {
        if self.dev_mode {
            let mut sources = vec![
                MediaSource {
                    id: "1".to_string(),
                    name: "Films".to_string(),
                    path: "/storage/movies".to_string(),
                    source_type: "video".to_string(),
                    protocol: Some("local".to_string()),
                },
                MediaSource {
                    id: "2".to_string(),
                    name: "Séries TV".to_string(),
                    path: "/storage/tvshows".to_string(),
                    source_type: "video".to_string(),
                    protocol: Some("local".to_string()),
                },
                MediaSource {
                    id: "3".to_string(),
                    name: "NAS Films".to_string(),
                    path: "smb://192.168.1.100/movies".to_string(),
                    source_type: "video".to_string(),
                    protocol: Some("smb".to_string()),
                },
                MediaSource {
                    id: "4".to_string(),
                    name: "NAS Music".to_string(),
                    path: "nfs://192.168.1.100/music".to_string(),
                    source_type: "music".to_string(),
                    protocol: Some("nfs".to_string()),
                },
                MediaSource {
                    id: "5".to_string(),
                    name: "Photos".to_string(),
                    path: "/storage/pictures".to_string(),
                    source_type: "pictures".to_string(),
                    protocol: Some("local".to_string()),
                },
            ];

            if let Some(st) = source_type {
                sources.retain(|s| s.source_type == st);
            }

            return Ok(sources);
        }

        // Real implementation would read from sources.xml
        self.read_sources_xml(source_type).await
    }

    pub async fn add_source(&self, source: &MediaSource) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Added Kodi source: {} -> {}", source.name, source.path);
            return Ok(());
        }

        self.write_source_to_xml(source).await
    }

    pub async fn remove_source(&self, source_id: &str) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Removed Kodi source: {}", source_id);
            return Ok(());
        }

        self.remove_source_from_xml(source_id).await
    }

    // === Settings ===

    pub async fn get_settings(&self, category: Option<&str>) -> Result<Vec<KodiSetting>, KodiError> {
        if self.dev_mode {
            let mut settings = vec![
                // Video settings
                KodiSetting {
                    id: "videoplayer.adjustrefreshrate".to_string(),
                    label: "Adjust display refresh rate".to_string(),
                    category: "video".to_string(),
                    value: serde_json::json!(2),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption { label: "Off".to_string(), value: serde_json::json!(0) },
                        SettingOption { label: "On start/stop".to_string(), value: serde_json::json!(1) },
                        SettingOption { label: "Always".to_string(), value: serde_json::json!(2) },
                    ]),
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "videoplayer.usedisplayasclock".to_string(),
                    label: "Sync playback to display".to_string(),
                    category: "video".to_string(),
                    value: serde_json::json!(true),
                    setting_type: "boolean".to_string(),
                    options: None,
                    min: None,
                    max: None,
                },
                // Audio settings
                KodiSetting {
                    id: "audiooutput.passthrough".to_string(),
                    label: "Allow passthrough".to_string(),
                    category: "audio".to_string(),
                    value: serde_json::json!(true),
                    setting_type: "boolean".to_string(),
                    options: None,
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "audiooutput.channels".to_string(),
                    label: "Number of channels".to_string(),
                    category: "audio".to_string(),
                    value: serde_json::json!(2),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption { label: "2.0".to_string(), value: serde_json::json!(2) },
                        SettingOption { label: "5.1".to_string(), value: serde_json::json!(6) },
                        SettingOption { label: "7.1".to_string(), value: serde_json::json!(8) },
                    ]),
                    min: None,
                    max: None,
                },
                // Interface settings
                KodiSetting {
                    id: "locale.language".to_string(),
                    label: "Language".to_string(),
                    category: "interface".to_string(),
                    value: serde_json::json!("resource.language.en_gb"),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption { label: "English".to_string(), value: serde_json::json!("resource.language.en_gb") },
                        SettingOption { label: "Français".to_string(), value: serde_json::json!("resource.language.fr_fr") },
                        SettingOption { label: "Deutsch".to_string(), value: serde_json::json!("resource.language.de_de") },
                    ]),
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "lookandfeel.skin".to_string(),
                    label: "Skin".to_string(),
                    category: "interface".to_string(),
                    value: serde_json::json!("skin.estuary"),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption { label: "Estuary".to_string(), value: serde_json::json!("skin.estuary") },
                        SettingOption { label: "Estouchy".to_string(), value: serde_json::json!("skin.estouchy") },
                    ]),
                    min: None,
                    max: None,
                },
                // Network settings
                KodiSetting {
                    id: "services.webserver".to_string(),
                    label: "Allow remote control via HTTP".to_string(),
                    category: "network".to_string(),
                    value: serde_json::json!(true),
                    setting_type: "boolean".to_string(),
                    options: None,
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "services.webserverport".to_string(),
                    label: "Port".to_string(),
                    category: "network".to_string(),
                    value: serde_json::json!(8080),
                    setting_type: "integer".to_string(),
                    options: None,
                    min: Some(1),
                    max: Some(65535),
                },
                // Cache settings
                KodiSetting {
                    id: "cache.buffermode".to_string(),
                    label: "Buffer mode".to_string(),
                    category: "cache".to_string(),
                    value: serde_json::json!(1),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption { label: "Buffer all internet filesystems".to_string(), value: serde_json::json!(0) },
                        SettingOption { label: "Buffer all filesystems".to_string(), value: serde_json::json!(1) },
                        SettingOption { label: "Only buffer true internet filesystems".to_string(), value: serde_json::json!(2) },
                        SettingOption { label: "No buffer".to_string(), value: serde_json::json!(3) },
                    ]),
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "cache.memorysize".to_string(),
                    label: "Memory size (MB)".to_string(),
                    category: "cache".to_string(),
                    value: serde_json::json!(20),
                    setting_type: "integer".to_string(),
                    options: None,
                    min: Some(0),
                    max: Some(1024),
                },
            ];

            if let Some(cat) = category {
                settings.retain(|s| s.category == cat);
            }

            return Ok(settings);
        }

        self.json_rpc_get_settings(category).await
    }

    pub async fn set_setting(&self, setting_id: &str, value: serde_json::Value) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Set Kodi setting {} = {:?}", setting_id, value);
            return Ok(());
        }

        self.json_rpc_call("Settings.SetSettingValue", serde_json::json!({
            "setting": setting_id,
            "value": value
        })).await?;
        Ok(())
    }

    // === Addons ===

    pub async fn get_addons(&self) -> Result<Vec<KodiAddon>, KodiError> {
        if self.dev_mode {
            return Ok(vec![
                KodiAddon {
                    id: "service.libreelec.settings".to_string(),
                    name: "LibreELEC Settings".to_string(),
                    version: "12.0.0".to_string(),
                    addon_type: "service".to_string(),
                    enabled: true,
                    installed: true,
                    description: Some("LibreELEC system settings".to_string()),
                    icon: None,
                },
                KodiAddon {
                    id: "skin.estuary".to_string(),
                    name: "Estuary".to_string(),
                    version: "3.0.1".to_string(),
                    addon_type: "skin".to_string(),
                    enabled: true,
                    installed: true,
                    description: Some("Default Kodi skin".to_string()),
                    icon: None,
                },
                KodiAddon {
                    id: "plugin.video.youtube".to_string(),
                    name: "YouTube".to_string(),
                    version: "7.0.5".to_string(),
                    addon_type: "video".to_string(),
                    enabled: true,
                    installed: true,
                    description: Some("Watch YouTube videos".to_string()),
                    icon: None,
                },
                KodiAddon {
                    id: "plugin.video.netflix".to_string(),
                    name: "Netflix".to_string(),
                    version: "1.18.0".to_string(),
                    addon_type: "video".to_string(),
                    enabled: false,
                    installed: true,
                    description: Some("Watch Netflix content".to_string()),
                    icon: None,
                },
            ]);
        }

        self.json_rpc_get_addons().await
    }

    pub async fn set_addon_enabled(&self, addon_id: &str, enabled: bool) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Set Kodi addon {} enabled: {}", addon_id, enabled);
            return Ok(());
        }

        self.json_rpc_call("Addons.SetAddonEnabled", serde_json::json!({
            "addonid": addon_id,
            "enabled": enabled
        })).await?;
        Ok(())
    }

    // === Library ===

    pub async fn scan_library(&self, library_type: &str) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Scanning {} library", library_type);
            return Ok(());
        }

        let method = match library_type {
            "video" => "VideoLibrary.Scan",
            "music" => "AudioLibrary.Scan",
            _ => return Err(KodiError::InvalidAction(library_type.to_string())),
        };

        self.json_rpc_call(method, serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn clean_library(&self, library_type: &str) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Cleaning {} library", library_type);
            return Ok(());
        }

        let method = match library_type {
            "video" => "VideoLibrary.Clean",
            "music" => "AudioLibrary.Clean",
            _ => return Err(KodiError::InvalidAction(library_type.to_string())),
        };

        self.json_rpc_call(method, serde_json::json!({})).await?;
        Ok(())
    }

    // === System ===

    pub async fn send_notification(&self, title: &str, message: &str) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi notification: {} - {}", title, message);
            return Ok(());
        }

        self.json_rpc_call("GUI.ShowNotification", serde_json::json!({
            "title": title,
            "message": message
        })).await?;
        Ok(())
    }

    pub async fn reboot(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi reboot requested");
            return Ok(());
        }

        self.json_rpc_call("System.Reboot", serde_json::json!({})).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi shutdown requested");
            return Ok(());
        }

        self.json_rpc_call("System.Shutdown", serde_json::json!({})).await?;
        Ok(())
    }

    // === Private JSON-RPC methods (stubs for real implementation) ===

    async fn json_rpc_call(&self, _method: &str, _params: serde_json::Value) -> Result<serde_json::Value, KodiError> {
        // TODO: Real JSON-RPC implementation
        // let client = reqwest::Client::new();
        // let response = client.post("http://localhost:8080/jsonrpc")
        //     .json(&json!({
        //         "jsonrpc": "2.0",
        //         "method": method,
        //         "params": params,
        //         "id": 1
        //     }))
        //     .send()
        //     .await?;
        Ok(serde_json::json!({}))
    }

    async fn json_rpc_get_status(&self) -> Result<KodiStatus, KodiError> {
        // TODO: Real implementation
        Err(KodiError::NotConnected)
    }

    async fn json_rpc_get_info(&self) -> Result<KodiInfo, KodiError> {
        // TODO: Real implementation
        Err(KodiError::NotConnected)
    }

    async fn json_rpc_get_settings(&self, _category: Option<&str>) -> Result<Vec<KodiSetting>, KodiError> {
        // TODO: Real implementation
        Ok(vec![])
    }

    async fn json_rpc_get_addons(&self) -> Result<Vec<KodiAddon>, KodiError> {
        // TODO: Real implementation
        Ok(vec![])
    }

    async fn read_sources_xml(&self, _source_type: Option<&str>) -> Result<Vec<MediaSource>, KodiError> {
        // TODO: Read from /storage/.kodi/userdata/sources.xml
        Ok(vec![])
    }

    async fn write_source_to_xml(&self, _source: &MediaSource) -> Result<(), KodiError> {
        // TODO: Write to /storage/.kodi/userdata/sources.xml
        Ok(())
    }

    async fn remove_source_from_xml(&self, _source_id: &str) -> Result<(), KodiError> {
        // TODO: Remove from /storage/.kodi/userdata/sources.xml
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KodiError {
    #[error("Not connected to Kodi")]
    NotConnected,

    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parse error: {0}")]
    XmlParse(String),
}
