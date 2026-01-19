use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;

use crate::models::manifest::FrontendConfig;
use crate::models::package::{AppRegistryEntry, WindowConfigResponse};
use crate::AppState;

/// Create the apps router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/registry", get(get_registry))
        .route("/:id/i18n/:locale", get(get_app_translations))
}

/// Get all installed apps with window support (for frontend registry)
async fn get_registry(
    State(state): State<AppState>,
) -> Result<Json<Vec<AppRegistryEntry>>, (StatusCode, Json<serde_json::Value>)> {
    // Query all installed packages with frontend config
    let packages = sqlx::query_as::<_, (String, String, Option<String>)>(
        r#"
        SELECT id, name, frontend_config
        FROM installed_packages
        WHERE status = 'installed' AND has_window = 1 AND frontend_config IS NOT NULL
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
    })?;

    let mut entries = Vec::new();

    for (id, name, frontend_config_json) in packages {
        if let Some(config_str) = frontend_config_json {
            if let Ok(config) = serde_json::from_str::<FrontendConfig>(&config_str) {
                entries.push(AppRegistryEntry {
                    id,
                    name,
                    icon: config.icon,
                    gradient: config.gradient,
                    component: config.component,
                    window: WindowConfigResponse {
                        width: config.window.width,
                        height: config.window.height,
                        min_width: config.window.min_width,
                        min_height: config.window.min_height,
                    },
                });
            }
        }
    }

    Ok(Json(entries))
}

/// Get translations for a specific app and locale
async fn get_app_translations(
    State(state): State<AppState>,
    Path((app_id, locale)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // First try to get from app_translations table
    let translation = sqlx::query_scalar::<_, String>(
        r#"
        SELECT translations
        FROM app_translations
        WHERE package_id = ? AND locale = ?
        "#,
    )
    .bind(&app_id)
    .bind(&locale)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
    })?;

    if let Some(trans_str) = translation {
        let translations: serde_json::Value = serde_json::from_str(&trans_str).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Invalid JSON: {}", e) })),
            )
        })?;
        return Ok(Json(translations));
    }

    // If not found in table, try to get from manifest's i18n field
    let frontend_config = sqlx::query_scalar::<_, String>(
        r#"
        SELECT frontend_config
        FROM installed_packages
        WHERE id = ? AND frontend_config IS NOT NULL
        "#,
    )
    .bind(&app_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
    })?;

    if let Some(config_str) = frontend_config {
        if let Ok(config) = serde_json::from_str::<FrontendConfig>(&config_str) {
            if let Some(translations) = config.i18n.get(&locale) {
                return Ok(Json(translations.clone()));
            }
            // Try fallback to English
            if locale != "en" {
                if let Some(translations) = config.i18n.get("en") {
                    return Ok(Json(translations.clone()));
                }
            }
        }
    }

    // Return empty object if no translations found
    Ok(Json(json!({})))
}
