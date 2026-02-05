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

/// SSH service manager for LibreELEC
///
/// On LibreELEC, SSH is provided by OpenSSH (sshd).
/// The service can be controlled via systemctl, but may also be
/// controlled by Kodi Settings (which uses kernel command line).
///
/// Configuration files:
/// - /etc/ssh/sshd_config (read-only, system default)
/// - /storage/.cache/services/sshd.conf (read-write, runtime config)
/// - SSH keys in /storage/.cache/ssh/
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
        // Return fake data in dev mode
        if self.dev_mode {
            let enabled = DEV_SSH_ENABLED.load(Ordering::Relaxed);
            return Ok(SshStatus {
                enabled,
                running: enabled, // In dev mode, running = enabled
                port: 22,
            });
        }

        // Check if sshd is running (most reliable indicator on LibreELEC)
        let active_output = AsyncCommand::new("systemctl")
            .args(["is-active", "sshd"])
            .output()
            .await;

        let running = active_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false);

        // Check if sshd is enabled (for auto-start on boot)
        let enabled_output = AsyncCommand::new("systemctl")
            .args(["is-enabled", "sshd"])
            .output()
            .await;

        let enabled = enabled_output
            .map(|o| {
                let status = String::from_utf8_lossy(&o.stdout);
                let status = status.trim();
                // "enabled" or "enabled-runtime" both count as enabled
                status == "enabled" || status == "enabled-runtime" || status == "static"
            })
            .unwrap_or(false);

        // If running but shows disabled, consider it enabled (LibreELEC quirk)
        let enabled = enabled || running;

        // Get SSH port from sshd config
        let port = self.get_ssh_port().await.unwrap_or(22);

        Ok(SshStatus {
            enabled,
            running,
            port,
        })
    }

    /// Get SSH port from config
    async fn get_ssh_port(&self) -> anyhow::Result<u16> {
        // Try to read from sshd_config (LibreELEC paths)
        let config_paths = [
            "/storage/.cache/services/sshd.conf",
            "/etc/ssh/sshd_config",
        ];

        for path in &config_paths {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                for line in content.lines() {
                    let line = line.trim();
                    // Skip comments
                    if line.starts_with('#') {
                        continue;
                    }
                    if line.to_lowercase().starts_with("port ") {
                        if let Some(port_str) = line.split_whitespace().nth(1) {
                            if let Ok(port) = port_str.parse::<u16>() {
                                return Ok(port);
                            }
                        }
                    }
                }
            }
        }

        Ok(22) // Default SSH port
    }

    /// Ensure SSH configuration directory exists
    async fn ensure_ssh_config_dir(&self) -> anyhow::Result<()> {
        let config_dir = "/storage/.cache/services";
        if !std::path::Path::new(config_dir).exists() {
            tokio::fs::create_dir_all(config_dir).await?;
        }
        Ok(())
    }

    /// Enable SSH service
    pub async fn enable(&self) -> anyhow::Result<()> {
        // In dev mode, just update the fake state
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would enable SSH service");
            DEV_SSH_ENABLED.store(true, Ordering::Relaxed);
            return Ok(());
        }

        // Ensure config directory exists
        self.ensure_ssh_config_dir().await.ok();

        // Try to start the service first (it may already be enabled)
        let start_output = AsyncCommand::new("systemctl")
            .args(["start", "sshd"])
            .output()
            .await?;

        if start_output.status.success() {
            tracing::info!("SSH service started successfully");

            // Also enable for boot
            let _ = AsyncCommand::new("systemctl")
                .args(["enable", "sshd"])
                .output()
                .await;

            return Ok(());
        }

        // If start failed, try enabling first then starting
        let enable_output = AsyncCommand::new("systemctl")
            .args(["enable", "sshd"])
            .output()
            .await?;

        if !enable_output.status.success() {
            let stderr = String::from_utf8_lossy(&enable_output.stderr);
            tracing::warn!("Failed to enable SSH (may already be enabled): {}", stderr);
        }

        // Try starting again
        let start_output = AsyncCommand::new("systemctl")
            .args(["start", "sshd"])
            .output()
            .await?;

        if !start_output.status.success() {
            let stderr = String::from_utf8_lossy(&start_output.stderr);
            anyhow::bail!("Failed to start SSH: {}", stderr);
        }

        tracing::info!("SSH service enabled and started");
        Ok(())
    }

    /// Disable SSH service
    pub async fn disable(&self) -> anyhow::Result<()> {
        // In dev mode, just update the fake state
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would disable SSH service");
            DEV_SSH_ENABLED.store(false, Ordering::Relaxed);
            return Ok(());
        }

        // Stop the service
        let stop_output = AsyncCommand::new("systemctl")
            .args(["stop", "sshd"])
            .output()
            .await?;

        if !stop_output.status.success() {
            let stderr = String::from_utf8_lossy(&stop_output.stderr);
            tracing::warn!("Failed to stop SSH (may already be stopped): {}", stderr);
        }

        // Disable the service (prevent auto-start)
        let disable_output = AsyncCommand::new("systemctl")
            .args(["disable", "sshd"])
            .output()
            .await?;

        if !disable_output.status.success() {
            let stderr = String::from_utf8_lossy(&disable_output.stderr);
            // Don't fail if disable doesn't work - the service is stopped
            tracing::warn!("Failed to disable SSH (service may be static): {}", stderr);
        }

        tracing::info!("SSH service stopped");
        Ok(())
    }

    /// Change SSH password (root password in LibreELEC)
    ///
    /// On LibreELEC, there is only one user (root), and the SSH password
    /// is the root system password. This uses chpasswd to change it.
    pub async fn change_password(&self, new_password: &str) -> anyhow::Result<()> {
        // Validate password
        if new_password.len() < 4 {
            anyhow::bail!("Password must be at least 4 characters");
        }

        // In dev mode, just log the action
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would change SSH password (length: {})", new_password.len());
            return Ok(());
        }

        // Use chpasswd to change the root password
        // This is the standard way to change passwords non-interactively
        let mut child = AsyncCommand::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Write "root:password" to stdin
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
