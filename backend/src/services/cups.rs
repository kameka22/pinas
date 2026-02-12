use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::process::Command as AsyncCommand;

/// CUPS service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CupsStatus {
    pub enabled: bool,
    pub running: bool,
    pub printer_count: u32,
}

/// Printer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Printer {
    pub name: String,
    pub uri: String,
    pub state: String,
    pub state_message: String,
    pub shared: bool,
    pub is_default: bool,
    pub model: String,
    pub location: String,
}

/// Detected USB printer (not yet configured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPrinter {
    pub uri: String,
    pub model: String,
}

/// Available driver for a printer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterDriver {
    pub id: String,
    pub name: String,
}

/// Print job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: u32,
    pub printer: String,
    pub title: String,
    pub user: String,
    pub state: String,
    pub size: u64,
    pub created_at: String,
}

/// Add printer request
#[derive(Debug, Clone, Deserialize)]
pub struct AddPrinterRequest {
    pub name: String,
    pub uri: String,
    pub driver: String,
    pub location: Option<String>,
    pub shared: Option<bool>,
}

/// Update printer request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePrinterRequest {
    pub shared: Option<bool>,
    pub is_default: Option<bool>,
    pub location: Option<String>,
}

const CUPS_SERVICE_NAME: &str = "cups";

/// CUPS printing service manager
///
/// On LibreELEC, CUPS is an optional package. The service is disabled by default
/// and must be explicitly enabled by the user.
///
/// Service control via systemctl enable/disable + start/stop.
/// Printer management via lpadmin/lpstat/lp CLI tools.
pub struct CupsService {
    dev_mode: bool,
}

// Dev mode state
static DEV_CUPS_ENABLED: AtomicBool = AtomicBool::new(false);

fn dev_printers() -> &'static Mutex<Vec<Printer>> {
    static INSTANCE: OnceLock<Mutex<Vec<Printer>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(Vec::new()))
}

fn dev_jobs() -> &'static Mutex<Vec<PrintJob>> {
    static INSTANCE: OnceLock<Mutex<Vec<PrintJob>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(Vec::new()))
}

fn dev_job_counter() -> &'static Mutex<u32> {
    static INSTANCE: OnceLock<Mutex<u32>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(0))
}

impl CupsService {
    pub fn new() -> Self {
        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::debug!("CupsService running in dev mode - using fake printer data");
        }

