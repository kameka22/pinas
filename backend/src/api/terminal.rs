use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::process::Command;

use crate::api::middleware::AdminUser;
use crate::AppState;

/// Virtual root shown to frontend (always /storage)
const VIRTUAL_ROOT: &str = "/storage";

/// Real root in production
const PROD_ROOT: &str = "/storage";

/// Real root in dev mode (relative to cwd)
const DEV_ROOT: &str = "data";

/// Rate limiting: max commands per window
const RATE_LIMIT_MAX: usize = 30;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

/// Rate limiting for completion (lighter)
const COMPLETE_RATE_LIMIT_MAX: usize = 60;
const COMPLETE_RATE_LIMIT_WINDOW_SECS: u64 = 60;

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

/// Request body for tab completion
#[derive(Deserialize)]
pub struct CompleteRequest {
    partial: String,
    #[serde(default = "default_cwd")]
    cwd: String,
}

/// A single completion match
#[derive(Serialize)]
pub struct CompleteMatch {
    name: String,
    is_dir: bool,
}

/// Response from tab completion
#[derive(Serialize)]
pub struct CompleteResponse {
    matches: Vec<CompleteMatch>,
    common_prefix: String,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Rate limiter state (per user)
static RATE_LIMITER: std::sync::LazyLock<Mutex<HashMap<String, Vec<Instant>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Rate limiter for completion (separate, lighter)
static COMPLETE_RATE_LIMITER: std::sync::LazyLock<Mutex<HashMap<String, Vec<Instant>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Check rate limit for a user. Returns true if allowed.
fn check_rate_limit(user_id: &str) -> bool {
    let mut map = RATE_LIMITER.lock().unwrap();
    let now = Instant::now();
    let window = Duration::from_secs(RATE_LIMIT_WINDOW_SECS);

    let timestamps = map.entry(user_id.to_string()).or_default();

    // Remove expired entries
    timestamps.retain(|t| now.duration_since(*t) < window);

    if timestamps.len() >= RATE_LIMIT_MAX {
        return false;
    }

    timestamps.push(now);
    true
}

/// Check rate limit for completion. Returns true if allowed.
fn check_complete_rate_limit(user_id: &str) -> bool {
    let mut map = COMPLETE_RATE_LIMITER.lock().unwrap();
    let now = Instant::now();
    let window = Duration::from_secs(COMPLETE_RATE_LIMIT_WINDOW_SECS);

    let timestamps = map.entry(user_id.to_string()).or_default();
    timestamps.retain(|t| now.duration_since(*t) < window);

    if timestamps.len() >= COMPLETE_RATE_LIMIT_MAX {
        return false;
    }

    timestamps.push(now);
    true
}

/// Dangerous commands/patterns that should be blocked
const BLOCKED_PATTERNS: &[&str] = &[
    // Filesystem destruction
    "rm -rf /",
    "rm -rf /*",
    "rm -fr /",
    "rm -fr /*",
    "rm -rf --no-preserve-root",
    "mkfs",
    "dd if=",
    "dd of=/dev/",
    "> /dev/sd",
    "> /dev/nvme",
    "> /dev/mmcblk",
    "> /dev/vd",
    "mv /* ",
    "find / -delete",
    "find / -exec rm",
    // Fork bomb and system overload
    ":(){:|:&};:",
    ":(){ :|:& };:",
    // System control (managed via PiNAS UI)
    "shutdown",
    "reboot",
    "poweroff",
    "halt",
    "init 0",
    "init 6",
    "systemctl stop pinas",
    "systemctl disable pinas",
    // Privilege and permission abuse
    "chmod -R 777 /",
    "chmod u+s",
    "chmod g+s",
    "chown -R",
    "usermod",
    "useradd",
    "userdel",
    "groupmod",
    "groupadd",
    "groupdel",
    "visudo",
    "passwd",
    // Remote code execution / exfiltration
    "wget | sh",
    "curl | sh",
    "wget | bash",
    "curl | bash",
    "| sh",
    "| bash",
    "| zsh",
    // Reverse shells
    "nc -e",
    "nc -c",
    "ncat -e",
    "ncat -c",
    "/dev/tcp/",
    "/dev/udp/",
    // Persistence
    "crontab",
    "at ",
    "at\t",
    // Firewall/network tampering
    "iptables",
    "ip6tables",
    "nftables",
    "ufw ",
    // Kernel/module manipulation
    "insmod",
    "rmmod",
    "modprobe",
    // Boot/mount manipulation
    "mount -o remount",
    "umount /flash",
    "umount /storage",
];

/// Check if a command contains blocked patterns
fn is_command_blocked(command: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    BLOCKED_PATTERNS
        .iter()
        .any(|pattern| cmd_lower.contains(&pattern.to_lowercase()))
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

/// Execute a terminal command (admin only)
pub async fn execute(
    State(state): State<AppState>,
    admin: AdminUser,
    Json(req): Json<ExecRequest>,
) -> impl IntoResponse {
    let command = req.command.trim();
    let dev_mode = state.config.dev_mode;
    let real_root = get_real_root(dev_mode);

    // Rate limiting
    if !check_rate_limit(&admin.id) {
        tracing::warn!(
            user = %admin.username,
            "Terminal rate limit exceeded"
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ExecResponse {
                output: "Rate limit exceeded. Please wait before executing more commands."
                    .to_string(),
                exit_code: 1,
                dev_mode,
                cwd: req.cwd.clone(),
            }),
        );
    }

    // Audit log
    tracing::info!(
        user = %admin.username,
        user_id = %admin.id,
        command = %command,
        cwd = %req.cwd,
        "Terminal command executed"
    );

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
        tracing::warn!(
            user = %admin.username,
            command = %command,
            "Blocked dangerous terminal command"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(ExecResponse {
                output: "Command blocked for security reasons".to_string(),
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
        Ok((output, exit_code)) => {
            if exit_code != 0 {
                tracing::debug!(
                    user = %admin.username,
                    command = %command,
                    exit_code = exit_code,
                    "Terminal command failed"
                );
            }
            (
                StatusCode::OK,
                Json(ExecResponse {
                    output,
                    exit_code,
                    dev_mode,
                    cwd: virtual_cwd,
                }),
            )
        }
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
        format!(
            "{}/{}",
            VIRTUAL_ROOT,
            target.strip_prefix("~/").unwrap()
        )
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

/// Tab-complete file/directory names (admin only)
pub async fn complete(
    State(state): State<AppState>,
    admin: AdminUser,
    Json(req): Json<CompleteRequest>,
) -> impl IntoResponse {
    let dev_mode = state.config.dev_mode;
    let real_root = get_real_root(dev_mode);

    // Rate limiting
    if !check_complete_rate_limit(&admin.id) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(CompleteResponse {
                matches: vec![],
                common_prefix: String::new(),
            }),
        );
    }

    // Ensure real root exists
    if !real_root.exists() {
        let _ = std::fs::create_dir_all(&real_root);
    }

    let partial = &req.partial;

    // Determine the directory to list and the prefix to filter by
    let (search_dir_virtual, prefix) = if partial.is_empty() {
        // Empty partial: list current directory
        (req.cwd.clone(), String::new())
    } else if partial.starts_with('/') {
        // Absolute path
        let p = Path::new(partial);
        if partial.ends_with('/') {
            // e.g. "/storage/foo/" -> list that dir, no prefix filter
            (partial.to_string(), String::new())
        } else if let Some(parent) = p.parent() {
            let file_part = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            (parent.to_string_lossy().to_string(), file_part)
        } else {
            (partial.to_string(), String::new())
        }
    } else {
        // Relative path
        let p = Path::new(partial);
        if partial.ends_with('/') {
            // e.g. "shares/" -> list cwd/shares/
            let full = if req.cwd == VIRTUAL_ROOT {
                format!("{}/{}", VIRTUAL_ROOT, partial.trim_end_matches('/'))
            } else {
                format!("{}/{}", req.cwd, partial.trim_end_matches('/'))
            };
            (full, String::new())
        } else if partial.contains('/') {
            // e.g. "shares/med" -> dir=cwd/shares, prefix=med
            if let Some(parent) = p.parent() {
                let file_part = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                let full_parent = if req.cwd == VIRTUAL_ROOT {
                    format!("{}/{}", VIRTUAL_ROOT, parent.to_string_lossy())
                } else {
                    format!("{}/{}", req.cwd, parent.to_string_lossy())
                };
                (full_parent, file_part)
            } else {
                (req.cwd.clone(), partial.to_string())
            }
        } else {
            // Simple name, e.g. "sha" -> list cwd, prefix=sha
            (req.cwd.clone(), partial.to_string())
        }
    };

    // Convert virtual directory to real path
    let real_dir = virtual_to_real(&search_dir_virtual, &real_root);

    // Verify the directory exists and is within root
    if !real_dir.exists() || !real_dir.is_dir() || !is_path_within_root(&real_dir, &real_root) {
        return (
            StatusCode::OK,
            Json(CompleteResponse {
                matches: vec![],
                common_prefix: String::new(),
            }),
        );
    }

    // Read directory entries and filter by prefix
    let mut matches: Vec<CompleteMatch> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&real_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files unless prefix starts with '.'
            if name.starts_with('.') && !prefix.starts_with('.') {
                continue;
            }

            if prefix.is_empty() || name.starts_with(&prefix) {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                matches.push(CompleteMatch { name, is_dir });
            }
        }
    }

    // Sort matches alphabetically (dirs first, then files)
    matches.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    // Calculate longest common prefix among all match names
    let common_prefix = if matches.is_empty() {
        String::new()
    } else if matches.len() == 1 {
        matches[0].name.clone()
    } else {
        let first = &matches[0].name;
        let mut common_len = first.len();
        for m in &matches[1..] {
            common_len = first
                .chars()
                .zip(m.name.chars())
                .take(common_len)
                .take_while(|(a, b)| a == b)
                .count();
        }
        first[..common_len].to_string()
    };

    (
        StatusCode::OK,
        Json(CompleteResponse {
            matches,
            common_prefix,
        }),
    )
}

/// Create the terminal router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/exec", post(execute))
        .route("/complete", post(complete))
}
