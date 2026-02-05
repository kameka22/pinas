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

/// SSH service manager
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

        // Check if sshd is enabled
        let enabled_output = AsyncCommand::new("systemctl")
            .args(["is-enabled", "sshd"])
            .output()
            .await;

        let enabled = enabled_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
            .unwrap_or(false);

        // Check if sshd is running
        let active_output = AsyncCommand::new("systemctl")
            .args(["is-active", "sshd"])
            .output()
            .await;

        let running = active_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false);

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
        // Try to read from sshd_config
        let config_paths = [
            "/etc/ssh/sshd_config",
            "/storage/.config/ssh/sshd_config",
        ];

        for path in &config_paths {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("Port ") {
                        if let Some(port_str) = line.strip_prefix("Port ") {
                            if let Ok(port) = port_str.trim().parse::<u16>() {
                                return Ok(port);
                            }
                        }
                    }
                }
            }
        }

        Ok(22) // Default SSH port
    }

    /// Enable SSH service
    pub async fn enable(&self) -> anyhow::Result<()> {
        // In dev mode, just update the fake state
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would enable SSH service");
            DEV_SSH_ENABLED.store(true, Ordering::Relaxed);
            return Ok(());
        }

        // Enable the service
        let output = AsyncCommand::new("systemctl")
            .args(["enable", "sshd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to enable SSH: {}", stderr);
        }

        // Start the service
        let output = AsyncCommand::new("systemctl")
            .args(["start", "sshd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
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
        let output = AsyncCommand::new("systemctl")
            .args(["stop", "sshd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to stop SSH (may already be stopped): {}", stderr);
        }

        // Disable the service
        let output = AsyncCommand::new("systemctl")
            .args(["disable", "sshd"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to disable SSH: {}", stderr);
        }

        tracing::info!("SSH service stopped and disabled");
        Ok(())
    }

    /// Change SSH password (root password in LibreELEC)
    /// This uses the same mechanism as LibreELEC/Kodi for changing the root password
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
