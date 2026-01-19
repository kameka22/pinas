use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

/// Service status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub enabled: bool,
    pub uptime: Option<u64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub pid: Option<u32>,
    pub description: Option<String>,
}

/// Log entry from service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Service for managing systemd services
pub struct ServiceManager;

impl ServiceManager {
    pub fn new() -> Self {
        Self
    }

    /// Get status of a specific service
    pub async fn get_status(&self, service_name: &str) -> anyhow::Result<ServiceStatus> {
        // Sanitize service name to prevent command injection
        let safe_name = sanitize_service_name(service_name)?;

        // Check if service is active
        let active_output = AsyncCommand::new("systemctl")
            .args(["is-active", &safe_name])
            .output()
            .await;

        let running = active_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
            .unwrap_or(false);

        // Check if service is enabled
        let enabled_output = AsyncCommand::new("systemctl")
            .args(["is-enabled", &safe_name])
            .output()
            .await;

        let enabled = enabled_output
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
            .unwrap_or(false);

        // Get service details via show
        let show_output = AsyncCommand::new("systemctl")
            .args(["show", &safe_name, "--property=MainPID,MemoryCurrent,Description,ActiveEnterTimestamp"])
            .output()
            .await;

        let mut pid: Option<u32> = None;
        let mut memory_usage: Option<u64> = None;
        let mut description: Option<String> = None;
        let mut uptime: Option<u64> = None;

        if let Ok(output) = show_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    match key {
                        "MainPID" => {
                            if let Ok(p) = value.parse::<u32>() {
                                if p > 0 {
                                    pid = Some(p);
                                }
                            }
                        }
                        "MemoryCurrent" => {
                            if let Ok(m) = value.parse::<u64>() {
                                if m < u64::MAX {
                                    memory_usage = Some(m);
                                }
                            }
                        }
                        "Description" => {
                            if !value.is_empty() {
                                description = Some(value.to_string());
                            }
                        }
                        "ActiveEnterTimestamp" => {
                            // Parse timestamp and calculate uptime
                            if !value.is_empty() && value != "n/a" {
                                uptime = parse_uptime_from_timestamp(value);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Get CPU usage from /proc if we have a PID
        let cpu_usage = if let Some(p) = pid {
            get_process_cpu_usage(p).await
        } else {
            None
        };

        Ok(ServiceStatus {
            name: safe_name,
            running,
            enabled,
            uptime,
            memory_usage,
            cpu_usage,
            pid,
            description,
        })
    }

    /// Start a service
    pub async fn start(&self, service_name: &str) -> anyhow::Result<()> {
        let safe_name = sanitize_service_name(service_name)?;

        let output = AsyncCommand::new("systemctl")
            .args(["start", &safe_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start service: {}", stderr);
        }

        Ok(())
    }

    /// Stop a service
    pub async fn stop(&self, service_name: &str) -> anyhow::Result<()> {
        let safe_name = sanitize_service_name(service_name)?;

        let output = AsyncCommand::new("systemctl")
            .args(["stop", &safe_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to stop service: {}", stderr);
        }

        Ok(())
    }

    /// Restart a service
    pub async fn restart(&self, service_name: &str) -> anyhow::Result<()> {
        let safe_name = sanitize_service_name(service_name)?;

        let output = AsyncCommand::new("systemctl")
            .args(["restart", &safe_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to restart service: {}", stderr);
        }

        Ok(())
    }

    /// Enable a service (start on boot)
    pub async fn enable(&self, service_name: &str) -> anyhow::Result<()> {
        let safe_name = sanitize_service_name(service_name)?;

        let output = AsyncCommand::new("systemctl")
            .args(["enable", &safe_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to enable service: {}", stderr);
        }

        Ok(())
    }

    /// Disable a service
    pub async fn disable(&self, service_name: &str) -> anyhow::Result<()> {
        let safe_name = sanitize_service_name(service_name)?;

        let output = AsyncCommand::new("systemctl")
            .args(["disable", &safe_name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to disable service: {}", stderr);
        }

        Ok(())
    }

    /// Get logs for a service
    pub async fn get_logs(&self, service_name: &str, lines: u32) -> anyhow::Result<Vec<LogEntry>> {
        let safe_name = sanitize_service_name(service_name)?;
        let lines_str = lines.min(1000).to_string(); // Cap at 1000 lines

        let output = AsyncCommand::new("journalctl")
            .args(["-u", &safe_name, "-n", &lines_str, "--no-pager", "-o", "json"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut logs = Vec::new();

        for line in stdout.lines() {
            if line.is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = json.get("__REALTIME_TIMESTAMP")
                    .and_then(|v| v.as_str())
                    .map(|ts| {
                        // Convert microseconds to ISO timestamp
                        if let Ok(us) = ts.parse::<i64>() {
                            chrono::DateTime::from_timestamp(us / 1_000_000, ((us % 1_000_000) * 1000) as u32)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| ts.to_string())
                        } else {
                            ts.to_string()
                        }
                    })
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                let priority = json.get("PRIORITY")
                    .and_then(|v| v.as_str())
                    .unwrap_or("6");

                let level = match priority {
                    "0" | "1" | "2" | "3" => "error",
                    "4" => "warn",
                    _ => "info",
                };

                let message = json.get("MESSAGE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if !message.is_empty() {
                    logs.push(LogEntry {
                        timestamp,
                        level: level.to_string(),
                        message,
                    });
                }
            }
        }

        Ok(logs)
    }

    /// List all services (filtered by PiNAS-managed ones)
    pub async fn list_services(&self) -> anyhow::Result<Vec<ServiceStatus>> {
        let output = AsyncCommand::new("systemctl")
            .args(["list-units", "--type=service", "--all", "--no-pager", "--plain"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in stdout.lines().skip(1) {
            // Skip header line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let unit = parts[0];
                if let Some(name) = unit.strip_suffix(".service") {
                    // Filter to show only pinas-managed services or common ones
                    if name.starts_with("pinas-") ||
                       name == "docker" ||
                       name == "smbd" ||
                       name == "nmbd" {
                        if let Ok(status) = self.get_status(name).await {
                            services.push(status);
                        }
                    }
                }
            }
        }

        Ok(services)
    }
}

/// Sanitize service name to prevent command injection
fn sanitize_service_name(name: &str) -> anyhow::Result<String> {
    // Only allow alphanumeric, dash, underscore, and dot
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();

    if sanitized.is_empty() || sanitized != name {
        anyhow::bail!("Invalid service name: {}", name);
    }

    // Add .service suffix if not present
    if sanitized.ends_with(".service") {
        Ok(sanitized)
    } else {
        Ok(sanitized)
    }
}

/// Parse uptime from ActiveEnterTimestamp
fn parse_uptime_from_timestamp(timestamp: &str) -> Option<u64> {
    // Format: "Thu 2024-01-18 10:30:00 UTC"
    // Try to parse and calculate seconds since then
    use chrono::{DateTime, Utc};

    // Try various formats
    let formats = [
        "%a %Y-%m-%d %H:%M:%S %Z",
        "%Y-%m-%d %H:%M:%S %Z",
        "%a %Y-%m-%d %H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(dt) = DateTime::parse_from_str(timestamp, fmt) {
            let now = Utc::now();
            let duration = now.signed_duration_since(dt.with_timezone(&Utc));
            if duration.num_seconds() > 0 {
                return Some(duration.num_seconds() as u64);
            }
        }
    }

    None
}

/// Get CPU usage for a process (simplified)
async fn get_process_cpu_usage(pid: u32) -> Option<f64> {
    // Read /proc/[pid]/stat to get CPU time
    let stat_path = format!("/proc/{}/stat", pid);
    if let Ok(content) = tokio::fs::read_to_string(&stat_path).await {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() > 14 {
            let utime: u64 = parts[13].parse().ok()?;
            let stime: u64 = parts[14].parse().ok()?;
            let total_time = utime + stime;

            // Get system uptime
            if let Ok(uptime_str) = tokio::fs::read_to_string("/proc/uptime").await {
                let uptime: f64 = uptime_str.split_whitespace().next()?.parse().ok()?;
                let hz = 100.0; // Typical clock ticks per second
                let process_time = total_time as f64 / hz;
                let cpu_usage = (process_time / uptime) * 100.0;
                return Some((cpu_usage * 100.0).round() / 100.0);
            }
        }
    }
    None
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
