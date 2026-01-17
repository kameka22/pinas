use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod models;
mod services;

use crate::config::AppConfig;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pinas=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    let bind_addr = config.bind_address.clone();

    // Initialize database
    let db = db::init_pool(&config.database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    // Create app state
    let state = AppState {
        config: Arc::new(config),
        db,
    };

    // Build router
    let app = create_router(state);

    // Start server
    let addr: SocketAddr = bind_addr.parse()?;
    tracing::info!("PiNAS server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the main router with all routes
fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/api/health", get(health_check))
        // API routes
        .nest("/api/auth", api::auth::router())
        .nest("/api/files", api::files::router())
        .nest("/api/system", api::system::router())
        .nest("/api/storage", api::storage::router())
        .nest("/api/shares", api::shares::router())
        .nest("/api/users", api::users::router())
        // WebSocket
        .route("/api/ws", get(api::ws::ws_handler))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // State
        .with_state(state)
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
