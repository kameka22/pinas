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
use tokio::sync::{broadcast, Mutex};
use axum::http::{header, Method};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod models;
mod services;

use crate::api::ws::{FileTaskEvent, TaskProgressEvent};
use crate::config::AppConfig;
use crate::models::storage::StorageAlertEvent;
use crate::services::update::UpdateAppliedInfo;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: sqlx::SqlitePool,
    pub task_tx: broadcast::Sender<TaskProgressEvent>,
    pub file_task_tx: broadcast::Sender<FileTaskEvent>,
    pub storage_tx: broadcast::Sender<StorageAlertEvent>,
    pub just_updated: Arc<Mutex<Option<UpdateAppliedInfo>>>,
    pub tls_enabled: bool,
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

    // Create broadcast channel for task progress events
    let (task_tx, _) = broadcast::channel::<TaskProgressEvent>(100);

    // Create broadcast channel for file task events
    let (file_task_tx, _) = broadcast::channel::<FileTaskEvent>(100);

    // Create broadcast channel for storage alert events
    let (storage_tx, _) = broadcast::channel::<StorageAlertEvent>(50);

    // Check if an update was just applied (flag file from previous version)
    let data_dir = std::env::var("PINAS_DATA_DIR")
        .unwrap_or_else(|_| "/storage/.pinas".to_string());
    let just_updated = services::update::UpdateService::read_update_applied_flag(&data_dir);
    if let Some(ref info) = just_updated {
        tracing::info!("System was just updated: {} -> {}", info.previous_version, info.version);
    }

    // Initialize Samba (disable LibreELEC default, generate smb.conf)
    let share_svc = services::share::ShareService::new(db.clone());
    share_svc.initialize_samba().await;

    let tls_enabled = config.tls_enabled;

    let dev_mode = std::env::var("PINAS_DEV_MODE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    // Create app state
    let state = AppState {
        config: Arc::new(config),
        db: db.clone(),
        task_tx,
        file_task_tx,
        storage_tx: storage_tx.clone(),
        just_updated: Arc::new(Mutex::new(just_updated)),
        tls_enabled,
    };

    // Start storage health monitor (background task every 60s)
    services::storage::StorageService::start_health_monitor(
        db, storage_tx, dev_mode,
    );

    // Capture TLS paths before state is moved into router
    let tls_cert_path = state.config.tls_cert_path.clone();
    let tls_key_path = state.config.tls_key_path.clone();

    // Build router
    let app = create_router(state);

    // Start server
    let addr: SocketAddr = bind_addr.parse()?;

    if tls_enabled {
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            &tls_cert_path,
            &tls_key_path,
        )
        .await?;

        tracing::info!("PiNAS server starting on https://{}", addr);
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        tracing::info!("PiNAS server starting on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

/// Create the main router with all routes
fn create_router(state: AppState) -> Router {
    let tls_enabled = state.tls_enabled;

    // CORS configuration — restrict to same-origin and local dev
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse().unwrap(),  // SvelteKit dev server
            "http://localhost:3000".parse().unwrap(),  // Backend self-serve
            "http://127.0.0.1:5173".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "https://localhost:3000".parse().unwrap(), // HTTPS variants
            "https://127.0.0.1:3000".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::COOKIE])
        .allow_credentials(true);

    let static_dir = state.config.static_dir.clone();

    let mut app = Router::new()
        // Health check
        .route("/api/health", get(health_check))
        // API routes
        .nest("/api/auth", api::auth::router())
        .nest("/api/setup", api::setup::router())
        .nest("/api/files", api::files::router())
        .nest("/api/system/update", api::update::router())
        .nest("/api/system", api::system::router())
        .nest("/api/storage", api::storage::router())
        .nest("/api/shares", api::shares::router())
        .nest("/api/users", api::users::router())
        .nest("/api/groups", api::groups::router())
        .nest("/api/packages", api::packages::router())
        .nest("/api/docker", api::docker::router())
        .nest("/api/apps", api::apps::router())
        .nest("/api/services", api::services::router())
        .nest("/api/terminal", api::terminal::router())
        .nest("/api/locations", api::locations::router())
        .nest("/api/display", api::display::router())
        .nest("/api/kodi", api::kodi::router())
        .nest("/api/network", api::network::router())
        .nest("/api/permissions", api::permissions::router())
        .nest("/api/preferences", api::preferences::router())
        .nest("/api/service-access", api::service_access::router())
        .nest("/api/ssh", api::ssh::router())
        .nest("/api/cups", api::cups::router())
        // WebSocket
        .route("/api/ws", get(api::ws::ws_handler))
        // State
        .with_state(state);

    // Serve static frontend files if configured
    if let Some(dir) = static_dir {
        let index_path = format!("{}/index.html", dir);
        tracing::info!("Serving static files from: {}", dir);
        app = app.fallback_service(
            ServeDir::new(&dir).fallback(ServeFile::new(&index_path)),
        );
    }

    // Apply middleware
    let mut app = app
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-frame-options"),
            header::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-content-type-options"),
            header::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("referrer-policy"),
            header::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("x-xss-protection"),
            header::HeaderValue::from_static("1; mode=block"),
        ));

    // Add HSTS header only when TLS is active
    if tls_enabled {
        app = app.layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("strict-transport-security"),
            header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    app
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
