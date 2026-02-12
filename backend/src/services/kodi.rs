use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const KODI_JSONRPC_URL: &str = "http://localhost:8080/jsonrpc";
const KODI_TIMEOUT: Duration = Duration::from_secs(5);
const SOURCES_XML_PATH: &str = "/storage/.kodi/userdata/sources.xml";

/// Kodi JSON-RPC service for controlling Kodi
pub struct KodiService {
    dev_mode: bool,
    client: Client,
    username: String,
    password: String,
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
    pub current_type: Option<String>,
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
    pub source_type: String,
    pub protocol: Option<String>,
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
    pub setting_type: String,
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
    pub fn new(dev_mode: bool, username: String, password: String) -> Self {
        let client = Client::builder()
            .timeout(KODI_TIMEOUT)
            .build()
            .unwrap_or_default();

        Self {
            dev_mode,
            client,
            username,
            password,
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
                *self.fake_current_title.lock().unwrap() =
                    Some("Big Buck Bunny (2008)".to_string());
            }
            tracing::info!("[DEV] Kodi play/pause toggled");
            return Ok(());
        }

        // Get active player first
        let players = self
            .json_rpc_call("Player.GetActivePlayers", serde_json::json!({}))
            .await?;
        if let Some(player) = players.as_array().and_then(|a| a.first()) {
            let playerid = player["playerid"].as_i64().unwrap_or(1);
            self.json_rpc_call(
                "Player.PlayPause",
                serde_json::json!({ "playerid": playerid }),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            self.fake_playing.store(false, Ordering::Relaxed);
            *self.fake_current_title.lock().unwrap() = None;
            tracing::info!("[DEV] Kodi playback stopped");
            return Ok(());
        }

        let players = self
            .json_rpc_call("Player.GetActivePlayers", serde_json::json!({}))
            .await?;
        if let Some(player) = players.as_array().and_then(|a| a.first()) {
            let playerid = player["playerid"].as_i64().unwrap_or(1);
            self.json_rpc_call("Player.Stop", serde_json::json!({ "playerid": playerid }))
                .await?;
        }
        Ok(())
    }

    pub async fn get_volume(&self) -> Result<u8, KodiError> {
        if self.dev_mode {
            return Ok(self.fake_volume.load(Ordering::Relaxed));
        }

        let result = self
            .json_rpc_call(
                "Application.GetProperties",
                serde_json::json!({ "properties": ["volume"] }),
            )
            .await?;

        Ok(result["volume"].as_u64().unwrap_or(50) as u8)
    }

    pub async fn set_volume(&self, volume: u8) -> Result<(), KodiError> {
        let volume = volume.min(100);

        if self.dev_mode {
            self.fake_volume.store(volume, Ordering::Relaxed);
            tracing::info!("[DEV] Kodi volume set to {}", volume);
            return Ok(());
        }

        self.json_rpc_call(
            "Application.SetVolume",
            serde_json::json!({ "volume": volume }),
        )
        .await?;
        Ok(())
    }

    pub async fn goto_previous(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi goto previous");
            return Ok(());
        }

