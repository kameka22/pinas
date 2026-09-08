use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::api::error::ApiError;
use crate::api::middleware::AdminUser;
use crate::services::timedate::{TimeDateService, TimeUpdate};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(status).put(update))
        .route("/zones", get(zones))
        .route("/sync", post(sync_now))
}

async fn status(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(TimeDateService::new(state.db.clone()).status().await?))
}

async fn zones() -> impl IntoResponse {
    Json(TimeDateService::timezones())
}

async fn update(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(update): Json<TimeUpdate>,
) -> Result<impl IntoResponse, ApiError> {
    TimeDateService::new(state.db.clone())
        .update(update)
        .await
        .map(Json)
        .map_err(|e| ApiError::bad_request(e).with_code("TIME_UPDATE_FAILED"))
}

async fn sync_now(State(state): State<AppState>, _admin: AdminUser) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(TimeDateService::new(state.db.clone()).sync_now().await?))
}