        Self { dev_mode }
    }

    /// Get current CUPS service status
    pub async fn get_status(&self) -> anyhow::Result<CupsStatus> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            let printer_count = if enabled {
                dev_printers().lock().unwrap().len() as u32
            } else {
                0
            };
            return Ok(CupsStatus {
                enabled,
                running: enabled,
                printer_count,
            });
        }

        let running = self.is_cups_running().await;
        let enabled = self.is_cups_enabled().await || running;
        let printer_count = if running {
            self.count_printers().await.unwrap_or(0)
        } else {
            0
        };

        Ok(CupsStatus {
            enabled,
            running,
            printer_count,
        })
    }

    /// Check if CUPS is running
    async fn is_cups_running(&self) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-active", CUPS_SERVICE_NAME])
            .output()
            .await;

        if let Ok(o) = output {
            return String::from_utf8_lossy(&o.stdout).trim() == "active";
        }

        false
    }

    /// Check if CUPS is enabled at boot
    async fn is_cups_enabled(&self) -> bool {
        let output = AsyncCommand::new("systemctl")
            .args(["is-enabled", CUPS_SERVICE_NAME])
            .output()
            .await;

        if let Ok(o) = output {
            return String::from_utf8_lossy(&o.stdout).trim() == "enabled";
        }

        false
    }

    /// Count configured printers
    async fn count_printers(&self) -> anyhow::Result<u32> {
        let output = AsyncCommand::new("lpstat")
            .args(["-p"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(0);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().filter(|l| l.starts_with("printer ")).count();
        Ok(count as u32)
    }

    /// Enable CUPS service
    pub async fn enable(&self) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Enabling CUPS service");
            DEV_CUPS_ENABLED.store(true, Ordering::Relaxed);
            return Ok(());
        }

        // Enable at boot
        let output = AsyncCommand::new("systemctl")
            .args(["enable", CUPS_SERVICE_NAME])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to enable CUPS: {}", stderr);
        }

        // Start the service
        let output = AsyncCommand::new("systemctl")
            .args(["start", CUPS_SERVICE_NAME])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to start CUPS: {}", stderr);
        }

        tracing::info!("CUPS service enabled and started");
        Ok(())
    }

    /// Disable CUPS service
    pub async fn disable(&self) -> anyhow::Result<()> {
        if self.dev_mode {
            tracing::info!("[DEV MODE] Disabling CUPS service");
            DEV_CUPS_ENABLED.store(false, Ordering::Relaxed);
            return Ok(());
        }

        // Stop the service
        let output = AsyncCommand::new("systemctl")
            .args(["stop", CUPS_SERVICE_NAME])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to stop CUPS: {}", stderr);
        }

        // Disable at boot
        let output = AsyncCommand::new("systemctl")
            .args(["disable", CUPS_SERVICE_NAME])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to disable CUPS: {}", stderr);
        }

        tracing::info!("CUPS service disabled");
        Ok(())
    }

    /// Detect USB printers not yet configured
    pub async fn detect_printers(&self) -> anyhow::Result<Vec<DetectedPrinter>> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            if !enabled {
                anyhow::bail!("CUPS service is not enabled");
            }
            // Simulate detecting a USB printer
            return Ok(vec![
                DetectedPrinter {
                    uri: "usb://HP/LaserJet%201020?serial=AB1234".to_string(),
                    model: "HP LaserJet 1020".to_string(),
                },
                DetectedPrinter {
                    uri: "usb://Brother/HL-L2350DW?serial=CD5678".to_string(),
                    model: "Brother HL-L2350DW".to_string(),
                },
            ]);
        }

        let output = AsyncCommand::new("lpinfo")
            .args(["-v"])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to detect printers: lpinfo not available");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut printers = Vec::new();

        for line in stdout.lines() {
            // Format: "direct usb://HP/LaserJet%201020?serial=..."
            if line.contains("usb://") {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let uri = parts[1].trim().to_string();
                    let model = Self::uri_to_model(&uri);
                    printers.push(DetectedPrinter { uri, model });
                }
            }
        }

        Ok(printers)
    }

    /// Extract model name from USB URI
    fn uri_to_model(uri: &str) -> String {
        // usb://HP/LaserJet%201020?serial=... -> HP LaserJet 1020
        if let Some(rest) = uri.strip_prefix("usb://") {
            let path = rest.split('?').next().unwrap_or(rest);
            let decoded = path
                .replace("%20", " ")
                .replace("%2F", "/")
                .replace('/', " ");
            return decoded;
        }
        uri.to_string()
    }

    /// Get list of configured printers
    pub async fn get_printers(&self) -> anyhow::Result<Vec<Printer>> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            if !enabled {
                anyhow::bail!("CUPS service is not enabled");
            }
            return Ok(dev_printers().lock().unwrap().clone());
        }

        let output = AsyncCommand::new("lpstat")
            .args(["-p", "-d", "-l"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut printers = Vec::new();
        let mut current: Option<Printer> = None;

        // Parse lpstat output (also get default printer)
        let default_output = AsyncCommand::new("lpstat")
            .args(["-d"])
            .output()
            .await
            .ok();
        let default_printer = default_output
            .as_ref()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.split(':')
                    .nth(1)
                    .map(|p| p.trim().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        for line in stdout.lines() {
            if line.starts_with("printer ") {
                // Save previous printer
                if let Some(p) = current.take() {
                    printers.push(p);
                }
                // "printer HP_LaserJet is idle.  enabled since ..."
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                let name = parts.get(1).unwrap_or(&"unknown").to_string();
                let state = if line.contains("idle") {
                    "idle"
                } else if line.contains("printing") {
                    "processing"
                } else {
                    "stopped"
                };
                current = Some(Printer {
                    name: name.clone(),
                    uri: String::new(),
                    state: state.to_string(),
                    state_message: String::new(),
                    shared: false,
                    is_default: name == default_printer,
                    model: String::new(),
                    location: String::new(),
                });
            } else if let Some(ref mut p) = current {
                let trimmed = line.trim();
                if trimmed.starts_with("Description:") {
                    p.model = trimmed.trim_start_matches("Description:").trim().to_string();
                } else if trimmed.starts_with("Location:") {
                    p.location = trimmed.trim_start_matches("Location:").trim().to_string();
                } else if trimmed.starts_with("Connection:") {
                    p.uri = trimmed.trim_start_matches("Connection:").trim().to_string();
                }
            }
        }

        if let Some(p) = current.take() {
            printers.push(p);
        }

        Ok(printers)
    }

    /// Get available drivers for a printer URI
    pub async fn get_drivers(&self, uri: &str) -> anyhow::Result<Vec<PrinterDriver>> {
        if self.dev_mode {
            // Simulate drivers list based on URI
            let model = Self::uri_to_model(uri);
            return Ok(vec![
                PrinterDriver {
                    id: format!("drv:///sample.drv/generic.ppd"),
                    name: format!("{} - Generic PostScript", model),
                },
                PrinterDriver {
                    id: format!("gutenprint.5.3://{}/expert", uri.split("://").nth(1).unwrap_or("unknown")),
                    name: format!("{} - Gutenprint", model),
                },
            ]);
        }

        let output = AsyncCommand::new("lpinfo")
            .args(["-m"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let model = Self::uri_to_model(uri).to_lowercase();

        // Filter drivers that match the model
        let keywords: Vec<&str> = model.split_whitespace().collect();

        let mut drivers: Vec<PrinterDriver> = stdout
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                keywords.iter().any(|kw| lower.contains(kw))
            })
            .take(20) // Limit results
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    Some(PrinterDriver {
                        id: parts[0].trim().to_string(),
                        name: parts[1].trim().to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Always add generic driver as fallback
        drivers.push(PrinterDriver {
            id: "drv:///sample.drv/generic.ppd".to_string(),
            name: "Generic PostScript Printer".to_string(),
        });

        Ok(drivers)
    }

    /// Add a new printer
    pub async fn add_printer(&self, req: &AddPrinterRequest) -> anyhow::Result<()> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            if !enabled {
                anyhow::bail!("CUPS service is not enabled");
            }

            let printer = Printer {
                name: req.name.clone(),
                uri: req.uri.clone(),
                state: "idle".to_string(),
                state_message: String::new(),
                shared: req.shared.unwrap_or(true),
                is_default: dev_printers().lock().unwrap().is_empty(),
                model: Self::uri_to_model(&req.uri),
                location: req.location.clone().unwrap_or_default(),
            };

            dev_printers().lock().unwrap().push(printer);
            tracing::info!("[DEV MODE] Added printer: {}", req.name);
            return Ok(());
        }

        // lpadmin -p {name} -v {uri} -m {driver} -E
        let mut cmd = AsyncCommand::new("lpadmin");
        cmd.args(["-p", &req.name, "-v", &req.uri, "-m", &req.driver, "-E"]);

        if let Some(ref loc) = req.location {
            cmd.args(["-L", loc]);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to add printer: {}", stderr);
        }

        // Set sharing
        let shared = req.shared.unwrap_or(true);
        let share_value = if shared { "true" } else { "false" };
        AsyncCommand::new("lpadmin")
            .args(["-p", &req.name, "-o", &format!("printer-is-shared={}", share_value)])
            .output()
            .await?;

        // If first printer, set as default
        if self.count_printers().await.unwrap_or(0) == 1 {
            AsyncCommand::new("lpoptions")
                .args(["-d", &req.name])
                .output()
                .await?;
        }

        tracing::info!("Added printer: {} ({})", req.name, req.uri);
        Ok(())
    }

    /// Remove a printer
    pub async fn remove_printer(&self, name: &str) -> anyhow::Result<()> {
        if self.dev_mode {
            let mut printers = dev_printers().lock().unwrap();
            printers.retain(|p| p.name != name);
            tracing::info!("[DEV MODE] Removed printer: {}", name);
            return Ok(());
        }

        let output = AsyncCommand::new("lpadmin")
            .args(["-x", name])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove printer: {}", stderr);
        }

        tracing::info!("Removed printer: {}", name);
        Ok(())
    }

    /// Update printer settings
    pub async fn update_printer(&self, name: &str, req: &UpdatePrinterRequest) -> anyhow::Result<()> {
        if self.dev_mode {
            let mut printers = dev_printers().lock().unwrap();
            let found = printers.iter().any(|p| p.name == name);
            if !found {
                anyhow::bail!("Printer not found: {}", name);
            }

            if let Some(true) = req.is_default {
                for p in printers.iter_mut() {
                    p.is_default = p.name == name;
                }
            }
            if let Some(p) = printers.iter_mut().find(|p| p.name == name) {
                if let Some(shared) = req.shared {
                    p.shared = shared;
                }
                if let Some(ref loc) = req.location {
                    p.location = loc.clone();
                }
            }
            return Ok(());
        }

        if let Some(shared) = req.shared {
            let value = if shared { "true" } else { "false" };
            AsyncCommand::new("lpadmin")
                .args(["-p", name, "-o", &format!("printer-is-shared={}", value)])
                .output()
                .await?;
        }

        if let Some(true) = req.is_default {
            AsyncCommand::new("lpoptions")
                .args(["-d", name])
                .output()
                .await?;
        }

        if let Some(ref loc) = req.location {
            AsyncCommand::new("lpadmin")
                .args(["-p", name, "-L", loc])
                .output()
                .await?;
        }

        Ok(())
    }

    /// Get print jobs
    pub async fn get_jobs(&self, printer_name: Option<&str>) -> anyhow::Result<Vec<PrintJob>> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            if !enabled {
                anyhow::bail!("CUPS service is not enabled");
            }
            let jobs = dev_jobs().lock().unwrap();
            let filtered: Vec<PrintJob> = if let Some(name) = printer_name {
                jobs.iter().filter(|j| j.printer == name).cloned().collect()
            } else {
                jobs.clone()
            };
            return Ok(filtered);
        }

        let mut cmd = AsyncCommand::new("lpstat");
        cmd.arg("-o");
        if let Some(name) = printer_name {
            cmd.arg(name);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut jobs = Vec::new();

        for line in stdout.lines() {
            // Format: "printer-1  user  1024  Mon 01 Jan 2026 12:00:00  document.pdf"
            let parts: Vec<&str> = line.splitn(4, char::is_whitespace).collect();
            if parts.len() >= 2 {
                let job_name = parts[0]; // "printer-123"
                let (printer, id) = if let Some(pos) = job_name.rfind('-') {
                    let p = &job_name[..pos];
                    let i = job_name[pos + 1..].parse::<u32>().unwrap_or(0);
                    (p.to_string(), i)
                } else {
                    (job_name.to_string(), 0)
                };

                jobs.push(PrintJob {
                    id,
                    printer,
                    title: parts.get(3).unwrap_or(&"Unknown").to_string(),
                    user: parts.get(1).unwrap_or(&"root").to_string(),
                    state: "pending".to_string(),
                    size: parts
                        .get(2)
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0),
                    created_at: String::new(),
                });
            }
        }

        Ok(jobs)
    }

    /// Cancel a print job
    pub async fn cancel_job(&self, job_id: u32) -> anyhow::Result<()> {
        if self.dev_mode {
            let mut jobs = dev_jobs().lock().unwrap();
            jobs.retain(|j| j.id != job_id);
            tracing::info!("[DEV MODE] Cancelled job: {}", job_id);
            return Ok(());
        }

        let output = AsyncCommand::new("cancel")
            .arg(job_id.to_string())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to cancel job: {}", stderr);
        }

        tracing::info!("Cancelled print job: {}", job_id);
        Ok(())
    }

    /// Print a test page
    pub async fn print_test_page(&self, printer_name: &str) -> anyhow::Result<()> {
        if self.dev_mode {
            let enabled = DEV_CUPS_ENABLED.load(Ordering::Relaxed);
            if !enabled {
                anyhow::bail!("CUPS service is not enabled");
            }

            // Simulate adding a job
            let mut counter = dev_job_counter().lock().unwrap();
            *counter += 1;
            let job_id = *counter;

            dev_jobs().lock().unwrap().push(PrintJob {
                id: job_id,
                printer: printer_name.to_string(),
                title: "Test Page".to_string(),
                user: "root".to_string(),
                state: "processing".to_string(),
                size: 4096,
                created_at: chrono::Utc::now().to_rfc3339(),
            });

            tracing::info!("[DEV MODE] Printing test page on: {}", printer_name);
            return Ok(());
        }

        let output = AsyncCommand::new("lp")
            .args(["-d", printer_name, "/usr/share/cups/data/testprint"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to print test page: {}", stderr);
        }

        tracing::info!("Test page sent to: {}", printer_name);
        Ok(())
    }
}

impl Default for CupsService {
    fn default() -> Self {
        Self::new()
    }
}
