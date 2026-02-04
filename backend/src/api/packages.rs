use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::models::manifest::{
    PackageManifest, Requirements, InstallConfig, UninstallConfig, FrontendConfig, WindowConfig,
    InstallStep,
};
use crate::services::package::PackageService;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_packages))
        .route("/catalog", get(get_catalog))
        .route("/install", post(install_package))
        .route("/:id", get(get_package))
        .route("/:id", delete(uninstall_package))
        .route("/task/:id", get(get_task))
}

/// List installed packages
async fn list_packages(State(state): State<AppState>) -> impl IntoResponse {
    let service = PackageService::new(state.db.clone()).await;

    match service.list_installed().await {
        Ok(packages) => Json(packages).into_response(),
        Err(e) => {
            tracing::error!("Failed to list packages: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get package catalog from remote, with built-in fallback
async fn get_catalog(State(_state): State<AppState>) -> impl IntoResponse {
    let catalog_url = std::env::var("PINAS_CATALOG_URL")
        .unwrap_or_else(|_| "https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/catalog.json".to_string());

    tracing::debug!("Fetching catalog from: {}", catalog_url);

    // Try to fetch remote catalog
    match reqwest::get(&catalog_url).await {
        Ok(response) => {
            tracing::debug!("Catalog response status: {}", response.status());
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(catalog) => {
                        let app_count = catalog.get("apps")
                            .and_then(|a| a.as_array())
                            .map(|a| a.len())
                            .unwrap_or(0);
                        tracing::info!("Loaded remote catalog with {} apps", app_count);
                        return Json(catalog).into_response();
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse catalog JSON: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to fetch remote catalog: {}", e);
        }
    }

    // Fallback to built-in catalog
    tracing::info!("Using built-in catalog (remote unavailable)");
    Json(get_builtin_catalog()).into_response()
}

/// Built-in catalog for when remote is unavailable
fn get_builtin_catalog() -> serde_json::Value {
    serde_json::json!({
        "version": "1.0.0",
        "updated": chrono::Utc::now().to_rfc3339(),
        "apps": [
            {
                "id": "docker",
                "name": "Docker",
                "version": "24.0.7",
                "category": "containers",
                "icon": "mdi:docker",
                "manifest": null
            }
        ]
    })
}

/// Get a specific installed package
async fn get_package(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PackageService::new(state.db.clone()).await;

    match service.get_installed(&id).await {
        Ok(Some(package)) => Json(package).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get package: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Install package request
#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    /// Package ID to install (will be resolved from catalog)
    #[serde(default)]
    pub package_id: Option<String>,
    /// Either a manifest URL or inline manifest
    #[serde(default)]
    pub manifest_url: Option<String>,
    #[serde(default)]
    pub manifest: Option<PackageManifest>,
}

/// Install package response
#[derive(Debug, Serialize)]
pub struct InstallResponse {
    pub task_id: String,
    pub package_id: String,
}

/// Install a package
async fn install_package(
    State(state): State<AppState>,
    Json(request): Json<InstallRequest>,
) -> impl IntoResponse {
    let service = PackageService::new(state.db.clone()).await;

    // Initialize directories
    if let Err(e) = service.init_directories().await {
        tracing::error!("Failed to init directories: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response();
    }

    // Get manifest - try multiple methods
    let (manifest, manifest_url) = if let Some(manifest) = request.manifest {
        // Inline manifest provided
        (manifest, request.manifest_url)
    } else if let Some(url) = &request.manifest_url {
        // Manifest URL provided
        match fetch_manifest(url).await {
            Ok(manifest) => (manifest, Some(url.clone())),
            Err(e) => {
                tracing::error!("Failed to fetch manifest: {}", e);
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": format!("Failed to fetch manifest: {}", e)
                }))).into_response();
            }
        }
    } else if let Some(package_id) = &request.package_id {
        // Package ID provided - resolve from catalog or use built-in
        match resolve_package_manifest(package_id).await {
            Ok((manifest, url)) => (manifest, url),
            Err(e) => {
                tracing::error!("Failed to resolve package {}: {}", package_id, e);
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": format!("Failed to resolve package: {}", e)
                }))).into_response();
            }
        }
    } else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Either package_id, manifest, or manifest_url is required"
        }))).into_response();
    };

    // Install package
    match service.install(&manifest, manifest_url.as_deref()).await {
        Ok(task_id) => Json(InstallResponse {
            task_id,
            package_id: manifest.id,
        }).into_response(),
        Err(e) => {
            tracing::error!("Failed to install package: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

/// Resolve package manifest from package ID
/// First tries catalog, then falls back to built-in manifests
async fn resolve_package_manifest(package_id: &str) -> anyhow::Result<(PackageManifest, Option<String>)> {
    // Try to fetch from catalog first
    let catalog_url = std::env::var("PINAS_CATALOG_URL")
        .unwrap_or_else(|_| "https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/catalog.json".to_string());

    if let Ok(response) = reqwest::get(&catalog_url).await {
        if response.status().is_success() {
            if let Ok(catalog) = response.json::<serde_json::Value>().await {
                if let Some(apps) = catalog.get("apps").and_then(|a| a.as_array()) {
                    for app in apps {
                        if app.get("id").and_then(|i| i.as_str()) == Some(package_id) {
                            if let Some(manifest_url) = app.get("manifest").and_then(|m| m.as_str()) {
                                let manifest = fetch_manifest(manifest_url).await?;
                                return Ok((manifest, Some(manifest_url.to_string())));
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to built-in manifests for known packages
    match package_id {
        "docker" => Ok((get_docker_manifest(), None)),
        _ => anyhow::bail!("Unknown package: {}. Catalog unavailable.", package_id),
    }
}

/// Built-in Docker manifest for testing without catalog
fn get_docker_manifest() -> PackageManifest {
    use std::collections::HashMap;

    let mut description = HashMap::new();
    description.insert("en".to_string(), "Container platform for deploying and managing applications".to_string());
    description.insert("fr".to_string(), "Plateforme de conteneurs pour déployer et gérer des applications".to_string());

    let mut i18n = HashMap::new();
    i18n.insert("en".to_string(), serde_json::json!({
        "serviceStatus": "Service running",
        "status": {
            "normal": "Normal",
            "stopped": "Stopped",
            "error": "Error"
        },
        "stats": {
            "projects": "Projects",
            "containers": "Containers",
            "local": "Local",
            "data": "Data"
        },
        "cpuUsage": "CPU usage",
        "memoryCapacity": "Memory capacity",
        "available": "Available",
        "views": {
            "overview": "Overview",
            "project": "Project",
            "container": "Container",
            "image": "Image",
            "network": "Network",
            "log": "Log",
            "management": "Management"
        },
        "table": {
            "name": "Name",
            "image": "Image",
            "status": "Status",
            "ports": "Ports",
            "actions": "Actions",
            "repository": "Repository",
            "tag": "Tag",
            "imageId": "Image ID",
            "size": "Size",
            "created": "Created"
        },
        "noContainers": "No containers found",
        "noImages": "No images found",
        "underDevelopment": "This section is under development"
    }));
    i18n.insert("fr".to_string(), serde_json::json!({
        "serviceStatus": "Service en cours",
        "status": {
            "normal": "Normal",
            "stopped": "Arrêté",
            "error": "Erreur"
        },
        "stats": {
            "projects": "Projets",
            "containers": "Conteneurs",
            "local": "Local",
            "data": "Données"
        },
        "cpuUsage": "Utilisation CPU",
        "memoryCapacity": "Capacité mémoire",
        "available": "Disponible",
        "views": {
            "overview": "Aperçu",
            "project": "Projet",
            "container": "Conteneur",
            "image": "Image",
            "network": "Réseau",
            "log": "Journal",
            "management": "Gestion"
        },
        "table": {
            "name": "Nom",
            "image": "Image",
            "status": "Statut",
            "ports": "Ports",
            "actions": "Actions",
            "repository": "Dépôt",
            "tag": "Tag",
            "imageId": "ID Image",
            "size": "Taille",
            "created": "Créé"
        },
        "noContainers": "Aucun conteneur trouvé",
        "noImages": "Aucune image trouvée",
        "underDevelopment": "Cette section est en cours de développement"
    }));

    // Build installation steps matching the catalog manifest
    let install_steps = vec![
        InstallStep::Mkdir { path: "${PACKAGES_DIR}/docker".to_string() },
        InstallStep::Download {
            url: "https://download.docker.com/linux/static/stable/${ARCH}/docker-24.0.7.tgz".to_string(),
            sha256: None,  // Docker doesn't provide stable checksums
            sha256_aarch64: None,
            sha256_x86_64: None,
            dest: "${DOWNLOADS_DIR}/docker-24.0.7.tgz".to_string(),
        },
        InstallStep::Extract {
            src: "${DOWNLOADS_DIR}/docker-24.0.7.tgz".to_string(),
            dest: "${PACKAGES_DIR}/docker".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/docker".to_string(),
            dest: "${BIN_DIR}/docker".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/dockerd".to_string(),
            dest: "${BIN_DIR}/dockerd".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/containerd".to_string(),
            dest: "${BIN_DIR}/containerd".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/containerd-shim-runc-v2".to_string(),
            dest: "${BIN_DIR}/containerd-shim-runc-v2".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/ctr".to_string(),
            dest: "${BIN_DIR}/ctr".to_string(),
        },
        InstallStep::Symlink {
            src: "${PACKAGES_DIR}/docker/docker/runc".to_string(),
            dest: "${BIN_DIR}/runc".to_string(),
        },
        InstallStep::Chmod { path: "${BIN_DIR}/docker".to_string(), mode: "755".to_string() },
        InstallStep::Chmod { path: "${BIN_DIR}/dockerd".to_string(), mode: "755".to_string() },
        InstallStep::Mkdir { path: "${DATA_DIR}/docker".to_string() },
        InstallStep::Template { src: "docker.service".to_string(), dest: "/storage/.config/system.d/docker.service".to_string() },
        InstallStep::Template { src: "daemon.json".to_string(), dest: "${DATA_DIR}/docker/daemon.json".to_string() },
        InstallStep::Exec { command: "systemctl daemon-reload".to_string(), ignore_error: false },
        InstallStep::Exec { command: "systemctl enable docker.service".to_string(), ignore_error: false },
        InstallStep::Exec { command: "systemctl start docker.service".to_string(), ignore_error: false },
    ];

    let uninstall_steps = vec![
        InstallStep::Exec { command: "systemctl stop docker.service".to_string(), ignore_error: true },
        InstallStep::Exec { command: "systemctl disable docker.service".to_string(), ignore_error: true },
        InstallStep::Delete { path: "/storage/.config/system.d/docker.service".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/docker".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/dockerd".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/containerd".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/containerd-shim-runc-v2".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/ctr".to_string() },
        InstallStep::Delete { path: "${BIN_DIR}/runc".to_string() },
        InstallStep::Delete { path: "${PACKAGES_DIR}/docker".to_string() },
        InstallStep::Exec { command: "systemctl daemon-reload".to_string(), ignore_error: true },
    ];

    // Template files (base64 encoded)
    let mut files = HashMap::new();
    // docker.service content
    files.insert("docker.service".to_string(),
        "W1VuaXRdCkRlc2NyaXB0aW9uPURvY2tlciBBcHBsaWNhdGlvbiBDb250YWluZXIgRW5naW5lCkRvY3VtZW50YXRpb249aHR0cHM6Ly9kb2NzLmRvY2tlci5jb20KQWZ0ZXI9bmV0d29yay10YXJnZXQudGFyZ2V0IGxvY2FsLWZzLnRhcmdldApXYW50cz1uZXR3b3JrLXRhcmdldC50YXJnZXQKCltTZXJ2aWNlXQpUeXBlPW5vdGlmeQpFbnZpcm9ubWVudD0iUEFUSD0vc3RvcmFnZS8ucGluYXMvYmluOi91c3IvbG9jYWwvYmluOi91c3IvYmluOi9iaW4iCkV4ZWNTdGFydD0vc3RvcmFnZS8ucGluYXMvYmluL2RvY2tlcmQgLS1jb25maWctZmlsZT0vc3RvcmFnZS8ucGluYXMvZG9ja2VyL2RhZW1vbi5qc29uCkV4ZWNSZWxvYWQ9L2Jpbi9raWxsIC1zIEhVUCAkTUFJTlBJRApUaW1lb3V0U3RhcnRTZWM9MAoKUmVzdGFydD1hbHdheXMKUmVzdGFydFNlYz01cwoKTGltaXROT0ZJTEU9MTA0ODU3NgpMaW1pdE5QUk9DPWluZmluaXR5CkxpbWl0Q09SRT1pbmZpbml0eQoKRGVsZWdhdGU9eWVzCktpbGxNb2RlPXByb2Nlc3MKT09NU2NvcmVBZGp1c3Q9LTUwMAoKW0luc3RhbGxdCldhbnRlZEJ5PW11bHRpLXVzZXIudGFyZ2V0Cg==".to_string());
    // daemon.json content
    files.insert("daemon.json".to_string(),
        "ewogICJkYXRhLXJvb3QiOiAiL3N0b3JhZ2UvLnBpbmFzL2RvY2tlci9kYXRhIiwKICAic3RvcmFnZS1kcml2ZXIiOiAib3ZlcmxheTIiLAogICJsb2ctZHJpdmVyIjogImpzb24tZmlsZSIsCiAgImxvZy1vcHRzIjogewogICAgIm1heC1zaXplIjogIjEwbSIsCiAgICAibWF4LWZpbGUiOiAiMyIKICB9LAogICJkZWZhdWx0LWFkZHJlc3MtcG9vbHMiOiBbCiAgICB7CiAgICAgICJiYXNlIjogIjE3Mi4xNy4wLjAvMTYiLAogICAgICAic2l6ZSI6IDI0CiAgICB9CiAgXQp9Cg==".to_string());

    PackageManifest {
        id: "docker".to_string(),
        name: "Docker".to_string(),
        version: "24.0.7".to_string(),
        description,
        author: Some("Docker Inc.".to_string()),
        license: Some("Apache-2.0".to_string()),
        website: Some("https://www.docker.com".to_string()),
        icon: Some("mdi:docker".to_string()),
        requirements: Requirements {
            min_ram: Some(512),
            min_disk: Some(500),
            arch: vec!["aarch64".to_string(), "x86_64".to_string()],
            dependencies: vec![],
        },
        install: InstallConfig {
            install_type: "binary".to_string(),
            steps: install_steps,
            image: None,
            container: None,
        },
        uninstall: UninstallConfig {
            steps: uninstall_steps,
        },
        files,
        config: HashMap::new(),
        frontend: Some(FrontendConfig {
            icon: "mdi:docker".to_string(),
            gradient: "from-blue-500 to-blue-600".to_string(),
            component: "DockerApp".to_string(),
            window: WindowConfig {
                width: 1000,
                height: 650,
                min_width: 800,
                min_height: 500,
            },
            config: HashMap::new(),
            i18n,
        }),
    }
}

/// Uninstall a package
async fn uninstall_package(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PackageService::new(state.db.clone()).await;

    match service.uninstall(&id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to uninstall package: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Get installation task status
async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = PackageService::new(state.db.clone()).await;

    match service.get_task(&id).await {
        Ok(Some(task)) => Json(task).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Task not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get task: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Fetch manifest from URL
async fn fetch_manifest(url: &str) -> anyhow::Result<PackageManifest> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }
    let manifest = response.json::<PackageManifest>().await?;
    Ok(manifest)
}
