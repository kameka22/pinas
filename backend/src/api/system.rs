use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sysinfo::{System, Signal, Pid};

use crate::AppState;
use crate::api::middleware::AdminUser;
use crate::services::share::ShareService;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(get_info))
        .route("/services", get(get_services))
        .route("/processes", get(get_processes))
        .route("/processes/{pid}/kill", post(kill_process))
        .route("/reboot", post(reboot))
        .route("/shutdown", post(shutdown))
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub version: String,
    pub hostname: String,
    /// Board model from /proc/device-tree/model (e.g. "Raspberry Pi 5 Model B Rev 1.0")
    pub model: Option<String>,
    /// SoC serial from /proc/cpuinfo
    pub serial: Option<String>,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime: u64,
    /// Unix timestamp of the last boot
    pub boot_time: u64,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub load_average: LoadAverage,
    pub dev_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub usage: f32,
    pub frequency_mhz: u64,
    /// Celsius, from /sys/class/thermal/thermal_zone0
    pub temperature: Option<f32>,
}

fn read_device_model() -> Option<String> {
    std::fs::read_to_string("/proc/device-tree/model")
        .ok()
        .map(|s| s.trim_end_matches('\0').trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_cpu_serial() -> Option<String> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    cpuinfo
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.trim() == "Serial")
        .map(|(_, value)| value.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_cpu_temperature() -> Option<f32> {
    std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()?
        .trim()
        .parse::<f32>()
        .ok()
        .map(|millidegrees| millidegrees / 1000.0)
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu: f32,
    pub memory: u64,
    pub memory_percent: f32,
    pub status: String,
    pub command: String,
    pub start_time: u64,
}

#[derive(Debug, Serialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
    pub total_processes: usize,
    pub running_processes: usize,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
}

/// Get system information
async fn get_info(State(state): State<AppState>) -> impl IntoResponse {
    let sys = state.system.read().await;

    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();

    let load_avg = System::load_average();

    let (model, serial, temperature) = if state.config.dev_mode {
        (
            Some("Raspberry Pi 5 Model B (simulated)".to_string()),
            Some("10000000deadbeef".to_string()),
            Some(45.0),
        )
    } else {
        (read_device_model(), read_cpu_serial(), read_cpu_temperature())
    };

    let info = SystemInfo {
        version: include_str!("../../../VERSION").trim().to_string(),
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        model,
        serial,
        os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        uptime: System::uptime(),
        boot_time: System::boot_time(),
        cpu: CpuInfo {
            model: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
            cores: sys.cpus().len(),
            usage: cpu_usage,
            frequency_mhz: sys.cpus().first().map(|c| c.frequency()).unwrap_or(0),
            temperature,
        },
        memory: MemoryInfo {
            total: total_memory,
            used: used_memory,
            available: available_memory,
            usage_percent: (used_memory as f32 / total_memory as f32) * 100.0,
        },
        load_average: LoadAverage {
            one: load_avg.one,
            five: load_avg.five,
            fifteen: load_avg.fifteen,
        },
        dev_mode: state.config.dev_mode,
    };

    Json(info)
}

/// Get services status
async fn get_services(State(state): State<AppState>) -> impl IntoResponse {
    let mut services = Vec::new();

    // Get real Samba status
    let share_svc = ShareService::new(state.db.clone());
    if let Ok(samba) = share_svc.get_samba_status().await {
        services.push(ServiceStatus {
            name: "samba".to_string(),
            status: if samba.running { "running".to_string() } else { "stopped".to_string() },
            enabled: samba.enabled,
        });
    }

    if let Ok(nfs) = share_svc.get_nfs_status().await {
        services.push(ServiceStatus {
            name: "nfs".to_string(),
            status: if nfs.running { "running".to_string() } else { "stopped".to_string() },
            enabled: nfs.enabled,
        });
    }
    services.push(ServiceStatus {
        name: "ssh".to_string(),
        status: "running".to_string(),
        enabled: true,
    });

    Json(services)
}

/// Reboot the system (admin only)
async fn reboot(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("Reboot requested");
    if state.config.dev_mode {
        tracing::info!("[DEV] Reboot skipped (dev_mode)");
        return StatusCode::OK;
    }
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let _ = tokio::process::Command::new("systemctl")
            .arg("reboot")
            .status()
            .await;
    });
    StatusCode::OK
}

/// Shutdown the system (admin only)
async fn shutdown(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("Shutdown requested");
    if state.config.dev_mode {
        tracing::info!("[DEV] Shutdown skipped (dev_mode)");
        return StatusCode::OK;
    }
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let _ = tokio::process::Command::new("systemctl")
            .arg("poweroff")
            .status()
            .await;
    });
    StatusCode::OK
}

/// Get list of running processes
async fn get_processes(State(state): State<AppState>, _admin: AdminUser) -> impl IntoResponse {
    // The shared snapshot is refreshed every 2s, so CPU deltas are already meaningful
    let sys = state.system.read().await;

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let cpu_usage = sys.global_cpu_usage();
    let memory_usage = (used_memory as f32 / total_memory as f32) * 100.0;

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let status = match process.status() {
                sysinfo::ProcessStatus::Run => "running",
                sysinfo::ProcessStatus::Sleep => "sleeping",
                sysinfo::ProcessStatus::Stop => "stopped",
                sysinfo::ProcessStatus::Zombie => "zombie",
                sysinfo::ProcessStatus::Idle => "idle",
                _ => "unknown",
            };

            // Get user name from user_id
            let user = process
                .user_id()
                .map(|uid| {
                    sysinfo::Users::new_with_refreshed_list()
                        .iter()
                        .find(|u| u.id() == uid)
                        .map(|u| u.name().to_string())
                        .unwrap_or_else(|| format!("{}", **uid))
                })
                .unwrap_or_else(|| "unknown".to_string());

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                user,
                cpu: process.cpu_usage(),
                memory: process.memory(),
                memory_percent: (process.memory() as f32 / total_memory as f32) * 100.0,
                status: status.to_string(),
                command: process.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" "),
                start_time: process.start_time(),
            }
        })
        .collect();

    // Sort by CPU usage descending
    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));

    let total_processes = processes.len();
    let running_processes = processes.iter().filter(|p| p.status == "running").count();

    Json(ProcessListResponse {
        processes,
        total_processes,
        running_processes,
        cpu_usage,
        memory_usage,
        total_memory,
        used_memory,
    })
}

/// Kill a process by PID
async fn kill_process(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(pid): Path<u32>,
) -> impl IntoResponse {
    let pid = Pid::from_u32(pid);
    let mut sys = state.system.write().await;
    // Make sure we act on the current process table, not a 2s-old snapshot
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

    if let Some(process) = sys.process(pid) {
        // Default to SIGTERM for graceful termination
        if process.kill_with(Signal::Term).unwrap_or(false) {
            tracing::info!("Process {} killed successfully", pid);
            return (StatusCode::OK, Json(serde_json::json!({"success": true, "message": "Process terminated"})));
        } else {
            tracing::warn!("Failed to kill process {}", pid);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "message": "Failed to kill process"})));
        }
    }

    (StatusCode::NOT_FOUND, Json(serde_json::json!({"success": false, "message": "Process not found"})))
}
