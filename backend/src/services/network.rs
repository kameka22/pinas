use serde::{Deserialize, Serialize};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub hostname: String,
    pub interfaces: Vec<NetworkInterface>,
    pub dns: DnsConfig,
    pub default_gateway: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub ip_address: String,
    pub subnet_mask: String,
    pub mac_address: String,
    pub speed: String,
    pub method: String,
    pub gateway: String,
    pub dns: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub manual: bool,
    pub primary: String,
    pub secondary: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInterfaceRequest {
    pub name: String,
    pub method: String,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
    pub dns: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHostnameRequest {
    pub hostname: String,
}

pub struct NetworkService {
    dev_mode: bool,
}

impl NetworkService {
    pub fn new() -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::info!("NetworkService running in dev mode - using fake network data");
        }

        Self { dev_mode }
    }

    pub async fn get_status(&self) -> anyhow::Result<NetworkStatus> {
        if self.dev_mode {
            return Ok(self.dev_status());
        }

        let hostname = self.get_hostname().await?;
        let interfaces = self.get_interfaces().await?;
        let dns = self.get_dns_config().await?;
        let default_gateway = self.get_default_gateway().await?;

        Ok(NetworkStatus {
            hostname,
            interfaces,
            dns,
            default_gateway,
        })
    }

    pub async fn update_interface(&self, req: &UpdateInterfaceRequest) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!(
                "[DEV MODE] Would configure interface {} as {} (ip={:?})",
                req.name,
                req.method,
                req.ip_address
            );
            return Ok(());
        }

        // Find connman service for the interface
        let service_id = self.find_connman_service(&req.name).await?;

        if req.method == "dhcp" {
            // Set to DHCP
            let output = AsyncCommand::new("connmanctl")
                .args(["config", &service_id, "--ipv4", "dhcp"])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to set DHCP: {}", stderr);
            }
        } else {
            // Set static IP
            let ip = req.ip_address.as_deref().unwrap_or("");
            let mask = req.subnet_mask.as_deref().unwrap_or("255.255.255.0");
            let gw = req.gateway.as_deref().unwrap_or("");

            if ip.is_empty() {
                anyhow::bail!("IP address is required for static configuration");
            }

            let output = AsyncCommand::new("connmanctl")
                .args(["config", &service_id, "--ipv4", "manual", ip, mask, gw])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to set static IP: {}", stderr);
            }

            // Set DNS if provided
            if let Some(dns) = &req.dns {
                if !dns.is_empty() {
                    let dns_servers: Vec<&str> = dns.split(',').map(|s| s.trim()).collect();
                    let mut args = vec!["config", &service_id, "--nameservers"];
                    args.extend(dns_servers);

                    let output = AsyncCommand::new("connmanctl")
                        .args(&args)
                        .output()
                        .await?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Failed to set DNS for interface: {}", stderr);
                    }
                }
            }
        }

        tracing::info!("Interface {} configured as {}", req.name, req.method);
        Ok(())
    }

    pub async fn update_dns(&self, config: &DnsConfig) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!(
                "[DEV MODE] Would set DNS: manual={}, primary={}, secondary={}",
                config.manual,
                config.primary,
                config.secondary
            );
            return Ok(());
        }

        if config.manual {
            // Find the active connman service
            let service_id = self.find_active_connman_service().await?;

            let mut dns_servers = vec![];
            if !config.primary.is_empty() {
                dns_servers.push(config.primary.as_str());
            }
            if !config.secondary.is_empty() {
                dns_servers.push(config.secondary.as_str());
            }

            if dns_servers.is_empty() {
                anyhow::bail!("At least one DNS server is required");
            }

            let mut args = vec!["config", &service_id, "--nameservers"];
            args.extend(dns_servers);

            let output = AsyncCommand::new("connmanctl")
                .args(&args)
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to set DNS: {}", stderr);
            }
        } else {
            // Reset to automatic DNS (DHCP-provided)
            let service_id = self.find_active_connman_service().await?;

            let output = AsyncCommand::new("connmanctl")
                .args(["config", &service_id, "--nameservers"])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to reset DNS to automatic: {}", stderr);
            }
        }

        tracing::info!("DNS configuration updated (manual={})", config.manual);
        Ok(())
    }

    pub async fn update_hostname(&self, hostname: &str) -> anyhow::Result<()> {
        if hostname.is_empty() || hostname.len() > 63 {
            anyhow::bail!("Hostname must be between 1 and 63 characters");
        }

        // Validate hostname characters
        if !hostname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            anyhow::bail!("Hostname can only contain letters, numbers and hyphens");
        }

        if self.dev_mode {
            tracing::info!("[DEV MODE] Would set hostname to: {}", hostname);
            return Ok(());
        }

        // Set hostname via hostnamectl (or echo on LibreELEC)
        let output = AsyncCommand::new("hostname")
            .arg(hostname)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set hostname: {}", stderr);
        }

        // Persist hostname
        let _ = tokio::fs::write("/storage/.cache/hostname", hostname).await;

        tracing::info!("Hostname changed to: {}", hostname);
        Ok(())
    }

    // --- Private helpers ---

    fn dev_status(&self) -> NetworkStatus {
        NetworkStatus {
            hostname: "pinas".to_string(),
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    display_name: "LAN 1".to_string(),
                    status: "connected".to_string(),
                    ip_address: "192.168.1.100".to_string(),
                    subnet_mask: "255.255.255.0".to_string(),
                    mac_address: "dc:a6:32:aa:bb:cc".to_string(),
                    speed: "1000 Mbps".to_string(),
                    method: "dhcp".to_string(),
                    gateway: "192.168.1.1".to_string(),
                    dns: "8.8.8.8".to_string(),
                },
                NetworkInterface {
                    name: "wlan0".to_string(),
                    display_name: "WiFi".to_string(),
                    status: "disconnected".to_string(),
                    ip_address: "".to_string(),
                    subnet_mask: "".to_string(),
                    mac_address: "dc:a6:32:dd:ee:ff".to_string(),
                    speed: "".to_string(),
                    method: "dhcp".to_string(),
                    gateway: "".to_string(),
                    dns: "".to_string(),
                },
            ],
            dns: DnsConfig {
                manual: false,
                primary: "8.8.8.8".to_string(),
                secondary: "8.8.4.4".to_string(),
            },
            default_gateway: "192.168.1.1".to_string(),
        }
    }

    async fn get_hostname(&self) -> anyhow::Result<String> {
        let output = AsyncCommand::new("hostname").output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn get_interfaces(&self) -> anyhow::Result<Vec<NetworkInterface>> {
        let mut interfaces = Vec::new();

        // List network interfaces from /sys/class/net
        let entries = match std::fs::read_dir("/sys/class/net") {
            Ok(e) => e,
            Err(_) => return Ok(interfaces),
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback and virtual interfaces
            if name == "lo" || name.starts_with("veth") || name.starts_with("docker") || name.starts_with("br-") {
                continue;
            }

            let display_name = if name.starts_with("eth") {
                format!("LAN {}", name.trim_start_matches("eth").parse::<u32>().unwrap_or(0) + 1)
            } else if name.starts_with("wlan") {
                "WiFi".to_string()
            } else if name.starts_with("enp") || name.starts_with("ens") {
                format!("LAN ({})", name)
            } else {
                name.clone()
            };

            // Get MAC address
            let mac_address = tokio::fs::read_to_string(format!("/sys/class/net/{}/address", name))
                .await
                .unwrap_or_default()
                .trim()
                .to_string();

            // Get operstate (up/down)
            let operstate = tokio::fs::read_to_string(format!("/sys/class/net/{}/operstate", name))
                .await
                .unwrap_or_default()
                .trim()
                .to_string();

            let status = if operstate == "up" {
                "connected".to_string()
            } else {
                "disconnected".to_string()
            };

            // Get speed (only if connected)
            let speed = if status == "connected" {
                tokio::fs::read_to_string(format!("/sys/class/net/{}/speed", name))
                    .await
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .map(|s| format!("{} Mbps", s))
                    .unwrap_or_default()
            } else {
                String::new()
            };

            // Get IP address and subnet via `ip addr show`
            let (ip_address, subnet_mask) = self.get_interface_ip(&name).await;

            // Get method (DHCP vs static) from connman
            let method = self.get_interface_method(&name).await;

            // Get gateway from route
            let gateway = self.get_interface_gateway(&name).await;

            // Get DNS
            let dns = self.get_interface_dns(&name).await;

            interfaces.push(NetworkInterface {
                name,
                display_name,
                status,
                ip_address,
                subnet_mask,
                mac_address,
                speed,
                method,
                gateway,
                dns,
            });
        }

        // Sort: connected first, then by name
        interfaces.sort_by(|a, b| {
            let a_connected = a.status == "connected";
            let b_connected = b.status == "connected";
            b_connected.cmp(&a_connected).then(a.name.cmp(&b.name))
        });

        Ok(interfaces)
    }

    async fn get_interface_ip(&self, name: &str) -> (String, String) {
        let output = AsyncCommand::new("ip")
            .args(["-4", "addr", "show", name])
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("inet ") {
                    // Parse "inet 192.168.1.100/24 ..."
                    if let Some(addr_cidr) = line.split_whitespace().nth(1) {
                        let parts: Vec<&str> = addr_cidr.split('/').collect();
                        let ip = parts[0].to_string();
                        let mask = parts
                            .get(1)
                            .and_then(|p| p.parse::<u32>().ok())
                            .map(cidr_to_mask)
                            .unwrap_or_else(|| "255.255.255.0".to_string());
                        return (ip, mask);
                    }
                }
            }
        }

        (String::new(), String::new())
    }

    async fn get_interface_method(&self, name: &str) -> String {
        // Try connmanctl to determine if DHCP or manual
        let output = AsyncCommand::new("connmanctl")
            .args(["services"])
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(name) || (name.starts_with("eth") && line.contains("ethernet")) || (name.starts_with("wlan") && line.contains("wifi")) {
                    // Get service ID and query it
                    if let Some(service_id) = line.split_whitespace().last() {
                        let detail = AsyncCommand::new("connmanctl")
                            .args(["services", service_id])
                            .output()
                            .await;

                        if let Ok(detail) = detail {
                            let detail_str = String::from_utf8_lossy(&detail.stdout);
                            for detail_line in detail_str.lines() {
                                if detail_line.contains("IPv4 =") || detail_line.contains("IPv4.Configuration") {
                                    if detail_line.contains("Method=dhcp") || detail_line.contains("Method = dhcp") {
                                        return "dhcp".to_string();
                                    } else if detail_line.contains("Method=manual") || detail_line.contains("Method = manual") {
                                        return "manual".to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        "dhcp".to_string()
    }

    async fn get_interface_gateway(&self, name: &str) -> String {
        let output = AsyncCommand::new("ip")
            .args(["route", "show", "dev", name])
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("default via ") {
                    if let Some(gw) = line.split_whitespace().nth(2) {
                        return gw.to_string();
                    }
                }
            }
        }

        String::new()
    }

    async fn get_interface_dns(&self, _name: &str) -> String {
        // Read from resolv.conf
        if let Ok(content) = tokio::fs::read_to_string("/etc/resolv.conf").await {
            let servers: Vec<&str> = content
                .lines()
                .filter(|l| l.starts_with("nameserver "))
                .filter_map(|l| l.split_whitespace().nth(1))
                .collect();
            return servers.join(", ");
        }

        String::new()
    }

    async fn get_dns_config(&self) -> anyhow::Result<DnsConfig> {
        let mut primary = String::new();
        let mut secondary = String::new();

        if let Ok(content) = tokio::fs::read_to_string("/etc/resolv.conf").await {
            let servers: Vec<String> = content
                .lines()
                .filter(|l| l.starts_with("nameserver "))
                .filter_map(|l| l.split_whitespace().nth(1))
                .map(|s| s.to_string())
                .collect();

            if let Some(s) = servers.first() {
                primary = s.clone();
            }
            if let Some(s) = servers.get(1) {
                secondary = s.clone();
            }
        }

        // Determine if DNS is manually configured
        // On LibreELEC with connman, check if nameservers were manually set
        let manual = self.is_dns_manual().await;

        Ok(DnsConfig {
            manual,
            primary,
            secondary,
        })
    }

    async fn is_dns_manual(&self) -> bool {
        let output = AsyncCommand::new("connmanctl")
            .args(["services"])
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(service_id) = line.split_whitespace().last() {
                    if service_id.starts_with("ethernet_") || service_id.starts_with("wifi_") {
                        let detail = AsyncCommand::new("connmanctl")
                            .args(["services", service_id])
                            .output()
                            .await;

                        if let Ok(detail) = detail {
                            let detail_str = String::from_utf8_lossy(&detail.stdout);
                            for detail_line in detail_str.lines() {
                                if detail_line.contains("Nameservers.Configuration") {
                                    let value = detail_line.split('=').nth(1).unwrap_or("").trim();
                                    if !value.is_empty() && value != "[]" {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    async fn get_default_gateway(&self) -> anyhow::Result<String> {
        let output = AsyncCommand::new("ip")
            .args(["route", "show", "default"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("default via ") {
                if let Some(gw) = line.split_whitespace().nth(2) {
                    return Ok(gw.to_string());
                }
            }
        }

        Ok(String::new())
    }

    async fn find_connman_service(&self, interface_name: &str) -> anyhow::Result<String> {
        let output = AsyncCommand::new("connmanctl")
            .args(["services"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let iface_type = if interface_name.starts_with("eth") || interface_name.starts_with("enp") {
            "ethernet"
        } else if interface_name.starts_with("wlan") {
            "wifi"
        } else {
            interface_name
        };

        for line in stdout.lines() {
            if let Some(service_id) = line.split_whitespace().last() {
                if service_id.contains(iface_type) {
                    return Ok(service_id.to_string());
                }
            }
        }

        anyhow::bail!("No connman service found for interface {}", interface_name)
    }

    async fn find_active_connman_service(&self) -> anyhow::Result<String> {
        let output = AsyncCommand::new("connmanctl")
            .args(["services"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Return first connected service (marked with * or first ethernet/wifi)
        for line in stdout.lines() {
            if let Some(service_id) = line.split_whitespace().last() {
                if service_id.starts_with("ethernet_") || service_id.starts_with("wifi_") {
                    // Check if this service is connected
                    if line.contains("*A") || line.contains("*O") {
                        return Ok(service_id.to_string());
                    }
                }
            }
        }

        // Fallback: return first ethernet or wifi service
        for line in stdout.lines() {
            if let Some(service_id) = line.split_whitespace().last() {
                if service_id.starts_with("ethernet_") || service_id.starts_with("wifi_") {
                    return Ok(service_id.to_string());
                }
            }
        }

        anyhow::bail!("No active connman service found")
    }
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert CIDR prefix length to subnet mask string
fn cidr_to_mask(prefix: u32) -> String {
    if prefix > 32 {
        return "255.255.255.0".to_string();
    }
    let mask: u32 = if prefix == 0 {
        0
    } else {
        !0u32 << (32 - prefix)
    };
    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF
    )
}
