use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthUser;
use crate::AppState;

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct PreferenceEntry {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct SetPreferenceRequest {
    value: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_preferences))
        .route("/{key}", get(get_preference).put(set_preference))
}

/// Get all preferences for the authenticated user
async fn get_all_preferences(
    State(state): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PreferenceEntry>(
        "SELECT key, value FROM user_preferences WHERE user_id = ?",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(entries) => {
            let map: std::collections::HashMap<String, String> =
                entries.into_iter().map(|e| (e.key, e.value)).collect();
            Json(map).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get preferences: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get a single preference by key
async fn get_preference(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let result: Result<Option<String>, _> = sqlx::query_scalar(
        "SELECT value FROM user_preferences WHERE user_id = ? AND key = ?",
    )
    .bind(user.id)
    .bind(&key)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(value)) => Json(serde_json::json!({ "value": value })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                message: format!("Preference '{}' not found", key),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get preference: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Set a preference value (INSERT OR REPLACE)
async fn set_preference(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
    Json(payload): Json<SetPreferenceRequest>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT OR REPLACE INTO user_preferences (user_id, key, value, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(user.id)
    .bind(&key)
    .bind(&payload.value)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to set preference: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}
