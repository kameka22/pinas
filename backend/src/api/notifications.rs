use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::services::notification::{NotificationError, NotificationService};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).delete(clear))
        .route("/count", get(unread_count))
        .route("/read-all", post(mark_all_read))
        .route("/{id}/read", post(mark_read))
        .route("/{id}", delete(remove))
}

impl From<NotificationError> for ApiError {
    fn from(err: NotificationError) -> Self {
        match err {
            NotificationError::NotFound => ApiError::not_found(err).with_code("NOTIFICATION_NOT_FOUND"),
            NotificationError::Database(_) => ApiError::internal(err).with_code("DATABASE_ERROR"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    unread: bool,
}

fn default_limit() -> i64 {
    50
}

async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> Result<impl IntoResponse, ApiError> {
    let items = NotificationService::new(state.db.clone())
        .list(q.limit.clamp(1, 500), q.unread)
        .await?;
    Ok(Json(items))
}

async fn unread_count(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let count = NotificationService::new(state.db.clone()).unread_count().await?;
    Ok(Json(serde_json::json!({ "unread": count })))
}

async fn mark_read(State(state): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    NotificationService::new(state.db.clone()).mark_read(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mark_all_read(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let n = NotificationService::new(state.db.clone()).mark_all_read().await?;
    Ok(Json(serde_json::json!({ "updated": n })))
}

async fn remove(State(state): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    NotificationService::new(state.db.clone()).delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let n = NotificationService::new(state.db.clone()).clear().await?;
    Ok(Json(serde_json::json!({ "deleted": n })))
}
