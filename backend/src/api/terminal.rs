use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;

use crate::AppState;

/// Request body for executing a command
#[derive(Deserialize)]
pub struct ExecRequest {
    command: String,
}

/// Response from command execution
#[derive(Serialize)]
pub struct ExecResponse {
    output: String,
    exit_code: i32,
    dev_mode: bool,
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

/// Execute a terminal command
pub async fn execute(
    State(state): State<AppState>,
    Json(req): Json<ExecRequest>,
) -> impl IntoResponse {
    let command = req.command.trim();

    // Validate command is not empty
    if command.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ExecResponse {
                output: "Error: Empty command".to_string(),
                exit_code: 1,
                dev_mode: state.config.dev_mode,
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
                dev_mode: state.config.dev_mode,
            }),
        );
    }

    // In dev mode, don't execute commands
    if state.config.dev_mode {
        return (
            StatusCode::OK,
            Json(ExecResponse {
                output: format!("⚠️ Mode développement: commande non exécutée\n$ {}", command),
                exit_code: 0,
                dev_mode: true,
            }),
        );
    }

    // Execute the command with timeout
    match execute_command(command).await {
        Ok((output, exit_code)) => (
            StatusCode::OK,
            Json(ExecResponse {
                output,
                exit_code,
                dev_mode: false,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExecResponse {
                output: format!("Error: {}", e),
                exit_code: 1,
                dev_mode: false,
            }),
        ),
    }
}

/// Execute a shell command with timeout
async fn execute_command(command: &str) -> Result<(String, i32), String> {
    let timeout = Duration::from_secs(30);

    let result = tokio::time::timeout(timeout, async {
        Command::new("sh")
            .arg("-c")
            .arg(command)
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
