use anyhow::{anyhow, Result};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogOutput, LogsOptions,
    RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions,
    Stats, StatsOptions,
};
use bollard::image::{ListImagesOptions, RemoveImageOptions};
use bollard::models::{ContainerSummary, HostConfig, ImageSummary, PortBinding};
use bollard::network::ListNetworksOptions;
use bollard::volume::{ListVolumesOptions, RemoveVolumeOptions};
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::manifest::ContainerConfig;

/// Docker service for managing containers and images
pub struct DockerService {
    client: Option<Docker>,
}

/// Docker system stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStats {
    pub running: bool,
    pub version: Option<String>,
    pub containers_total: i64,
    pub containers_running: i64,
    pub containers_paused: i64,
    pub containers_stopped: i64,
    pub images: i64,
    pub data_usage: Option<u64>,
}

/// Container info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
    pub ports: Vec<PortInfo>,
}

/// Port info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub host: Option<u16>,
    pub container: u16,
    pub protocol: String,
}

/// Image info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

/// Volume info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mount_point: String,
    pub created: String,
}

/// Network info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub containers: Vec<String>,
}

/// Result of a prune operation
#[derive(Debug, Clone, Serialize)]
pub struct PruneResult {
    pub deleted: u64,
    pub space_reclaimed: u64,
}

/// Container stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
}

impl DockerService {
    /// Create a new Docker service
    pub async fn new() -> Self {
        let client = Self::connect().await.ok();
        Self { client }
    }

    /// Try to connect to Docker daemon
    async fn connect() -> Result<Docker> {
        // Try socket path first (common on Linux)
        let socket_path = std::env::var("DOCKER_HOST")
            .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_string());

        // Also check LibreELEC's typical paths
        let paths = vec![
            socket_path.clone(),
            "unix:///storage/.pinas/docker/docker.sock".to_string(),
            "unix:///run/docker.sock".to_string(),
        ];

        for path in paths {
            if let Ok(docker) = Docker::connect_with_socket(&path.replace("unix://", ""), 120, bollard::API_DEFAULT_VERSION) {
                // Verify connection
                if docker.ping().await.is_ok() {
                    tracing::info!("Connected to Docker at {}", path);
                    return Ok(docker);
                }
            }
        }

