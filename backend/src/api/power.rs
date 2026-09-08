use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::services::power::{PowerService, ScheduledTaskInput};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/governor", post(set_governor))
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route("/schedules/{id}", delete(delete_schedule))
        .route("/schedules/{id}/toggle", post(toggle_schedule))
}

async fn status(State(state): State<AppState>) -> impl IntoResponse {
    Json(PowerService::new(state.db.clone()).status().await)
}

#[derive(Deserialize)]
struct GovernorBody {
    governor: String,
}

async fn set_governor(State(state): State<AppState>, Json(body): Json<GovernorBody>) -> Result<impl IntoResponse, ApiError> {
    let svc = PowerService::new(state.db.clone());
    svc.set_governor(&body.governor).await.map_err(|e| ApiError::bad_request(e).with_code("GOVERNOR_REJECTED"))?;
    Ok(Json(svc.status().await))
}

async fn list_schedules(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(PowerService::new(state.db.clone()).list_tasks().await?))
}

async fn create_schedule(State(state): State<AppState>, Json(input): Json<ScheduledTaskInput>) -> Result<impl IntoResponse, ApiError> {
    let task = PowerService::new(state.db.clone())
        .create_task(input)
        .await
        .map_err(|e| ApiError::bad_request(e).with_code("INVALID_SCHEDULE"))?;
    Ok((StatusCode::CREATED, Json(task)))
}

#[derive(Deserialize)]
struct ToggleBody {
    enabled: bool,
}

async fn toggle_schedule(State(state): State<AppState>, Path(id): Path<String>, Json(body): Json<ToggleBody>) -> Result<impl IntoResponse, ApiError> {
    PowerService::new(state.db.clone())
        .set_task_enabled(&id, body.enabled)
        .await
        .map_err(|e| ApiError::not_found(e))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_schedule(State(state): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    PowerService::new(state.db.clone()).delete_task(&id).await.map_err(|e| ApiError::not_found(e))?;
    Ok(StatusCode::NO_CONTENT)
}
