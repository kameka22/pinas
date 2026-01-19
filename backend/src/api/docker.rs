use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::models::manifest::ContainerConfig;
use crate::services::docker::DockerService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Docker system
        .route("/status", get(get_status))
        // Containers
        .route("/containers", get(list_containers))
        .route("/containers", post(create_container))
        .route("/containers/:id", get(get_container))
        .route("/containers/:id", delete(remove_container))
        .route("/containers/:id/start", post(start_container))
        .route("/containers/:id/stop", post(stop_container))
        .route("/containers/:id/restart", post(restart_container))
        .route("/containers/:id/logs", get(get_logs))
        .route("/containers/:id/stats", get(get_container_stats))
        // Images
        .route("/images", get(list_images))
        .route("/images/pull", post(pull_image))
        .route("/images/:id", delete(remove_image))
}

/// Get Docker status
async fn get_status(State(_state): State<AppState>) -> impl IntoResponse {
    let service = DockerService::new().await;

    if service.is_available() {
        match service.get_stats().await {
            Ok(stats) => Json(stats).into_response(),
            Err(e) => {
                tracing::error!("Failed to get Docker stats: {}", e);
                Json(DockerService::get_unavailable_stats()).into_response()
            }
        }
    } else {
        Json(DockerService::get_unavailable_stats()).into_response()
    }
}

/// Query parameters for listing containers
#[derive(Debug, Deserialize)]
pub struct ListContainersQuery {
    #[serde(default)]
    pub all: bool,
}

/// List containers
async fn list_containers(
    State(_state): State<AppState>,
    Query(query): Query<ListContainersQuery>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.list_containers(query.all).await {
        Ok(containers) => Json(containers).into_response(),
        Err(e) => {
            tracing::error!("Failed to list containers: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get a specific container
async fn get_container(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.get_container(&id).await {
        Ok(container) => Json(container).into_response(),
        Err(e) => {
            tracing::error!("Failed to get container: {}", e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// Create a container
async fn create_container(
    State(_state): State<AppState>,
    Json(config): Json<ContainerConfig>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.create_container(&config).await {
        Ok(id) => Json(serde_json::json!({ "id": id })).into_response(),
        Err(e) => {
            tracing::error!("Failed to create container: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Remove a container
#[derive(Debug, Deserialize)]
pub struct RemoveQuery {
    #[serde(default)]
    pub force: bool,
}

async fn remove_container(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RemoveQuery>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.remove_container(&id, query.force).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to remove container: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Start a container
async fn start_container(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.start_container(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to start container: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Stop a container
async fn stop_container(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.stop_container(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to stop container: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Restart a container
async fn restart_container(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.restart_container(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to restart container: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Query parameters for logs
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_tail")]
    pub tail: usize,
}

fn default_tail() -> usize {
    100
}

/// Get container logs
async fn get_logs(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.get_logs(&id, query.tail).await {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => {
            tracing::error!("Failed to get logs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get container stats
async fn get_container_stats(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.get_container_stats(&id).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!("Failed to get container stats: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// List images
async fn list_images(State(_state): State<AppState>) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.list_images().await {
        Ok(images) => Json(images).into_response(),
        Err(e) => {
            tracing::error!("Failed to list images: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Pull image request
#[derive(Debug, Deserialize)]
pub struct PullImageRequest {
    pub image: String,
}

/// Pull an image
async fn pull_image(
    State(_state): State<AppState>,
    Json(request): Json<PullImageRequest>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.pull_image(&request.image).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to pull image: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Remove an image
async fn remove_image(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RemoveQuery>,
) -> impl IntoResponse {
    let service = DockerService::new().await;

    match service.remove_image(&id, query.force).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to remove image: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