        Err(anyhow!("Failed to connect to Docker daemon"))
    }

    /// Check if Docker is available
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Try to reconnect to Docker daemon (useful after Docker is installed)
    pub async fn reconnect(&mut self) -> bool {
        if let Ok(client) = Self::connect().await {
            self.client = Some(client);
            true
        } else {
            false
        }
    }

    /// Wait for Docker to become available (with timeout)
    pub async fn wait_for_docker(&mut self, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if self.reconnect().await {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        false
    }

    /// Get Docker client or error
    fn client(&self) -> Result<&Docker> {
        self.client.as_ref().ok_or_else(|| anyhow!("Docker is not available"))
    }

    /// Get Docker system stats
    pub async fn get_stats(&self) -> Result<DockerStats> {
        let client = self.client()?;
        let info = client.info().await?;

        Ok(DockerStats {
            running: true,
            version: info.server_version,
            containers_total: info.containers.unwrap_or(0),
            containers_running: info.containers_running.unwrap_or(0),
            containers_paused: info.containers_paused.unwrap_or(0),
            containers_stopped: info.containers_stopped.unwrap_or(0),
            images: info.images.unwrap_or(0),
            data_usage: None, // TODO: Parse from df
        })
    }

    /// Get Docker stats when not available
    pub fn get_unavailable_stats() -> DockerStats {
        DockerStats {
            running: false,
            version: None,
            containers_total: 0,
            containers_running: 0,
            containers_paused: 0,
            containers_stopped: 0,
            images: 0,
            data_usage: None,
        }
    }

    /// List all containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let client = self.client()?;

        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = client.list_containers(Some(options)).await?;

        Ok(containers
            .into_iter()
            .map(|c| ContainerInfo {
                id: c.id.unwrap_or_default(),
                name: c.names.unwrap_or_default().first()
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                created: c.created.unwrap_or(0),
                ports: c.ports.unwrap_or_default()
                    .into_iter()
                    .map(|p| PortInfo {
                        host: p.public_port,
                        container: p.private_port,
                        protocol: p.typ.map(|t| t.to_string()).unwrap_or_else(|| "tcp".to_string()),
                    })
                    .collect(),
            })
            .collect())
    }

    /// Get a specific container
    pub async fn get_container(&self, id_or_name: &str) -> Result<ContainerInfo> {
        let containers = self.list_containers(true).await?;
        containers
            .into_iter()
            .find(|c| c.id.starts_with(id_or_name) || c.name == id_or_name)
            .ok_or_else(|| anyhow!("Container not found: {}", id_or_name))
    }

    /// Start a container
    pub async fn start_container(&self, id_or_name: &str) -> Result<()> {
        let client = self.client()?;
        client
            .start_container(id_or_name, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, id_or_name: &str) -> Result<()> {
        let client = self.client()?;
        client
            .stop_container(id_or_name, Some(StopContainerOptions { t: 10 }))
            .await?;
        Ok(())
    }

    /// Restart a container
    pub async fn restart_container(&self, id_or_name: &str) -> Result<()> {
        let client = self.client()?;
        client
            .restart_container(id_or_name, Some(RestartContainerOptions { t: 10 }))
            .await?;
        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, id_or_name: &str, force: bool) -> Result<()> {
        let client = self.client()?;
        client
            .remove_container(
                id_or_name,
                Some(RemoveContainerOptions {
                    force,
                    ..Default::default()
                }),
            )
            .await?;
        Ok(())
    }

    /// Get container logs
    pub async fn get_logs(&self, id_or_name: &str, tail: usize) -> Result<Vec<String>> {
        let client = self.client()?;

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        };

        let mut logs = client.logs(id_or_name, Some(options));
        let mut lines = Vec::new();

        while let Some(log) = logs.next().await {
            match log? {
                LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                    if let Ok(s) = String::from_utf8(message.to_vec()) {
                        lines.push(s);
                    }
                }
                _ => {}
            }
        }

        Ok(lines)
    }

    /// Get container stats (single snapshot)
    pub async fn get_container_stats(&self, id_or_name: &str) -> Result<ContainerStats> {
        let client = self.client()?;

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = client.stats(id_or_name, Some(options));

        if let Some(stats_result) = stats_stream.next().await {
            let stats = stats_result?;

            let cpu_percent = calculate_cpu_percent(&stats);
            let memory_usage = stats.memory_stats.usage.unwrap_or(0);
            let memory_limit = stats.memory_stats.limit.unwrap_or(1);
            let memory_percent = (memory_usage as f64 / memory_limit as f64) * 100.0;

            let (network_rx, network_tx) = stats
                .networks
                .as_ref()
                .map(|networks| {
                    networks.values().fold((0u64, 0u64), |(rx, tx), net| {
                        (rx + net.rx_bytes, tx + net.tx_bytes)
                    })
                })
                .unwrap_or((0, 0));

            return Ok(ContainerStats {
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                network_rx,
                network_tx,
            });
        }

        Err(anyhow!("No stats available"))
    }

    /// List all images
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>> {
        let client = self.client()?;

        let options = ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        };

        let images = client.list_images(Some(options)).await?;

        Ok(images
            .into_iter()
            .map(|i| ImageInfo {
                id: i.id.chars().take(12).collect(),
                repo_tags: i.repo_tags,
                size: i.size,
                created: i.created,
            })
            .collect())
    }

    /// Pull an image
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        let client = self.client()?;

        use bollard::image::CreateImageOptions;
        use futures_util::TryStreamExt;

        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let mut stream = client.create_image(Some(options), None, None);

        while let Some(info) = stream.try_next().await? {
            if let Some(status) = info.status {
                tracing::info!("Pull {}: {}", image, status);
            }
        }

        Ok(())
    }

    /// Remove an image
    pub async fn remove_image(&self, image: &str, force: bool) -> Result<()> {
        let client = self.client()?;

        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };

        client.remove_image(image, Some(options), None).await?;
        Ok(())
    }

    /// List all volumes
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        let client = self.client()?;

        let options = ListVolumesOptions::<String> {
            ..Default::default()
        };

        let response = client.list_volumes(Some(options)).await?;
        let volumes = response.volumes.unwrap_or_default();

        Ok(volumes
            .into_iter()
            .map(|v| VolumeInfo {
                name: v.name,
                driver: v.driver,
                mount_point: v.mountpoint,
                created: v.created_at.unwrap_or_default(),
            })
            .collect())
    }

    /// Remove a volume
    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        let client = self.client()?;
        client.remove_volume(name, Some(RemoveVolumeOptions { force })).await?;
        Ok(())
    }

    /// List all networks
    pub async fn list_networks(&self) -> Result<Vec<NetworkInfo>> {
        let client = self.client()?;

        let options = ListNetworksOptions::<String> {
            ..Default::default()
        };

        let networks = client.list_networks(Some(options)).await?;

        Ok(networks
            .into_iter()
            .map(|n| {
                let containers: Vec<String> = n
                    .containers
                    .unwrap_or_default()
                    .values()
                    .filter_map(|c| c.name.clone())
                    .collect();

                NetworkInfo {
                    id: n.id.unwrap_or_default(),
                    name: n.name.unwrap_or_default(),
                    driver: n.driver.unwrap_or_default(),
                    scope: n.scope.unwrap_or_default(),
                    containers,
                }
            })
            .collect())
    }

    /// Remove a network
    pub async fn remove_network(&self, id: &str) -> Result<()> {
        let client = self.client()?;
        client.remove_network(id).await?;
        Ok(())
    }

    /// Prune unused images (dangling by default, all if `all` is true)
    pub async fn prune_images(&self, all: bool) -> Result<PruneResult> {
        let client = self.client()?;
        let mut filters = HashMap::new();
        if !all {
            filters.insert("dangling", vec!["true"]);
        }
        let options = bollard::image::PruneImagesOptions { filters };
        let result = client.prune_images(Some(options)).await?;
        let deleted = result.images_deleted.map(|v| v.len() as u64).unwrap_or(0);
        let space = result.space_reclaimed.unwrap_or(0) as u64;
        Ok(PruneResult { deleted, space_reclaimed: space })
    }

    /// Prune unused volumes
    pub async fn prune_volumes(&self) -> Result<PruneResult> {
        let client = self.client()?;
        let options = bollard::volume::PruneVolumesOptions::<String> {
            filters: Default::default(),
        };
        let result = client.prune_volumes(Some(options)).await?;
        let deleted = result.volumes_deleted.map(|v| v.len() as u64).unwrap_or(0);
        let space = result.space_reclaimed.unwrap_or(0) as u64;
        Ok(PruneResult { deleted, space_reclaimed: space })
    }

    /// Create a container from config
    pub async fn create_container(&self, config: &ContainerConfig) -> Result<String> {
        let client = self.client()?;

        let image = config.image.as_ref()
            .ok_or_else(|| anyhow!("Image is required"))?;

        // Build port bindings
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
        for port in &config.ports {
            let container_port = format!("{}/{}", port.container, port.protocol);
            port_bindings.insert(
                container_port,
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(port.host.to_string()),
                }]),
            );
        }

        // Build volume binds
        let binds: Vec<String> = config
            .volumes
            .iter()
            .map(|v| {
                if v.readonly {
                    format!("{}:{}:ro", v.host, v.container)
                } else {
                    format!("{}:{}", v.host, v.container)
                }
            })
            .collect();

        // Build environment variables
        let env: Vec<String> = config
            .environment
            .iter()
            .map(|e| format!("{}={}", e.name, e.value))
            .collect();

        // Build tmpfs map
        let tmpfs_map: Option<HashMap<String, String>> = if config.tmpfs.is_empty() {
            None
        } else {
            Some(config.tmpfs.iter().map(|t| (t.clone(), String::new())).collect())
        };

        // Build host config
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(binds),
            network_mode: config.network.clone(),
            devices: if config.devices.is_empty() {
                None
            } else {
                Some(
                    config
                        .devices
                        .iter()
                        .map(|d| {
                            let parts: Vec<&str> = d.split(':').collect();
                            bollard::models::DeviceMapping {
                                path_on_host: Some(parts.get(0).unwrap_or(&"").to_string()),
                                path_in_container: Some(parts.get(1).unwrap_or(parts.get(0).unwrap_or(&"")).to_string()),
                                cgroup_permissions: Some("rwm".to_string()),
                            }
                        })
                        .collect(),
                )
            },
            restart_policy: config.restart.as_ref().map(|r| bollard::models::RestartPolicy {
                name: Some(match r.as_str() {
                    "always" => bollard::models::RestartPolicyNameEnum::ALWAYS,
                    "unless-stopped" => bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED,
                    "on-failure" => bollard::models::RestartPolicyNameEnum::ON_FAILURE,
                    _ => bollard::models::RestartPolicyNameEnum::NO,
                }),
                maximum_retry_count: None,
            }),
            privileged: Some(config.privileged),
            cap_add: if config.cap_add.is_empty() { None } else { Some(config.cap_add.clone()) },
            cap_drop: if config.cap_drop.is_empty() { None } else { Some(config.cap_drop.clone()) },
            dns: if config.dns.is_empty() { None } else { Some(config.dns.clone()) },
            extra_hosts: if config.extra_hosts.is_empty() { None } else { Some(config.extra_hosts.clone()) },
            tmpfs: tmpfs_map,
            ..Default::default()
        };

        // Build container config
        let container_config = Config {
            image: Some(image.clone()),
            hostname: config.hostname.clone(),
            env: Some(env),
            user: config.user.clone(),
            cmd: config.command.clone(),
            entrypoint: config.entrypoint.clone(),
            host_config: Some(host_config),
            labels: Some(config.labels.clone()),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: config.name.clone(),
            platform: None,
        };

        let response = client.create_container(Some(options), container_config).await?;
        Ok(response.id)
    }
}

/// Calculate CPU percentage from stats
fn calculate_cpu_percent(stats: &Stats) -> f64 {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
        - stats.precpu_stats.cpu_usage.total_usage as f64;

    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    let cpu_count = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * cpu_count * 100.0
    } else {
        0.0
    }
}
