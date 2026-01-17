use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::AppState;

/// File or folder item
#[derive(Debug, Serialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: Option<u64>,
    pub modified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Query parameters for listing files
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
}

/// Request to create a folder
#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: String,
}

/// Request to delete a file/folder
#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub path: String,
}

/// Request to rename a file/folder
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub path: String,
    pub new_name: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Create the files router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_files))
        .route("/folder", post(create_folder))
        .route("/", delete(delete_file))
        .route("/rename", patch(rename_file))
}

/// Validate that a path stays within the base directory (prevent path traversal)
fn validate_path(base: &Path, requested: &str) -> Result<PathBuf, String> {
    // Normalize the requested path - remove leading slashes
    let requested = requested.trim_start_matches('/');

    // Build the full path
    let full_path = base.join(requested);

    // For new paths that don't exist yet, we check the parent
    let check_path = if full_path.exists() {
        full_path.canonicalize()
    } else {
        // For non-existent paths, canonicalize the parent and append the filename
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                parent.canonicalize().map(|p| p.join(full_path.file_name().unwrap_or_default()))
            } else {
                // Parent doesn't exist either - try to canonicalize what we can
                Ok(full_path.clone())
            }
        } else {
            Ok(full_path.clone())
        }
    };

    let canonical = check_path.map_err(|e| format!("Invalid path: {}", e))?;

    let base_canonical = base.canonicalize()
        .map_err(|e| format!("Base path error: {}", e))?;

    // Check that the path starts with the base path
    if !canonical.starts_with(&base_canonical) {
        return Err("Access denied: path outside allowed directory".to_string());
    }

    Ok(full_path)
}

/// Get MIME type from file extension
fn get_mime_type(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_lowercase();

    let mime = match extension.as_str() {
        // Text
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "md" => "text/markdown",

        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",

        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",

        _ => return None,
    };

    Some(mime.to_string())
}

/// Format system time to ISO 8601 string
fn format_time(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    datetime.to_rfc3339()
}

/// List files in a directory
async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let base_path = PathBuf::from(&state.config.files_root);

    // Ensure base directory exists
    if !base_path.exists() {
        if let Err(e) = fs::create_dir_all(&base_path) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("Failed to create files directory: {}", e) }),
            ).into_response();
        }
    }

    let rel_path = query.path.unwrap_or_default();

    // Validate path
    let full_path = match validate_path(&base_path, &rel_path) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };

    // Check if path exists and is a directory
    if !full_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "Directory not found".to_string() }),
        ).into_response();
    }

    if !full_path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Path is not a directory".to_string() }),
        ).into_response();
    }

    // Read directory entries
    let entries = match fs::read_dir(&full_path) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: format!("Failed to read directory: {}", e) }),
            ).into_response();
        }
    };

    let mut files: Vec<FileItem> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files (starting with .)
        if name.starts_with('.') {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };

        let modified = metadata.modified()
            .map(format_time)
            .unwrap_or_else(|_| "".to_string());

        // Build relative path from files_root
        let item_rel_path = if rel_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel_path.trim_end_matches('/'), name)
        };

        let mime_type = if is_dir { None } else { get_mime_type(&path) };

        files.push(FileItem {
            name,
            path: item_rel_path,
            file_type: if is_dir { "folder".to_string() } else { "file".to_string() },
            size,
            modified,
            mime_type,
        });
    }

    // Sort: folders first, then by name
    files.sort_by(|a, b| {
        match (&a.file_type[..], &b.file_type[..]) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Json(files).into_response()
}

/// Create a new folder
async fn create_folder(
    State(state): State<AppState>,
    Json(payload): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    let base_path = PathBuf::from(&state.config.files_root);

    // Validate parent path
    let parent_path = match validate_path(&base_path, &payload.path) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };

    // Validate folder name (no path separators)
    if payload.name.contains('/') || payload.name.contains('\\') || payload.name.starts_with('.') {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid folder name".to_string() }),
        ).into_response();
    }

    let new_folder_path = parent_path.join(&payload.name);

    // Check if already exists
    if new_folder_path.exists() {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse { error: "A file or folder with this name already exists".to_string() }),
        ).into_response();
    }

    // Create the folder
    if let Err(e) = fs::create_dir(&new_folder_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to create folder: {}", e) }),
        ).into_response();
    }

    // Build relative path
    let rel_path = if payload.path.is_empty() {
        payload.name.clone()
    } else {
        format!("{}/{}", payload.path.trim_end_matches('/'), payload.name)
    };

    let modified = SystemTime::now();

    (
        StatusCode::CREATED,
        Json(FileItem {
            name: payload.name,
            path: rel_path,
            file_type: "folder".to_string(),
            size: None,
            modified: format_time(modified),
            mime_type: None,
        }),
    ).into_response()
}

/// Delete a file or folder
async fn delete_file(
    State(state): State<AppState>,
    Query(query): Query<DeleteQuery>,
) -> impl IntoResponse {
    let base_path = PathBuf::from(&state.config.files_root);

    // Validate path
    let full_path = match validate_path(&base_path, &query.path) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };

    // Check if exists
    if !full_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "File or folder not found".to_string() }),
        ).into_response();
    }

    // Don't allow deleting the root
    if full_path.canonicalize().ok() == base_path.canonicalize().ok() {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse { error: "Cannot delete root directory".to_string() }),
        ).into_response();
    }

    // Delete
    let result = if full_path.is_dir() {
        fs::remove_dir_all(&full_path)
    } else {
        fs::remove_file(&full_path)
    };

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to delete: {}", e) }),
        ).into_response(),
    }
}

/// Rename a file or folder
async fn rename_file(
    State(state): State<AppState>,
    Json(payload): Json<RenameRequest>,
) -> impl IntoResponse {
    let base_path = PathBuf::from(&state.config.files_root);

    // Validate original path
    let full_path = match validate_path(&base_path, &payload.path) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            ).into_response();
        }
    };

    // Check if exists
    if !full_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: "File or folder not found".to_string() }),
        ).into_response();
    }

    // Validate new name
    if payload.new_name.contains('/') || payload.new_name.contains('\\') || payload.new_name.starts_with('.') {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid name".to_string() }),
        ).into_response();
    }

    // Build new path
    let parent = match full_path.parent() {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: "Cannot rename root".to_string() }),
            ).into_response();
        }
    };

    let new_path = parent.join(&payload.new_name);

    // Check if destination exists
    if new_path.exists() {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse { error: "A file or folder with this name already exists".to_string() }),
        ).into_response();
    }

    // Rename
    if let Err(e) = fs::rename(&full_path, &new_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: format!("Failed to rename: {}", e) }),
        ).into_response();
    }

    // Build new relative path
    let parent_rel = PathBuf::from(&payload.path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let new_rel_path = if parent_rel.is_empty() {
        payload.new_name.clone()
    } else {
        format!("{}/{}", parent_rel, payload.new_name)
    };

    let metadata = new_path.metadata().ok();
    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
    let size = if is_dir { None } else { metadata.as_ref().map(|m| m.len()) };
    let modified = metadata
        .and_then(|m| m.modified().ok())
        .map(format_time)
        .unwrap_or_default();

    Json(FileItem {
        name: payload.new_name,
        path: new_rel_path,
        file_type: if is_dir { "folder".to_string() } else { "file".to_string() },
        size,
        modified,
        mime_type: if is_dir { None } else { get_mime_type(&new_path) },
    }).into_response()
}
