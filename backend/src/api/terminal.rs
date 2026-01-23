use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;

use crate::AppState;

/// Virtual root shown to frontend (always /storage)
const VIRTUAL_ROOT: &str = "/storage";

/// Real root in production
const PROD_ROOT: &str = "/storage";

/// Real root in dev mode (relative to cwd)
const DEV_ROOT: &str = "data";

/// Request body for executing a command
#[derive(Deserialize)]
pub struct ExecRequest {
    command: String,
    #[serde(default = "default_cwd")]
    cwd: String,
}

fn default_cwd() -> String {
    VIRTUAL_ROOT.to_string()
}

/// Response from command execution
#[derive(Serialize)]
pub struct ExecResponse {
    output: String,
    exit_code: i32,
    dev_mode: bool,
    cwd: String,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Dangerous commands/patterns that should be blocked
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "rm -fr /",
    "rm -fr /*",
    "mkfs",
    "dd if=",
    ":(){:|:&};:",
    "shutdown",
    "reboot",
    "poweroff",
    "halt",
    "init 0",
    "init 6",
    "> /dev/sda",
    "chmod -R 777 /",
    "chown -R",
    "mv /* ",
    "wget | sh",
    "curl | sh",
    "wget | bash",
    "curl | bash",
];

/// Check if a command contains blocked patterns
fn is_command_blocked(command: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    BLOCKED_PATTERNS.iter().any(|pattern| cmd_lower.contains(pattern))
}

/// Get the real root directory based on dev_mode
fn get_real_root(dev_mode: bool) -> PathBuf {
    if dev_mode {
        // In dev mode, use ./data relative to current working directory
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(DEV_ROOT)
    } else {
        PathBuf::from(PROD_ROOT)
    }
}

/// Convert virtual path (/storage/...) to real path
fn virtual_to_real(virtual_path: &str, real_root: &PathBuf) -> PathBuf {
    if virtual_path == VIRTUAL_ROOT {
        real_root.clone()
    } else if let Some(suffix) = virtual_path.strip_prefix(&format!("{}/", VIRTUAL_ROOT)) {
        real_root.join(suffix)
    } else {
        // Path doesn't start with /storage, treat as relative to root
        real_root.join(virtual_path.trim_start_matches('/'))
    }
}

/// Convert real path to virtual path (/storage/...)
fn real_to_virtual(real_path: &PathBuf, real_root: &PathBuf) -> String {
    if let Ok(canonical_real) = real_path.canonicalize() {
        if let Ok(canonical_root) = real_root.canonicalize() {
            if canonical_real == canonical_root {
                return VIRTUAL_ROOT.to_string();
            }
            if let Ok(suffix) = canonical_real.strip_prefix(&canonical_root) {
                return format!("{}/{}", VIRTUAL_ROOT, suffix.to_string_lossy());
            }
        }
    }
    // Fallback to virtual root if path is outside
    VIRTUAL_ROOT.to_string()
}

/// Check if a path is within the allowed root
fn is_path_within_root(path: &PathBuf, root: &PathBuf) -> bool {
    if let (Ok(canonical_path), Ok(canonical_root)) = (path.canonicalize(), root.canonicalize()) {
        canonical_path.starts_with(&canonical_root)
    } else {
        false
    }
}

