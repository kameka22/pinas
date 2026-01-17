use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sysinfo::System;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(get_info))
        .route("/services", get(get_services))
        .route("/reboot", post(reboot))
        .route("/shutdown", post(shutdown))
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime: u64,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub load_average: LoadAverage,
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub usage: f32,
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

/// Get system information
async fn get_info(State(_state): State<AppState>) -> impl IntoResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();

    let load_avg = System::load_average();

    let info = SystemInfo {
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        uptime: System::uptime(),
        cpu: CpuInfo {
            model: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
            cores: sys.cpus().len(),
            usage: cpu_usage,
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
    };

    Json(info)
}

/// Get services status
async fn get_services(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement actual service status check
    let services = vec![
        ServiceStatus {
            name: "samba".to_string(),
            status: "running".to_string(),
            enabled: true,
        },
        ServiceStatus {
            name: "nfs".to_string(),
            status: "stopped".to_string(),
            enabled: false,
        },
        ServiceStatus {
            name: "ssh".to_string(),
            status: "running".to_string(),
            enabled: true,
        },
    ];

    Json(services)
}

/// Reboot the system
async fn reboot(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement actual reboot
    // nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_AUTOBOOT)
    tracing::info!("Reboot requested");
    StatusCode::OK
}

/// Shutdown the system
async fn shutdown(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement actual shutdown
    // nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_POWER_OFF)
    tracing::info!("Shutdown requested");
    StatusCode::OK
}
