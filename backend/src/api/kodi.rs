use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthUser;
use crate::services::kodi::{KodiService, MediaSource};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Status & Info
        .route("/status", get(get_status))
        .route("/info", get(get_info))
        // Playback control
        .route("/playback/play-pause", post(play_pause))
        .route("/playback/stop", post(stop))
        .route("/playback/volume", get(get_volume).post(set_volume))
        // Input/Navigation
        .route("/input/:action", post(input_action))
        // Media Sources
        .route("/sources", get(get_sources).post(add_source))
        .route("/sources/:id", delete(remove_source))
        // Settings
        .route("/settings", get(get_settings))
        .route("/settings/:setting_id", put(set_setting))
        // Addons
        .route("/addons", get(get_addons))
        .route("/addons/:addon_id/enable", post(enable_addon))
        .route("/addons/:addon_id/disable", post(disable_addon))
        // Library
        .route("/library/:library_type/scan", post(scan_library))
        .route("/library/:library_type/clean", post(clean_library))
        // System
        .route("/notification", post(send_notification))
        .route("/reboot", post(reboot))
        .route("/shutdown", post(shutdown))
}

// === Status & Info ===

async fn get_status(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_status().await {
        Ok(status) => (StatusCode::OK, Json(status)).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_info(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_info().await {
        Ok(info) => (StatusCode::OK, Json(info)).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Playback Control ===

async fn play_pause(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.play_pause().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn stop(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.stop().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_volume(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_volume().await {
        Ok(volume) => (StatusCode::OK, Json(serde_json::json!({ "volume": volume }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SetVolumeRequest {
    volume: u8,
}

async fn set_volume(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<SetVolumeRequest>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.set_volume(body.volume).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "volume": body.volume }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Input/Navigation ===

async fn input_action(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(action): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.input_action(&action).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Media Sources ===

#[derive(Deserialize)]
struct SourcesQuery {
    #[serde(rename = "type")]
    source_type: Option<String>,
}

async fn get_sources(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SourcesQuery>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_sources(query.source_type.as_deref()).await {
        Ok(sources) => (StatusCode::OK, Json(sources)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct AddSourceRequest {
    name: String,
    path: String,
    source_type: String,
}

async fn add_source(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<AddSourceRequest>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    // Detect protocol from path
    let protocol = if body.path.starts_with("smb://") {
        Some("smb".to_string())
    } else if body.path.starts_with("nfs://") {
        Some("nfs".to_string())
    } else {
        Some("local".to_string())
    };

    let source = MediaSource {
        id: uuid::Uuid::new_v4().to_string(),
        name: body.name,
        path: body.path,
        source_type: body.source_type,
        protocol,
    };

    match kodi.add_source(&source).await {
        Ok(_) => (StatusCode::CREATED, Json(source)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn remove_source(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(source_id): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.remove_source(&source_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Settings ===

#[derive(Deserialize)]
struct SettingsQuery {
    category: Option<String>,
}

async fn get_settings(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<SettingsQuery>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_settings(query.category.as_deref()).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SetSettingRequest {
    value: serde_json::Value,
}

async fn set_setting(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(setting_id): Path<String>,
    Json(body): Json<SetSettingRequest>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.set_setting(&setting_id, body.value).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Addons ===

async fn get_addons(
    State(state): State<AppState>,
    _user: AuthUser,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.get_addons().await {
        Ok(addons) => (StatusCode::OK, Json(addons)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn enable_addon(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(addon_id): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.set_addon_enabled(&addon_id, true).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn disable_addon(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(addon_id): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.set_addon_enabled(&addon_id, false).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === Library ===

async fn scan_library(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(library_type): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.scan_library(&library_type).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "message": "Library scan started" }))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn clean_library(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(library_type): Path<String>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.clean_library(&library_type).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "message": "Library clean started" }))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// === System ===

#[derive(Deserialize)]
struct NotificationRequest {
    title: String,
    message: String,
}

async fn send_notification(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<NotificationRequest>,
) -> impl IntoResponse {
    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.send_notification(&body.title, &body.message).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn reboot(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    // Only admin can reboot
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin permission required" })),
        )
            .into_response();
    }

    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.reboot().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "message": "Reboot initiated" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn shutdown(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    // Only admin can shutdown
    if !user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin permission required" })),
        )
            .into_response();
    }

    let kodi = KodiService::new(state.config.dev_mode, state.config.kodi_username.clone(), state.config.kodi_password.clone());

    match kodi.shutdown().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "message": "Shutdown initiated" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