/// Execute a terminal command
pub async fn execute(
    State(state): State<AppState>,
    Json(req): Json<ExecRequest>,
) -> impl IntoResponse {
    let command = req.command.trim();
    let dev_mode = state.config.dev_mode;
    let real_root = get_real_root(dev_mode);

    // Ensure real root exists
    if !real_root.exists() {
        let _ = std::fs::create_dir_all(&real_root);
    }

    // Convert virtual cwd to real cwd
    let real_cwd = virtual_to_real(&req.cwd, &real_root);

    // Ensure cwd exists and is within root, fallback to root
    let real_cwd = if real_cwd.exists() && is_path_within_root(&real_cwd, &real_root) {
        real_cwd
    } else {
        real_root.clone()
    };

    let virtual_cwd = real_to_virtual(&real_cwd, &real_root);

    // Validate command is not empty
    if command.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ExecResponse {
                output: "Error: Empty command".to_string(),
                exit_code: 1,
                dev_mode,
                cwd: virtual_cwd,
            }),
        );
    }

    // Check for blocked commands
    if is_command_blocked(command) {
        return (
            StatusCode::FORBIDDEN,
            Json(ExecResponse {
                output: "❌ Commande non autorisée pour des raisons de sécurité".to_string(),
                exit_code: 1,
                dev_mode,
                cwd: virtual_cwd,
            }),
        );
    }

    // Handle cd command specially
    if command == "cd" || command.starts_with("cd ") {
        let new_virtual_cwd = handle_cd_command(command, &virtual_cwd, &real_root);
        return (
            StatusCode::OK,
            Json(ExecResponse {
                output: String::new(),
                exit_code: 0,
                dev_mode,
                cwd: new_virtual_cwd,
            }),
        );
    }

    // Execute the command with timeout
    match execute_command(command, &real_cwd).await {
        Ok((output, exit_code)) => (
            StatusCode::OK,
            Json(ExecResponse {
                output,
                exit_code,
                dev_mode,
                cwd: virtual_cwd,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExecResponse {
                output: format!("Error: {}", e),
                exit_code: 1,
                dev_mode,
                cwd: virtual_cwd,
            }),
        ),
    }
}

/// Handle cd command and return new virtual working directory
fn handle_cd_command(command: &str, current_virtual_cwd: &str, real_root: &PathBuf) -> String {
    let target = if command == "cd" {
        // cd without argument goes to root
        String::new()
    } else {
        // Extract path after "cd "
        command.strip_prefix("cd ").unwrap_or("").trim().to_string()
    };

    // Calculate new virtual path
    let new_virtual_path = if target.is_empty() || target == "~" {
        VIRTUAL_ROOT.to_string()
    } else if target == "-" {
        // cd - not supported, stay in current
        return current_virtual_cwd.to_string();
    } else if target.starts_with('/') {
        // Absolute path - ensure it starts with /storage
        if target.starts_with(VIRTUAL_ROOT) {
            target
        } else {
            // Trying to go outside /storage, redirect to /storage
            VIRTUAL_ROOT.to_string()
        }
    } else if target.starts_with("~/") {
        // Home-relative path
        format!("{}/{}", VIRTUAL_ROOT, target.strip_prefix("~/").unwrap())
    } else {
        // Relative path
        if current_virtual_cwd == VIRTUAL_ROOT {
            format!("{}/{}", VIRTUAL_ROOT, target)
        } else {
            format!("{}/{}", current_virtual_cwd, target)
        }
    };

    // Convert to real path to verify it exists and is within root
    let real_path = virtual_to_real(&new_virtual_path, real_root);

    // Canonicalize to resolve .. and . and verify within root
    match real_path.canonicalize() {
        Ok(canonical) => {
            if is_path_within_root(&canonical, real_root) {
                real_to_virtual(&canonical, real_root)
            } else {
                // Trying to escape root, stay at root
                VIRTUAL_ROOT.to_string()
            }
        }
        Err(_) => current_virtual_cwd.to_string(), // Path doesn't exist, stay in current
    }
}

/// Execute a shell command with timeout in specified directory
async fn execute_command(command: &str, cwd: &PathBuf) -> Result<(String, i32), String> {
    let timeout = Duration::from_secs(30);

    let result = tokio::time::timeout(timeout, async {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .output()
            .await
    })
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Combine stdout and stderr
            let combined = if stderr.is_empty() {
                stdout.to_string()
            } else if stdout.is_empty() {
                stderr.to_string()
            } else {
                format!("{}{}", stdout, stderr)
            };

            let exit_code = output.status.code().unwrap_or(-1);
            Ok((combined, exit_code))
        }
        Ok(Err(e)) => Err(format!("Failed to execute command: {}", e)),
        Err(_) => Err("Command timed out (30s limit)".to_string()),
    }
}

/// Create the terminal router
pub fn router() -> Router<AppState> {
    Router::new().route("/exec", post(execute))
}
