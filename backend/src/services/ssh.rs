use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::process::Command as AsyncCommand;

/// SSH service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshStatus {
    pub enabled: bool,
    pub running: bool,
    pub port: u16,
}

/// LibreELEC SSH config file path
/// This file controls whether sshd starts at boot.
/// Format: SSHD_START="true" or SSHD_START="false"
const SSHD_CONF_PATH: &str = "/storage/.cache/services/sshd.conf";
const SSHD_CONF_DIR: &str = "/storage/.cache/services";

/// Possible service names for SSH daemon on LibreELEC
const SSHD_SERVICE_NAMES: &[&str] = &["sshd", "sshd.service", "dropbear"];

/// SSH service manager for LibreELEC
///
/// On LibreELEC, SSH is controlled via:
/// 1. Config file: /storage/.cache/services/sshd.conf (SSHD_START=true/false)
/// 2. Service: systemctl start/stop sshd
///
/// systemctl enable/disable does NOT work (service is "static").
/// The config file is what controls auto-start at boot.
pub struct SshService {
    dev_mode: bool,
}

// Dev mode state (simulated SSH status)
static DEV_SSH_ENABLED: AtomicBool = AtomicBool::new(false);

impl SshService {
    pub fn new() -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::info!("SshService running in dev mode - using fake SSH data");
        }

        Self { dev_mode }
    }

    /// Get current SSH status
    pub async fn get_status(&self) -> anyhow::Result<SshStatus> {
        if self.dev_mode {
            let enabled = DEV_SSH_ENABLED.load(Ordering::Relaxed);
            return Ok(SshStatus {
                enabled,
                running: enabled,
                port: 22,
            });
        }

        // Check if sshd process is actually running
        let running = self.is_sshd_running().await;

        // Check if SSH is enabled in config file (for auto-start at boot)
        let config_enabled = self.read_sshd_conf().await;

        // Consider enabled if config says true OR if it's actually running
        let enabled = config_enabled || running;

        // Get SSH port from sshd config
        let port = self.get_ssh_port().await.unwrap_or(22);

        Ok(SshStatus {
            enabled,
            running,
            port,
        })
    }

    /// Check if sshd is actually running (try multiple methods)
    async fn is_sshd_running(&self) -> bool {
        // Method 1: systemctl is-active (try known service names)
        for name in SSHD_SERVICE_NAMES {
            let output = AsyncCommand::new("systemctl")
                .args(["is-active", name])
                .output()
                .await;

            if let Ok(o) = output {
                if String::from_utf8_lossy(&o.stdout).trim() == "active" {
                    return true;
                }
            }
        }

        // Method 2: check if sshd process exists (fallback)
        let output = AsyncCommand::new("pgrep")
            .args(["-x", "sshd"])
            .output()
            .await;

        if let Ok(o) = output {
            if o.status.success() {
                return true;
            }
        }

        false
    }

    /// Read SSHD_START value from config file
    async fn read_sshd_conf(&self) -> bool {
        match tokio::fs::read_to_string(SSHD_CONF_PATH).await {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("SSHD_START=") {
                        let value = line
                            .trim_start_matches("SSHD_START=")
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_lowercase();
                        return value == "true";
                    }
                }
                // No SSHD_START line found — default behavior depends on LibreELEC version
                // If the file exists but no SSHD_START, assume enabled
                true
            }
            Err(_) => {
                // No config file — SSH may still be running (default on fresh install)
                // Return false so we don't wrongly show enabled when no config exists
                false
            }
        }
    }

    /// Write SSHD_START value to config file
    async fn write_sshd_conf(&self, enabled: bool) -> anyhow::Result<()> {
        // Ensure directory exists
        if !std::path::Path::new(SSHD_CONF_DIR).exists() {
            tokio::fs::create_dir_all(SSHD_CONF_DIR).await?;
        }

        let value = if enabled { "true" } else { "false" };
        let content = format!("SSHD_START=\"{}\"\n", value);

        tokio::fs::write(SSHD_CONF_PATH, &content).await?;
        tracing::info!("Wrote sshd.conf: SSHD_START={}", value);
        Ok(())
    }

    /// Find the actual sshd service name on this system
    async fn find_sshd_service(&self) -> Option<&'static str> {
        for name in SSHD_SERVICE_NAMES {
            let output = AsyncCommand::new("systemctl")
                .args(["cat", name])
                .output()
                .await;

            if let Ok(o) = output {
                if o.status.success() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Get SSH port from config
    async fn get_ssh_port(&self) -> anyhow::Result<u16> {
        let config_paths = [
            "/storage/.cache/services/sshd.conf",
            "/storage/.config/ssh/sshd_config",
            "/etc/ssh/sshd_config",
        ];

        for path in &config_paths {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with('#') {
                        continue;
                    }
                    // Handle both "Port 22" and "SSHD_PORT=22"
                    if line.to_lowercase().starts_with("port ") {
                        if let Some(port_str) = line.split_whitespace().nth(1) {
                            if let Ok(port) = port_str.parse::<u16>() {
                                return Ok(port);
                            }
                        }
                    }
                    if line.starts_with("SSHD_PORT=") {
                        let val = line
                            .trim_start_matches("SSHD_PORT=")
                            .trim_matches('"')
                            .trim_matches('\'');
                        if let Ok(port) = val.parse::<u16>() {
                            return Ok(port);
                        }
                    }
                }
            }
        }

        Ok(22)
    }

    /// Enable SSH service
    pub async fn enable(&self) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would enable SSH service");
            DEV_SSH_ENABLED.store(true, Ordering::Relaxed);
            return Ok(());
        }

        // 1) Write config file (persists across reboots)
        self.write_sshd_conf(true).await?;

        // 2) Start the service
        let service = self.find_sshd_service().await.unwrap_or("sshd");

        let output = AsyncCommand::new("systemctl")
            .args(["start", service])
            .output()
            .await?;

        if !output.status.success() {
            // Try restart in case it was in a failed state
            let output2 = AsyncCommand::new("systemctl")
                .args(["restart", service])
                .output()
                .await?;

            if !output2.status.success() {
                let stderr = String::from_utf8_lossy(&output2.stderr);
                tracing::warn!("Failed to start SSH via systemctl: {}", stderr);
                // Don't fail — config is written, service may start on next boot
            }
        }

        // Verify it's actually running
        if self.is_sshd_running().await {
            tracing::info!("SSH service enabled and running");
        } else {
            tracing::warn!("SSH config written but service may not be running yet");
        }

        Ok(())
    }

    /// Disable SSH service
    pub async fn disable(&self) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would disable SSH service");
            DEV_SSH_ENABLED.store(false, Ordering::Relaxed);
            return Ok(());
        }

        // 1) Write config file (persists across reboots)
        self.write_sshd_conf(false).await?;

        // 2) Stop the service
        let service = self.find_sshd_service().await.unwrap_or("sshd");

        let output = AsyncCommand::new("systemctl")
            .args(["stop", service])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to stop SSH via systemctl: {} (service may already be stopped)", stderr);
        }

        tracing::info!("SSH service disabled");
        Ok(())
    }

    /// Change SSH password (root password in LibreELEC)
    pub async fn change_password(&self, new_password: &str) -> anyhow::Result<()> {
        if new_password.len() < 12 {
            anyhow::bail!("Password must be at least 12 characters");
        }

        if self.dev_mode {
            tracing::info!(
                "[DEV MODE] Would change SSH password (length: {})",
                new_password.len()
            );
            return Ok(());
        }

        let mut child = AsyncCommand::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            let input = format!("root:{}\n", new_password);
            stdin.write_all(input.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to change password: {}", stderr);
        }

        tracing::info!("SSH password changed successfully");
        Ok(())
    }
}

impl Default for SshService {
    fn default() -> Self {
        Self::new()
    }
}
