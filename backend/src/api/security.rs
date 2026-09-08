use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::api::error::ApiError;
use crate::services::password_policy::PasswordPolicy;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/password-policy", get(get_policy).put(put_policy))
}

async fn get_policy(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(PasswordPolicy::load(&state.db).await?))
}

async fn put_policy(
    State(state): State<AppState>,
    Json(policy): Json<PasswordPolicy>,
) -> Result<impl IntoResponse, ApiError> {
    if !(4..=128).contains(&policy.min_length) {
        return Err(ApiError::bad_request("min_length must be between 4 and 128"));
    }
    policy.save(&state.db).await?;
    Ok(Json(PasswordPolicy::load(&state.db).await?))
}
