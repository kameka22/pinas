//! System clock: time zone, NTP (systemd-timesyncd) and manual sync.
//! LibreELEC keeps the zone in /storage/.cache/timezone (applied by tz-data.service);
//! `timedatectl` is used where it works and ignored where the root filesystem is read-only.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::config::AppConfig;
use crate::services::settings::SettingsService;

const LE_TIMEZONE_FILE: &str = "/storage/.cache/timezone";
const TIMESYNCD_DROPIN: &str = "/storage/.config/timesyncd.conf.d/pinas.conf";
const DEFAULT_NTP_SERVERS: &str = "0.pool.ntp.org 1.pool.ntp.org";

#[derive(Debug, Clone, Serialize)]
pub struct TimeStatus {
    pub timezone: String,
    pub local_time: String,
    pub utc_time: String,
    pub ntp_enabled: bool,
    pub ntp_synchronized: bool,
    pub ntp_servers: Vec<String>,
    pub dev_mode: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeUpdate {
    pub timezone: Option<String>,
    pub ntp_enabled: Option<bool>,
    pub ntp_servers: Option<Vec<String>>,
}

pub struct TimeDateService {
    db: sqlx::SqlitePool,
    dev_mode: bool,
}

impl TimeDateService {
    pub fn new(db: sqlx::SqlitePool) -> Self {
        Self { db, dev_mode: AppConfig::global().dev_mode }
    }

    /// Every IANA zone name, sorted
    pub fn timezones() -> Vec<&'static str> {
        let mut zones: Vec<&'static str> = chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name()).collect();
        zones.sort_unstable();
        zones
    }

    pub fn is_valid_timezone(name: &str) -> bool {
        name.parse::<chrono_tz::Tz>().is_ok()
    }

    async fn run(cmd: &str, args: &[&str]) -> Result<String> {
        let out = Command::new(cmd).args(args).output().await?;
        if !out.status.success() {
            return Err(anyhow!("{} {}: {}", cmd, args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    async fn current_timezone(&self) -> String {
        let settings = SettingsService::new(self.db.clone());
        if self.dev_mode {
            return settings.get_or("time.timezone", "Europe/Paris").await.unwrap_or_else(|_| "UTC".into());
        }
        if let Ok(tz) = tokio::fs::read_to_string(LE_TIMEZONE_FILE).await {
            let tz = tz.trim();
            if Self::is_valid_timezone(tz) {
                return tz.to_string();
            }
        }
        if let Ok(tz) = Self::run("timedatectl", &["show", "-p", "Timezone", "--value"]).await {
            if Self::is_valid_timezone(&tz) {
                return tz;
            }
        }
        settings.get_or("time.timezone", "UTC").await.unwrap_or_else(|_| "UTC".into())
    }

    pub async fn status(&self) -> Result<TimeStatus> {
        let settings = SettingsService::new(self.db.clone());
        let timezone = self.current_timezone().await;
        let tz: chrono_tz::Tz = timezone.parse().unwrap_or(chrono_tz::UTC);
        let now = chrono::Utc::now();
        let ntp_servers: Vec<String> = settings
            .get_or("time.ntp_servers", DEFAULT_NTP_SERVERS)
            .await?
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let (ntp_enabled, ntp_synchronized) = if self.dev_mode {
            (settings.get_bool("time.ntp_enabled", true).await?, true)
        } else {
            let enabled = Self::run("timedatectl", &["show", "-p", "NTP", "--value"]).await.map(|v| v == "yes").unwrap_or(true);
            let synced = Self::run("timedatectl", &["show", "-p", "NTPSynchronized", "--value"]).await.map(|v| v == "yes").unwrap_or(false);
            (enabled, synced)
        };

        Ok(TimeStatus {
            timezone,
            local_time: now.with_timezone(&tz).to_rfc3339(),
            utc_time: now.to_rfc3339(),
            ntp_enabled,
            ntp_synchronized,
            ntp_servers,
            dev_mode: self.dev_mode,
        })
    }

    pub async fn update(&self, update: TimeUpdate) -> Result<TimeStatus> {
        let settings = SettingsService::new(self.db.clone());

        if let Some(tz) = update.timezone {
            if !Self::is_valid_timezone(&tz) {
                return Err(anyhow!("Unknown time zone: {}", tz));
            }
            settings.set("time.timezone", &tz).await?;
            if !self.dev_mode {
                // LibreELEC: the file is the source of truth at boot; timedatectl is best effort
                // (it fails on a read-only /etc), so the zone is also exported for this process.
                if let Some(parent) = std::path::Path::new(LE_TIMEZONE_FILE).parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                tokio::fs::write(LE_TIMEZONE_FILE, format!("{}\n", tz)).await?;
                if let Err(e) = Self::run("timedatectl", &["set-timezone", &tz]).await {
                    tracing::warn!("timedatectl set-timezone failed (expected on LibreELEC): {}", e);
                }
                let _ = Self::run("systemctl", &["restart", "tz-data.service"]).await;
            }
            tracing::info!("Time zone set to {}", tz);
        }

        if let Some(servers) = update.ntp_servers {
            let cleaned: Vec<String> = servers
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':')))
                .collect();
            if cleaned.is_empty() {
                return Err(anyhow!("At least one valid NTP server is required"));
            }
            settings.set("time.ntp_servers", &cleaned.join(" ")).await?;
            if !self.dev_mode {
                if let Some(parent) = std::path::Path::new(TIMESYNCD_DROPIN).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(TIMESYNCD_DROPIN, format!("[Time]\nNTP={}\n", cleaned.join(" "))).await?;
                let _ = Self::run("systemctl", &["restart", "systemd-timesyncd"]).await;
            }
        }

        if let Some(enabled) = update.ntp_enabled {
            settings.set("time.ntp_enabled", &enabled.to_string()).await?;
            if !self.dev_mode {
                Self::run("timedatectl", &["set-ntp", if enabled { "true" } else { "false" }]).await?;
            }
        }

        self.status().await
    }

    /// Force a resynchronization (restart timesyncd)
    pub async fn sync_now(&self) -> Result<TimeStatus> {
        if !self.dev_mode {
            Self::run("systemctl", &["restart", "systemd-timesyncd"]).await?;
        }
        self.status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timezone_list_is_sorted_and_validated() {
        let zones = TimeDateService::timezones();
        assert!(zones.len() > 300);
        assert!(zones.windows(2).all(|w| w[0] < w[1]));
        assert!(TimeDateService::is_valid_timezone("Europe/Paris"));
        assert!(!TimeDateService::is_valid_timezone("Mars/Olympus"));
    }
}