        let players = self
            .json_rpc_call("Player.GetActivePlayers", serde_json::json!({}))
            .await?;
        if let Some(player) = players.as_array().and_then(|a| a.first()) {
            let playerid = player["playerid"].as_i64().unwrap_or(1);
            self.json_rpc_call(
                "Player.GoTo",
                serde_json::json!({ "playerid": playerid, "to": "previous" }),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn goto_next(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi goto next");
            return Ok(());
        }

        let players = self
            .json_rpc_call("Player.GetActivePlayers", serde_json::json!({}))
            .await?;
        if let Some(player) = players.as_array().and_then(|a| a.first()) {
            let playerid = player["playerid"].as_i64().unwrap_or(1);
            self.json_rpc_call(
                "Player.GoTo",
                serde_json::json!({ "playerid": playerid, "to": "next" }),
            )
            .await?;
        }
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

    pub async fn get_sources(
        &self,
        source_type: Option<&str>,
    ) -> Result<Vec<MediaSource>, KodiError> {
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

        self.read_sources_xml(source_type).await
    }

    pub async fn add_source(&self, source: &MediaSource) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!(
                "[DEV] Added Kodi source: {} -> {}",
                source.name,
                source.path
            );
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

    pub async fn get_settings(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<KodiSetting>, KodiError> {
        if self.dev_mode {
            let mut settings = vec![
                KodiSetting {
                    id: "videoplayer.adjustrefreshrate".to_string(),
                    label: "Adjust display refresh rate".to_string(),
                    category: "video".to_string(),
                    value: serde_json::json!(2),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption {
                            label: "Off".to_string(),
                            value: serde_json::json!(0),
                        },
                        SettingOption {
                            label: "On start/stop".to_string(),
                            value: serde_json::json!(1),
                        },
                        SettingOption {
                            label: "Always".to_string(),
                            value: serde_json::json!(2),
                        },
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
                        SettingOption {
                            label: "2.0".to_string(),
                            value: serde_json::json!(2),
                        },
                        SettingOption {
                            label: "5.1".to_string(),
                            value: serde_json::json!(6),
                        },
                        SettingOption {
                            label: "7.1".to_string(),
                            value: serde_json::json!(8),
                        },
                    ]),
                    min: None,
                    max: None,
                },
                KodiSetting {
                    id: "locale.language".to_string(),
                    label: "Language".to_string(),
                    category: "interface".to_string(),
                    value: serde_json::json!("resource.language.en_gb"),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption {
                            label: "English".to_string(),
                            value: serde_json::json!("resource.language.en_gb"),
                        },
                        SettingOption {
                            label: "Français".to_string(),
                            value: serde_json::json!("resource.language.fr_fr"),
                        },
                        SettingOption {
                            label: "Deutsch".to_string(),
                            value: serde_json::json!("resource.language.de_de"),
                        },
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
                        SettingOption {
                            label: "Estuary".to_string(),
                            value: serde_json::json!("skin.estuary"),
                        },
                        SettingOption {
                            label: "Estouchy".to_string(),
                            value: serde_json::json!("skin.estouchy"),
                        },
                    ]),
                    min: None,
                    max: None,
                },
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
                KodiSetting {
                    id: "cache.buffermode".to_string(),
                    label: "Buffer mode".to_string(),
                    category: "cache".to_string(),
                    value: serde_json::json!(1),
                    setting_type: "list".to_string(),
                    options: Some(vec![
                        SettingOption {
                            label: "Buffer all internet filesystems".to_string(),
                            value: serde_json::json!(0),
                        },
                        SettingOption {
                            label: "Buffer all filesystems".to_string(),
                            value: serde_json::json!(1),
                        },
                        SettingOption {
                            label: "Only buffer true internet filesystems".to_string(),
                            value: serde_json::json!(2),
                        },
                        SettingOption {
                            label: "No buffer".to_string(),
                            value: serde_json::json!(3),
                        },
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

    pub async fn set_setting(
        &self,
        setting_id: &str,
        value: serde_json::Value,
    ) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Set Kodi setting {} = {:?}", setting_id, value);
            return Ok(());
        }

        self.json_rpc_call(
            "Settings.SetSettingValue",
            serde_json::json!({
                "setting": setting_id,
                "value": value
            }),
        )
        .await?;
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

    pub async fn set_addon_enabled(
        &self,
        addon_id: &str,
        enabled: bool,
    ) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!(
                "[DEV] Set Kodi addon {} enabled: {}",
                addon_id,
                enabled
            );
            return Ok(());
        }

        self.json_rpc_call(
            "Addons.SetAddonEnabled",
            serde_json::json!({
                "addonid": addon_id,
                "enabled": enabled
            }),
        )
        .await?;
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

        self.json_rpc_call(
            "GUI.ShowNotification",
            serde_json::json!({
                "title": title,
                "message": message
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn reboot(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi reboot requested");
            return Ok(());
        }

        self.json_rpc_call("System.Reboot", serde_json::json!({}))
            .await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), KodiError> {
        if self.dev_mode {
            tracing::info!("[DEV] Kodi shutdown requested");
            return Ok(());
        }

        self.json_rpc_call("System.Shutdown", serde_json::json!({}))
            .await?;
        Ok(())
    }

    // =========================================================================
    // Private JSON-RPC implementation
    // =========================================================================

    /// Send a JSON-RPC 2.0 request to Kodi and return the "result" field.
    async fn json_rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, KodiError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .client
            .post(KODI_JSONRPC_URL)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                tracing::warn!("Kodi JSON-RPC request failed: {}", e);
                KodiError::NotConnected
            })?;

        let json: serde_json::Value = response.json().await.map_err(|e| {
            tracing::warn!("Kodi JSON-RPC response parse error: {}", e);
            KodiError::JsonRpc("Invalid JSON response".to_string())
        })?;

        if let Some(error) = json.get("error") {
            let msg = error["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            tracing::warn!("Kodi JSON-RPC error for {}: {}", method, msg);
            return Err(KodiError::JsonRpc(msg));
        }

        Ok(json.get("result").cloned().unwrap_or(serde_json::json!(null)))
    }

    /// Get full player status: connected, playing, volume, current item
    async fn json_rpc_get_status(&self) -> Result<KodiStatus, KodiError> {
        // 1) Check connectivity + get app properties (volume, muted, version, name)
        let app_props = self
            .json_rpc_call(
                "Application.GetProperties",
                serde_json::json!({ "properties": ["volume", "muted", "version", "name"] }),
            )
            .await?;

        let volume = app_props["volume"].as_u64().unwrap_or(50) as u8;
        let muted = app_props["muted"].as_bool().unwrap_or(false);
        let version = if let Some(v) = app_props.get("version") {
            format!(
                "{}.{}",
                v["major"].as_i64().unwrap_or(0),
                v["minor"].as_i64().unwrap_or(0)
            )
        } else {
            "unknown".to_string()
        };
        let name = app_props["name"]
            .as_str()
            .unwrap_or("Kodi")
            .to_string();

        // 2) Get active players
        let players = self
            .json_rpc_call("Player.GetActivePlayers", serde_json::json!({}))
            .await?;

        let active_players = players.as_array().map(|a| a.len()).unwrap_or(0);
        let mut playing = false;
        let mut paused = false;
        let mut current_title: Option<String> = None;
        let mut current_type: Option<String> = None;

        if active_players > 0 {
            if let Some(player) = players.as_array().and_then(|a| a.first()) {
                let playerid = player["playerid"].as_i64().unwrap_or(1);
                let ptype = player["type"].as_str().unwrap_or("video").to_string();

                // Get player properties (speed → 0 = paused, 1 = playing)
                let player_props = self
                    .json_rpc_call(
                        "Player.GetProperties",
                        serde_json::json!({
                            "playerid": playerid,
                            "properties": ["speed", "type"]
                        }),
                    )
                    .await
                    .ok();

                let speed = player_props
                    .as_ref()
                    .and_then(|p| p["speed"].as_i64())
                    .unwrap_or(0);

                playing = speed != 0;
                paused = speed == 0 && active_players > 0;

                // Get current playing item
                let item_result = self
                    .json_rpc_call(
                        "Player.GetItem",
                        serde_json::json!({
                            "playerid": playerid,
                            "properties": ["title", "showtitle", "artist", "album"]
                        }),
                    )
                    .await
                    .ok();

                if let Some(item) = item_result.and_then(|r| r.get("item").cloned()) {
                    let title = item["title"].as_str().unwrap_or("").to_string();
                    let showtitle = item["showtitle"].as_str().unwrap_or("");

                    current_title = Some(if !showtitle.is_empty() {
                        format!("{} - {}", showtitle, title)
                    } else if !title.is_empty() {
                        title
                    } else {
                        item["label"].as_str().unwrap_or("Unknown").to_string()
                    });

                    current_type = Some(
                        item["type"]
                            .as_str()
                            .unwrap_or(&ptype)
                            .to_string(),
                    );
                }
            }
        }

        Ok(KodiStatus {
            connected: true,
            version,
            name,
            playing,
            paused,
            current_title,
            current_type,
            volume,
            muted,
        })
    }

    /// Get detailed system info
    async fn json_rpc_get_info(&self) -> Result<KodiInfo, KodiError> {
        let sys_props = self
            .json_rpc_call(
                "System.GetProperties",
                serde_json::json!({ "properties": ["cpuusage", "cputemperature"] }),
            )
            .await
            .ok();

        let app_props = self
            .json_rpc_call(
                "Application.GetProperties",
                serde_json::json!({ "properties": ["version", "name"] }),
            )
            .await?;

        let version = if let Some(v) = app_props.get("version") {
            format!(
                "{}.{}.{}",
                v["major"].as_i64().unwrap_or(0),
                v["minor"].as_i64().unwrap_or(0),
                v["revision"].as_str().unwrap_or(
                    &v["revision"].as_i64().map(|n| n.to_string()).unwrap_or_default()
                )
            )
        } else {
            "unknown".to_string()
        };

        let name = app_props["name"]
            .as_str()
            .unwrap_or("Kodi")
            .to_string();

        let cpu_temp = sys_props
            .as_ref()
            .and_then(|p| p["cputemperature"].as_str())
            .and_then(|s| {
                // Kodi returns something like "52.0 °C"
                s.split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<f32>().ok())
            });

        // Get free space via Files.GetDirectory on /storage
        let free_space = self.get_free_space().await;

        // Get uptime from XBMC.GetInfoLabels
        let uptime = self.get_uptime().await;

        Ok(KodiInfo {
            version: format!("{} {}", name, version),
            build: version.clone(),
            name,
            uptime: uptime.unwrap_or_else(|| "N/A".to_string()),
            cpu_temp,
            free_space: free_space.unwrap_or_else(|| "N/A".to_string()),
        })
    }

    async fn get_uptime(&self) -> Option<String> {
        let result = self
            .json_rpc_call(
                "XBMC.GetInfoLabels",
                serde_json::json!({ "labels": ["System.Uptime"] }),
            )
            .await
            .ok()?;

        result["System.Uptime"]
            .as_str()
            .map(|s| s.to_string())
    }

    async fn get_free_space(&self) -> Option<String> {
        let result = self
            .json_rpc_call(
                "XBMC.GetInfoLabels",
                serde_json::json!({ "labels": ["System.FreeSpace"] }),
            )
            .await
            .ok()?;

        result["System.FreeSpace"]
            .as_str()
            .map(|s| s.to_string())
    }

    /// Get settings via JSON-RPC
    async fn json_rpc_get_settings(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<KodiSetting>, KodiError> {
        let mut params = serde_json::json!({
            "level": "standard",
            "properties": ["value"]
        });

        if let Some(cat) = category {
            let kodi_category = match cat {
                "video" => "videoplayer",
                "audio" => "audiooutput",
                "interface" => "lookandfeel",
                "network" => "services",
                "cache" => "filecache",
                other => other,
            };
            params["filter"] = serde_json::json!({ "category": kodi_category });
        }

        let result = self
            .json_rpc_call("Settings.GetSettings", params)
            .await?;

        let mut settings = Vec::new();

        if let Some(items) = result["settings"].as_array() {
            for item in items {
                let setting_type = match item["type"].as_str().unwrap_or("") {
                    "boolean" => "boolean",
                    "integer" => "integer",
                    "number" => "integer",
                    "string" => "string",
                    _ => continue, // skip unsupported types (path, action, etc.)
                };

                // Map Kodi categories to our categories
                let kodi_cat = item["parent"].as_str().unwrap_or("");
                let mapped_category = if kodi_cat.starts_with("videoplayer") || kodi_cat.starts_with("videoscreen") {
                    "video"
                } else if kodi_cat.starts_with("audio") {
                    "audio"
                } else if kodi_cat.starts_with("lookandfeel") || kodi_cat.starts_with("locale") {
                    "interface"
                } else if kodi_cat.starts_with("services") || kodi_cat.starts_with("network") {
                    "network"
                } else if kodi_cat.starts_with("filecache") || kodi_cat.starts_with("cache") {
                    "cache"
                } else {
                    kodi_cat.split('.').next().unwrap_or("other")
                };

                // Build options for integer settings with defined values
                let options = item.get("options").and_then(|opts| {
                    opts.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|o| {
                                Some(SettingOption {
                                    label: o["label"].as_str()?.to_string(),
                                    value: o["value"].clone(),
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                });

                let has_options = options.as_ref().map(|o| !o.is_empty()).unwrap_or(false);

                settings.push(KodiSetting {
                    id: item["id"].as_str().unwrap_or("").to_string(),
                    label: item["label"].as_str().unwrap_or("").to_string(),
                    category: mapped_category.to_string(),
                    value: item["value"].clone(),
                    setting_type: if has_options {
                        "list".to_string()
                    } else {
                        setting_type.to_string()
                    },
                    options: if has_options { options } else { None },
                    min: item.get("minimum").and_then(|v| v.as_i64()),
                    max: item.get("maximum").and_then(|v| v.as_i64()),
                });
            }
        }

        Ok(settings)
    }

    /// Get addons via JSON-RPC
    async fn json_rpc_get_addons(&self) -> Result<Vec<KodiAddon>, KodiError> {
        let result = self
            .json_rpc_call(
                "Addons.GetAddons",
                serde_json::json!({
                    "properties": ["name", "version", "enabled", "installed", "description", "thumbnail"],
                    "installed": true
                }),
            )
            .await?;

        let mut addons = Vec::new();

        if let Some(items) = result["addons"].as_array() {
            for item in items {
                let addon_type = item["type"]
                    .as_str()
                    .unwrap_or("")
                    .split('.')
                    .nth(1)
                    .unwrap_or("unknown")
                    .to_string();

                addons.push(KodiAddon {
                    id: item["addonid"].as_str().unwrap_or("").to_string(),
                    name: item["name"].as_str().unwrap_or("").to_string(),
                    version: item["version"].as_str().unwrap_or("0.0.0").to_string(),
                    addon_type,
                    enabled: item["enabled"].as_bool().unwrap_or(false),
                    installed: item["installed"]
                        .as_bool()
                        .or(item["installed"].as_str().map(|s| s == "true"))
                        .unwrap_or(true),
                    description: item["description"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                    icon: item["thumbnail"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                });
            }
        }

        Ok(addons)
    }

    // =========================================================================
    // Sources XML read/write (/storage/.kodi/userdata/sources.xml)
    // =========================================================================

    /// Read Kodi sources.xml and return parsed MediaSource list
    async fn read_sources_xml(
        &self,
        source_type: Option<&str>,
    ) -> Result<Vec<MediaSource>, KodiError> {
        let content = tokio::fs::read_to_string(SOURCES_XML_PATH)
            .await
            .map_err(|e| {
                tracing::warn!("Failed to read sources.xml: {}", e);
                KodiError::Io(e)
            })?;

        let mut sources = Vec::new();
        let source_types = if let Some(st) = source_type {
            vec![st.to_string()]
        } else {
            vec![
                "video".to_string(),
                "music".to_string(),
                "pictures".to_string(),
                "files".to_string(),
                "programs".to_string(),
            ]
        };

        for stype in &source_types {
            // Find the <{type}> section
            let open_tag = format!("<{}>", stype);
            let close_tag = format!("</{}>", stype);

            let section = match content.find(&open_tag) {
                Some(start) => match content[start..].find(&close_tag) {
                    Some(end) => &content[start..start + end + close_tag.len()],
                    None => continue,
                },
                None => continue,
            };

            // Parse each <source> block within the section
            let mut search_from = 0;
            while let Some(src_start) = section[search_from..].find("<source>") {
                let abs_start = search_from + src_start;
                let src_end = match section[abs_start..].find("</source>") {
                    Some(e) => abs_start + e + "</source>".len(),
                    None => break,
                };

                let source_block = &section[abs_start..src_end];

                let name = Self::extract_xml_value(source_block, "name")
                    .unwrap_or_default();
                let path = Self::extract_xml_tag_value(source_block, "path")
                    .unwrap_or_default();

                if !name.is_empty() && !path.is_empty() {
                    let protocol = if path.starts_with("smb://") {
                        Some("smb".to_string())
                    } else if path.starts_with("nfs://") {
                        Some("nfs".to_string())
                    } else if path.starts_with("http://") || path.starts_with("https://") {
                        Some("http".to_string())
                    } else {
                        Some("local".to_string())
                    };

                    // Generate a stable ID from type+name
                    let id = format!("{}:{}", stype, name);

                    sources.push(MediaSource {
                        id,
                        name,
                        path,
                        source_type: stype.clone(),
                        protocol,
                    });
                }

                search_from = src_end;
            }
        }

        Ok(sources)
    }

    /// Add a source to sources.xml
    async fn write_source_to_xml(&self, source: &MediaSource) -> Result<(), KodiError> {
        let mut content = tokio::fs::read_to_string(SOURCES_XML_PATH)
            .await
            .unwrap_or_else(|_| Self::default_sources_xml());

        let stype = &source.source_type;
        let close_tag = format!("</{}>", stype);

        // Build the new <source> block
        let new_source = format!(
            "        <source>\n            <name>{}</name>\n            <path pathversion=\"1\">{}</path>\n            <allowsharing>true</allowsharing>\n        </source>\n    ",
            Self::escape_xml(&source.name),
            Self::escape_xml(&source.path)
        );

        if let Some(pos) = content.find(&close_tag) {
            // Insert before the closing tag
            content.insert_str(pos, &new_source);
        } else {
            // Section doesn't exist, create it before </sources>
            let section = format!(
                "    <{}>\n        <default pathversion=\"1\"></default>\n{}</{}>\n",
                stype, new_source, stype
            );
            if let Some(pos) = content.find("</sources>") {
                content.insert_str(pos, &section);
            }
        }

        tokio::fs::write(SOURCES_XML_PATH, &content)
            .await
            .map_err(KodiError::Io)?;

        tracing::info!("Added Kodi source: {} -> {}", source.name, source.path);
        Ok(())
    }

    /// Remove a source from sources.xml by its id ("type:name")
    async fn remove_source_from_xml(&self, source_id: &str) -> Result<(), KodiError> {
        let mut content = tokio::fs::read_to_string(SOURCES_XML_PATH)
            .await
            .map_err(KodiError::Io)?;

        // source_id format: "type:name"
        let parts: Vec<&str> = source_id.splitn(2, ':').collect();
        let source_name = if parts.len() == 2 { parts[1] } else { source_id };

        // Find and remove the <source> block containing this name
        let name_tag = format!("<name>{}</name>", Self::escape_xml(source_name));

        if let Some(name_pos) = content.find(&name_tag) {
            // Find the enclosing <source>...</source>
            let src_start = content[..name_pos]
                .rfind("<source>")
                .ok_or_else(|| KodiError::XmlParse("Malformed sources.xml".to_string()))?;
            let src_end = content[src_start..]
                .find("</source>")
                .map(|e| src_start + e + "</source>".len())
                .ok_or_else(|| KodiError::XmlParse("Malformed sources.xml".to_string()))?;

            // Also remove trailing whitespace/newline
            let end = if content[src_end..].starts_with('\n') {
                src_end + 1
            } else {
                src_end
            };

            content.replace_range(src_start..end, "");

            tokio::fs::write(SOURCES_XML_PATH, &content)
                .await
                .map_err(KodiError::Io)?;

            tracing::info!("Removed Kodi source: {}", source_name);
            Ok(())
        } else {
            Err(KodiError::XmlParse(format!(
                "Source '{}' not found",
                source_name
            )))
        }
    }

    // =========================================================================
    // XML helpers
    // =========================================================================

    /// Extract value from simple XML tag: <tag>value</tag>
    fn extract_xml_value(block: &str, tag: &str) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let start = block.find(&open)? + open.len();
        let end = block[start..].find(&close)? + start;
        let value = block[start..end].trim().to_string();
        Some(Self::unescape_xml(&value))
    }

    /// Extract value from XML tag with attributes: <tag attr="...">value</tag>
    fn extract_xml_tag_value(block: &str, tag: &str) -> Option<String> {
        let open_start = format!("<{}", tag);
        let close = format!("</{}>", tag);
        let tag_start = block.find(&open_start)? + open_start.len();
        // Find the end of the opening tag (after attributes)
        let content_start = block[tag_start..].find('>')? + tag_start + 1;
        let content_end = block[content_start..].find(&close)? + content_start;
        let value = block[content_start..content_end].trim().to_string();
        Some(Self::unescape_xml(&value))
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn unescape_xml(s: &str) -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
    }

    fn default_sources_xml() -> String {
        r#"<sources>
    <video>
        <default pathversion="1"></default>
    </video>
    <music>
        <default pathversion="1"></default>
    </music>
    <pictures>
        <default pathversion="1"></default>
    </pictures>
    <files>
        <default pathversion="1"></default>
    </files>
    <programs>
        <default pathversion="1"></default>
    </programs>
</sources>
"#
        .to_string()
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
