use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::cups::{AddPrinterRequest, CupsService, UpdatePrinterRequest};
use crate::AppState;

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiSuccess {
    success: bool,
}

/// Guard: returns 503 if CUPS service is not enabled
async fn require_enabled(service: &CupsService) -> Result<(), (StatusCode, Json<ApiError>)> {
    match service.get_status().await {
        Ok(status) if status.enabled => Ok(()),
        Ok(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                message: "CUPS service is not enabled. Enable it first.".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/enable", post(enable))
        .route("/disable", post(disable))
        .route("/printers", get(get_printers))
        .route("/printers", post(add_printer))
        .route("/printers/:name", delete(remove_printer))
        .route("/printers/:name", put(update_printer))
        .route("/printers/:name/test", post(test_page))
        .route("/detect", get(detect_printers))
        .route("/drivers", get(get_drivers))
        .route("/jobs", get(get_jobs))
        .route("/jobs/:id", delete(cancel_job))
}

/// Get CUPS service status
async fn get_status(State(_state): State<AppState>) -> impl IntoResponse {
    let service = CupsService::new();

    match service.get_status().await {
        Ok(status) => Json(status).into_response(),
        Err(e) => {
            tracing::error!("Failed to get CUPS status: {}", e);
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

/// Enable CUPS service
async fn enable(State(_state): State<AppState>) -> impl IntoResponse {
    let service = CupsService::new();

    match service.enable().await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to enable CUPS: {}", e);
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

/// Disable CUPS service
async fn disable(State(_state): State<AppState>) -> impl IntoResponse {
    let service = CupsService::new();

    match service.disable().await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to disable CUPS: {}", e);
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

/// Get configured printers (requires enabled)
async fn get_printers(State(_state): State<AppState>) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.get_printers().await {
        Ok(printers) => Json(printers).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Detect USB printers (requires enabled)
async fn detect_printers(State(_state): State<AppState>) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.detect_printers().await {
        Ok(printers) => Json(printers).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct DriversQuery {
    uri: String,
}

/// Get available drivers for a printer URI (requires enabled)
async fn get_drivers(
    State(_state): State<AppState>,
    Query(query): Query<DriversQuery>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.get_drivers(&query.uri).await {
        Ok(drivers) => Json(drivers).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Add a new printer (requires enabled)
async fn add_printer(
    State(_state): State<AppState>,
    Json(payload): Json<AddPrinterRequest>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.add_printer(&payload).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => {
            tracing::error!("Failed to add printer: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    message: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Remove a printer (requires enabled)
async fn remove_printer(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.remove_printer(&name).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Update printer settings (requires enabled)
async fn update_printer(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UpdatePrinterRequest>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.update_printer(&name, &payload).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Print test page (requires enabled)
async fn test_page(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.print_test_page(&name).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct JobsQuery {
    printer: Option<String>,
}

/// Get print jobs (requires enabled)
async fn get_jobs(
    State(_state): State<AppState>,
    Query(query): Query<JobsQuery>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.get_jobs(query.printer.as_deref()).await {
        Ok(jobs) => Json(jobs).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Cancel a print job (requires enabled)
async fn cancel_job(
    State(_state): State<AppState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    let service = CupsService::new();

    if let Err(resp) = require_enabled(&service).await {
        return resp.into_response();
    }

    match service.cancel_job(id).await {
        Ok(()) => Json(ApiSuccess { success: true }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}
