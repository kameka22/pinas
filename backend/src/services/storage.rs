use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::storage::*;

/// Protected mount points that cannot be modified
const PROTECTED_MOUNTS: [&str; 2] = ["/flash", "/storage"];

/// Protected device prefixes (system disk on LibreELEC)
const PROTECTED_DEVICE_PREFIXES: [&str; 1] = ["/dev/mmcblk0"];

/// Storage service handles disk enumeration, pools, and volumes
pub struct StorageService {
    db: SqlitePool,
    pools_base_path: String,
    dev_mode: bool,
}

impl StorageService {
    pub fn new(db: SqlitePool) -> Self {
        let pools_base_path = std::env::var("PINAS_POOLS_PATH")
            .unwrap_or_else(|_| "/storage/pools".to_string());

        let dev_mode = std::env::var("PINAS_DEV_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if dev_mode {
            tracing::info!("StorageService running in dev mode - using fake disk data");
        }

        Self { db, pools_base_path, dev_mode }
    }

    // ============ DEV MODE FAKE DATA ============

    /// Generate fake disks for dev mode
    fn get_fake_disks(&self) -> Vec<Disk> {
        vec![
            Disk {
                device_name: "mmcblk0".to_string(),
                device_path: "/dev/mmcblk0".to_string(),
                device_by_id: Some("/dev/disk/by-id/mmc-SD_Card_0x12345678".to_string()),
                model: "SD Card 32GB".to_string(),
                serial: Some("0x12345678".to_string()),
                size: 32 * 1024 * 1024 * 1024, // 32GB
                disk_type: DiskType::Sd,
                temperature: None,
                health_status: None,
                is_system: true,
                is_removable: false,
                partitions: vec![
                    Partition {
                        device_path: "/dev/mmcblk0p1".to_string(),
                        number: 1,
                        size: 512 * 1024 * 1024, // 512MB
                        fs_type: Some("vfat".to_string()),
                        label: Some("LIBREELEC".to_string()),
                        uuid: Some("1234-5678".to_string()),
                        mount_point: Some("/flash".to_string()),
                        is_system: true,
                    },
                    Partition {
                        device_path: "/dev/mmcblk0p2".to_string(),
                        number: 2,
                        size: 31 * 1024 * 1024 * 1024, // ~31GB
                        fs_type: Some("ext4".to_string()),
                        label: Some("STORAGE".to_string()),
                        uuid: Some("abcd-efgh-1234-5678".to_string()),
                        mount_point: Some("/storage".to_string()),
                        is_system: true,
                    },
                ],
            },
            Disk {
                device_name: "sda".to_string(),
                device_path: "/dev/sda".to_string(),
                device_by_id: Some("/dev/disk/by-id/usb-Samsung_T7_1TB_S123456".to_string()),
                model: "Samsung T7 1TB".to_string(),
                serial: Some("S123456".to_string()),
                size: 1000 * 1024 * 1024 * 1024, // 1TB
                disk_type: DiskType::Ssd,
                temperature: Some(35),
                health_status: Some("PASSED".to_string()),
                is_system: false,
                is_removable: true,
                partitions: vec![],
            },
            Disk {
                device_name: "sdb".to_string(),
                device_path: "/dev/sdb".to_string(),
                device_by_id: Some("/dev/disk/by-id/usb-WD_Elements_2TB_WD12345".to_string()),
                model: "WD Elements 2TB".to_string(),
                serial: Some("WD12345".to_string()),
                size: 2000 * 1024 * 1024 * 1024, // 2TB
                disk_type: DiskType::Hdd,
                temperature: Some(42),
                health_status: Some("PASSED".to_string()),
                is_system: false,
                is_removable: true,
                partitions: vec![
                    Partition {
                        device_path: "/dev/sdb1".to_string(),
                        number: 1,
                        size: 2000 * 1024 * 1024 * 1024,
                        fs_type: Some("ext4".to_string()),
                        label: Some("Data".to_string()),
                        uuid: Some("data-uuid-1234".to_string()),
                        mount_point: None,
                        is_system: false,
                    },
                ],
            },
        ]
    }

    /// Generate fake S.M.A.R.T. info for dev mode
    fn get_fake_smart_info(&self, device_name: &str) -> Result<SmartInfo> {
        let (model, serial, temp, hours) = match device_name {
            "sda" | "/dev/sda" => ("Samsung T7 1TB", "S123456", 35, 1234),
            "sdb" | "/dev/sdb" => ("WD Elements 2TB", "WD12345", 42, 5678),
            _ => return Err(anyhow!("No S.M.A.R.T. data available for {}", device_name)),
        };

        Ok(SmartInfo {
            device_path: format!("/dev/{}", device_name.trim_start_matches("/dev/")),
            model: model.to_string(),
            serial: Some(serial.to_string()),
            firmware: Some("1.0".to_string()),
            health_status: "PASSED".to_string(),
            temperature: Some(temp),
            power_on_hours: Some(hours),
            power_cycle_count: Some(123),
            reallocated_sectors: Some(0),
            pending_sectors: Some(0),
            attributes: vec![
                SmartAttribute { id: 1, name: "Raw_Read_Error_Rate".to_string(), value: 100, worst: 100, threshold: 50, raw_value: "0".to_string() },
                SmartAttribute { id: 5, name: "Reallocated_Sector_Ct".to_string(), value: 100, worst: 100, threshold: 10, raw_value: "0".to_string() },
                SmartAttribute { id: 9, name: "Power_On_Hours".to_string(), value: 99, worst: 99, threshold: 0, raw_value: hours.to_string() },
                SmartAttribute { id: 194, name: "Temperature_Celsius".to_string(), value: 100, worst: 100, threshold: 0, raw_value: temp.to_string() },
            ],
        })
    }

    // ============ DISK OPERATIONS ============

    /// List all physical disks with partitions
    pub async fn list_disks(&self) -> Result<Vec<Disk>> {
        // Return fake data in dev mode
        if self.dev_mode {
            return Ok(self.get_fake_disks());
        }

        // Try lsblk first, fall back to sysfs if not available
        match self.list_disks_lsblk().await {
            Ok(disks) => Ok(disks),
            Err(e) => {
                tracing::info!("lsblk not available ({}), using sysfs fallback", e);
                self.list_disks_sysfs().await
            }
        }
    }

    /// List disks using lsblk (standard Linux)
    async fn list_disks_lsblk(&self) -> Result<Vec<Disk>> {
        let output = Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE,LABEL,UUID,MODEL,SERIAL,TRAN,RM,HOTPLUG"])
            .output()
            .await
            .context("Failed to execute lsblk")?;

        if !output.status.success() {
            return Err(anyhow!("lsblk failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let lsblk: LsblkOutput = serde_json::from_str(&json_str)
            .context("Failed to parse lsblk output")?;

        let mut disks = Vec::new();

        for device in lsblk.blockdevices {
            if device.device_type != "disk" {
                continue;
            }

            let device_path = format!("/dev/{}", device.name);
            let device_by_id = self.get_device_by_id(&device.name).await;

            // Check if this is a system disk
            let is_system = self.is_system_device(&device_path, &device.children);

            // Determine disk type
            let disk_type = self.determine_disk_type(&device);

            // Get partitions
            let partitions = self.parse_partitions(&device.name, &device.children);

            // Try to get S.M.A.R.T. info (temperature only for list view)
            let (temperature, health_status) = self.get_basic_smart_info(&device_path).await;

            disks.push(Disk {
                device_name: device.name.clone(),
                device_path,
                device_by_id,
                model: device.model.unwrap_or_else(|| "Unknown".to_string()),
                serial: device.serial,
                size: device.size.unwrap_or(0),
                disk_type,
                temperature,
                health_status,
                is_system,
                is_removable: device.rm.unwrap_or(false) || device.hotplug.unwrap_or(false),
                partitions,
            });
        }

        Ok(disks)
    }

    /// List disks using /sys/block + /proc/partitions + blkid (LibreELEC fallback)
    async fn list_disks_sysfs(&self) -> Result<Vec<Disk>> {
        // Parse /proc/partitions for size info
        let partitions_data = tokio::fs::read_to_string("/proc/partitions").await
            .context("Failed to read /proc/partitions")?;
        let mut part_sizes: HashMap<String, u64> = HashMap::new();
        for line in partitions_data.lines().skip(2) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                let blocks: u64 = fields[2].parse().unwrap_or(0);
                let name = fields[3].to_string();
                part_sizes.insert(name, blocks * 1024); // blocks are 1KB
            }
        }

        // Parse blkid for filesystem info
        let blkid_info = self.parse_blkid().await;

        // Parse /proc/mounts for mount points
        let mounts_data = tokio::fs::read_to_string("/proc/mounts").await
            .unwrap_or_default();
        let mut mount_map: HashMap<String, String> = HashMap::new();
        for line in mounts_data.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 {
                mount_map.insert(fields[0].to_string(), fields[1].to_string());
            }
        }

        // Enumerate block devices from /sys/block
        let mut disks = Vec::new();
        let mut entries = tokio::fs::read_dir("/sys/block").await
            .context("Failed to read /sys/block")?;

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip virtual/pseudo block devices
            if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("dm-")
                || name.starts_with("nbd") || name.starts_with("zram") {
                continue;
            }

            let sys_path = format!("/sys/block/{}", name);
            let device_path = format!("/dev/{}", name);

            // Read device properties from sysfs
            let size = part_sizes.get(&name).copied().unwrap_or(0);

            // Skip devices with zero size (not connected)
            if size == 0 {
                continue;
            }
            let model = Self::read_sysfs_str(&format!("{}/device/model", sys_path)).await;
            let serial = Self::read_sysfs_str(&format!("{}/device/serial", sys_path)).await;
            let removable = Self::read_sysfs_u8(&format!("{}/removable", sys_path)).await == 1;

            // Determine disk type from name
            let disk_type = if name.starts_with("nvme") {
                DiskType::Nvme
            } else if name.starts_with("mmcblk") {
                DiskType::Sd
            } else if removable {
                DiskType::Usb
            } else if model.as_ref().map(|m| m.to_uppercase().contains("SSD")).unwrap_or(false) {
                DiskType::Ssd
            } else {
                DiskType::Hdd
            };

            // Enumerate partitions from /sys/block/{name}/{name}*
            let mut partitions = Vec::new();
            let mut child_devices: Vec<LsblkDevice> = Vec::new();
            if let Ok(mut part_entries) = tokio::fs::read_dir(&sys_path).await {
                let mut part_num: u32 = 0;
                while let Some(pentry) = part_entries.next_entry().await? {
                    let pname = pentry.file_name().to_string_lossy().to_string();
                    // Partitions are named like sda1, mmcblk0p1, nvme0n1p1
                    if !pname.starts_with(&name) || pname == name {
                        continue;
                    }
                    // Verify it's actually a partition (has a 'partition' file)
                    let partition_file = format!("{}/{}/partition", sys_path, pname);
                    if !Path::new(&partition_file).exists() {
                        continue;
                    }

                    part_num += 1;
                    let pdev_path = format!("/dev/{}", pname);
                    let psize = part_sizes.get(&pname).copied().unwrap_or(0);
                    let bi = blkid_info.get(&pdev_path);
                    let pmount = mount_map.get(&pdev_path).cloned();

                    let is_system_part = pmount.as_ref()
                        .map(|m| PROTECTED_MOUNTS.contains(&m.as_str()))
                        .unwrap_or(false);

                    partitions.push(Partition {
                        device_path: pdev_path.clone(),
                        number: part_num,
                        size: psize,
                        fs_type: bi.and_then(|b| b.fs_type.clone()),
                        label: bi.and_then(|b| b.label.clone()),
                        uuid: bi.and_then(|b| b.uuid.clone()),
                        mount_point: pmount.clone(),
                        is_system: is_system_part,
                    });

                    // Also build LsblkDevice for is_system_device check
                    child_devices.push(LsblkDevice {
                        name: pname,
                        size: Some(psize),
                        device_type: "part".to_string(),
                        mountpoint: pmount,
                        fstype: bi.and_then(|b| b.fs_type.clone()),
                        label: bi.and_then(|b| b.label.clone()),
                        uuid: bi.and_then(|b| b.uuid.clone()),
                        model: None,
                        serial: None,
                        tran: None,
                        rm: None,
                        hotplug: None,
                        children: None,
                    });
                }
            }

            let children = if child_devices.is_empty() { None } else { Some(child_devices) };
            let is_system = self.is_system_device(&device_path, &children);
            let device_by_id = self.get_device_by_id(&name).await;
            let (temperature, health_status) = self.get_basic_smart_info(&device_path).await;

            disks.push(Disk {
                device_name: name,
                device_path,
                device_by_id,
                model: model.unwrap_or_else(|| "Unknown".to_string()),
                serial,
                size,
                disk_type,
                temperature,
                health_status,
                is_system,
                is_removable: removable,
                partitions,
            });
        }

        Ok(disks)
    }

    /// Parse blkid output into a map of device -> BlkidInfo
    async fn parse_blkid(&self) -> HashMap<String, BlkidInfo> {
        let mut info = HashMap::new();

        let output = Command::new("blkid")
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Format: /dev/mmcblk0p1: LABEL="LIBREELEC" UUID="..." TYPE="vfat" ...
                let Some((dev, attrs)) = line.split_once(':') else { continue };
                let dev = dev.trim().to_string();

                let mut bi = BlkidInfo { fs_type: None, label: None, uuid: None };

                for part in attrs.split_whitespace() {
                    if let Some(val) = part.strip_prefix("TYPE=") {
                        bi.fs_type = Some(val.trim_matches('"').to_string());
                    } else if let Some(val) = part.strip_prefix("LABEL=") {
                        bi.label = Some(val.trim_matches('"').to_string());
                    } else if let Some(val) = part.strip_prefix("UUID=") {
                        bi.uuid = Some(val.trim_matches('"').to_string());
                    }
                }

                info.insert(dev, bi);
            }
        }

        info
    }

    /// Read a sysfs file as trimmed string
    async fn read_sysfs_str(path: &str) -> Option<String> {
        tokio::fs::read_to_string(path).await.ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// Read a sysfs file as u8
    async fn read_sysfs_u8(path: &str) -> u8 {
        tokio::fs::read_to_string(path).await.ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    /// Get detailed S.M.A.R.T. information for a disk
    pub async fn get_smart_info(&self, device_name: &str) -> Result<SmartInfo> {
        // Return fake data in dev mode
        if self.dev_mode {
            return self.get_fake_smart_info(device_name);
        }

        let device_path = if device_name.starts_with("/dev/") {
            device_name.to_string()
        } else {
            format!("/dev/{}", device_name)
        };

        // Check if device is protected
        if self.is_protected_device(&device_path) {
            // Still allow reading S.M.A.R.T. for system disks
        }

        let output = Command::new("smartctl")
            .args(["-a", "-j", &device_path])
            .output()
            .await
            .context("Failed to execute smartctl")?;

        // smartctl can return non-zero even with valid data
        let json_str = String::from_utf8_lossy(&output.stdout);

        if json_str.trim().is_empty() {
            return Err(anyhow!("No S.M.A.R.T. data available for {}", device_path));
        }

        let smart: SmartctlOutput = serde_json::from_str(&json_str)
            .context("Failed to parse smartctl output")?;

        let attributes = smart.ata_smart_attributes
            .map(|attrs| {
                attrs.table.into_iter().map(|attr| SmartAttribute {
                    id: attr.id,
                    name: attr.name,
                    value: attr.value,
                    worst: attr.worst,
                    threshold: attr.thresh,
                    raw_value: attr.raw.value.to_string(),
                }).collect()
            })
            .unwrap_or_default();

        Ok(SmartInfo {
            device_path,
            model: smart.model_name.unwrap_or_else(|| "Unknown".to_string()),
            serial: smart.serial_number,
            firmware: smart.firmware_version,
            health_status: smart.smart_status
                .map(|s| if s.passed { "PASSED".to_string() } else { "FAILED".to_string() })
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            temperature: smart.temperature.map(|t| t.current),
            power_on_hours: smart.power_on_time.map(|p| p.hours),
            power_cycle_count: smart.power_cycle_count,
            reallocated_sectors: None, // Would need to extract from attributes
            pending_sectors: None,
            attributes,
        })
    }

    /// Get disks available for creating new pools
    pub async fn get_candidates(&self) -> Result<Vec<DiskCandidate>> {
        let disks = self.list_disks().await?;
        let mut candidates = Vec::new();

        for disk in disks {
            // Skip system disks
            if disk.is_system {
                continue;
            }

            // Check if disk is empty (no partitions or all unmounted)
            let is_empty = disk.partitions.is_empty() ||
                disk.partitions.iter().all(|p| p.mount_point.is_none());

            candidates.push(DiskCandidate {
                device_path: disk.device_path,
                device_by_id: disk.device_by_id,
                model: disk.model,
                size: disk.size,
                disk_type: disk.disk_type,
                is_empty,
            });
        }

        Ok(candidates)
    }

    /// Wipe a disk (destroy all data)
    pub async fn wipe_disk(&self, device_name: &str) -> Result<()> {
        let device_path = if device_name.starts_with("/dev/") {
            device_name.to_string()
        } else {
            format!("/dev/{}", device_name)
        };

        // Check if device is protected
        if self.is_protected_device(&device_path) {
            return Err(anyhow!("Cannot wipe system disk: {}", device_path));
        }

        // In dev mode, just log and return success
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would wipe disk: {}", device_path);
            return Ok(());
        }

        // Unmount any mounted partitions first
        let disks = self.list_disks().await?;
        for disk in disks {
            if disk.device_path == device_path {
                for partition in disk.partitions {
                    if partition.mount_point.is_some() {
                        self.unmount(&partition.device_path).await?;
                    }
                }
            }
        }

        // Wipe partition table with sgdisk
        let output = Command::new("sgdisk")
            .args(["--zap-all", &device_path])
            .output()
            .await
            .context("Failed to execute sgdisk")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to wipe disk: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Notify kernel of partition table changes
        let _ = Command::new("partprobe")
            .arg(&device_path)
            .output()
            .await;

        Ok(())
    }

    // ============ POOL OPERATIONS ============

    /// List all storage pools
    pub async fn list_pools(&self) -> Result<Vec<StoragePoolInfo>> {
        let pools: Vec<StoragePool> = sqlx::query_as(
            "SELECT * FROM storage_pools ORDER BY name"
        )
        .fetch_all(&self.db)
        .await?;

        let mut pool_infos = Vec::new();

        for pool in pools {
            let volumes = self.list_volumes_for_pool(&pool.id).await?;

            // Parse devices from JSON
            let devices: Vec<String> = serde_json::from_str(&pool.devices)
                .unwrap_or_default();

            // Calculate usage
            let (used_size, available_size) = self.calculate_pool_usage(&pool.id).await;
            let total_size = pool.total_size.unwrap_or(0) as u64;

            pool_infos.push(StoragePoolInfo {
                id: pool.id,
                name: pool.name,
                description: pool.description,
                raid_type: pool.raid_type.parse().unwrap_or(RaidType::Basic),
                status: match pool.status.as_str() {
                    "normal" => PoolStatus::Normal,
                    "degraded" => PoolStatus::Degraded,
                    "rebuilding" => PoolStatus::Rebuilding,
                    "expanding" => PoolStatus::Expanding,
                    "creating" => PoolStatus::Creating,
                    _ => PoolStatus::Error,
                },
                devices,
                total_size,
                used_size,
                available_size,
                volumes,
                created_at: pool.created_at,
            });
        }

        Ok(pool_infos)
    }

    /// Get a single pool by ID
    pub async fn get_pool(&self, pool_id: &str) -> Result<Option<StoragePoolInfo>> {
        let pools = self.list_pools().await?;
        Ok(pools.into_iter().find(|p| p.id == pool_id))
    }

    /// Create a new storage pool
    pub async fn create_pool(&self, request: CreatePoolRequest) -> Result<String> {
        // Validate devices (even in dev mode, to catch logic errors)
        for device in &request.devices {
            if self.is_protected_device(device) {
                return Err(anyhow!("Cannot use system device in pool: {}", device));
            }
        }

        let pool_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Calculate total size from devices
        let total_size = self.calculate_devices_total_size(&request.devices).await;

        // In dev mode, skip actual disk operations
        let metadata = if self.dev_mode {
            tracing::info!("[DEV MODE] Would create pool '{}' with devices: {:?}", request.name, request.devices);
            // Return fake metadata based on raid type
            match request.raid_type {
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid5 | RaidType::BtrfsRaid6 | RaidType::BtrfsRaid10 => {
                    serde_json::json!({ "btrfs_uuid": format!("fake-btrfs-{}", &pool_id[..8]) }).to_string()
                }
                RaidType::Jbod | RaidType::Raid0 | RaidType::Raid1 | RaidType::Raid5 | RaidType::Raid6 | RaidType::Raid10 => {
                    serde_json::json!({ "md_device": "/dev/md0" }).to_string()
                }
                _ => "{}".to_string(),
            }
        } else {
            // Wipe devices if requested
            if request.wipe_devices {
                for device in &request.devices {
                    self.wipe_disk(device).await?;
                }
            }

            // Create pool based on RAID type
            match request.raid_type {
                RaidType::Basic => {
                    // Single device, just partition it
                    if request.devices.len() != 1 {
                        return Err(anyhow!("Basic pool requires exactly one device"));
                    }
                    self.create_basic_pool(&request.devices[0]).await?
                }
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid5 | RaidType::BtrfsRaid6 | RaidType::BtrfsRaid10 => {
                    self.create_btrfs_pool(&request.devices, &request.raid_type).await?
                }
                _ => {
                    // mdadm RAID - check if available
                    self.create_mdadm_pool(&request.devices, &request.raid_type).await?
                }
            }
        };

        // Insert into database
        let devices_json = serde_json::to_string(&request.devices)?;

        sqlx::query(
            "INSERT INTO storage_pools (id, name, description, raid_type, status, devices, total_size, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'normal', ?, ?, ?, ?, ?)"
        )
        .bind(&pool_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.raid_type.to_string())
        .bind(&devices_json)
        .bind(total_size as i64)
        .bind(&metadata)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(pool_id)
    }

    /// Update pool metadata
    pub async fn update_pool(&self, pool_id: &str, request: UpdatePoolRequest) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(name) = request.name {
            sqlx::query("UPDATE storage_pools SET name = ?, updated_at = ? WHERE id = ?")
                .bind(&name)
                .bind(&now)
                .bind(pool_id)
                .execute(&self.db)
                .await?;
        }

        if let Some(description) = request.description {
            sqlx::query("UPDATE storage_pools SET description = ?, updated_at = ? WHERE id = ?")
                .bind(&description)
                .bind(&now)
                .bind(pool_id)
                .execute(&self.db)
                .await?;
        }

        Ok(())
    }

    /// Delete a storage pool
    pub async fn delete_pool(&self, pool_id: &str) -> Result<()> {
        // Check if pool has mounted volumes (skip in dev mode for flexibility)
        let volumes = self.list_volumes_for_pool(pool_id).await?;
        if !self.dev_mode {
            for volume in &volumes {
                if volume.status == VolumeStatus::Mounted {
                    return Err(anyhow!("Cannot delete pool with mounted volumes. Unmount {} first.", volume.name));
                }
            }
        }

        // Get pool info
        let pool: Option<StoragePool> = sqlx::query_as(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(pool_id)
        .fetch_optional(&self.db)
        .await?;

        let pool = pool.ok_or_else(|| anyhow!("Pool not found"))?;

        if self.dev_mode {
            tracing::info!("[DEV MODE] Would delete pool '{}'", pool.name);
        }

        // Clean up volumes
        for volume in volumes {
            self.delete_volume(&volume.id).await?;
        }

        // Delete from database (cascade will delete volumes)
        sqlx::query("DELETE FROM storage_pools WHERE id = ?")
            .bind(pool_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ============ VOLUME OPERATIONS ============

    /// List all volumes
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        let volumes: Vec<StorageVolume> = sqlx::query_as(
            "SELECT * FROM storage_volumes ORDER BY name"
        )
        .fetch_all(&self.db)
        .await?;

        let mut volume_infos = Vec::new();
        for volume in volumes {
            volume_infos.push(self.volume_to_info(volume).await);
        }

        Ok(volume_infos)
    }

    /// List volumes for a specific pool
    pub async fn list_volumes_for_pool(&self, pool_id: &str) -> Result<Vec<VolumeInfo>> {
        let volumes: Vec<StorageVolume> = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE pool_id = ? ORDER BY name"
        )
        .bind(pool_id)
        .fetch_all(&self.db)
        .await?;

        let mut volume_infos = Vec::new();
        for volume in volumes {
            volume_infos.push(self.volume_to_info(volume).await);
        }

        Ok(volume_infos)
    }

    /// Create a volume in a pool
    pub async fn create_volume(&self, pool_id: &str, request: CreateVolumeRequest) -> Result<String> {
        let pool = self.get_pool(pool_id).await?
            .ok_or_else(|| anyhow!("Pool not found"))?;

        let volume_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Create mount point path
        let mount_point = format!("{}/{}/{}", self.pools_base_path, pool.name, request.name);

        // In dev mode, skip actual filesystem operations
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would create volume '{}' in pool '{}' at {}", request.name, pool.name, mount_point);
        } else {
            // Create the directory
            tokio::fs::create_dir_all(&mount_point).await
                .context("Failed to create mount point directory")?;

            // For basic pools, create filesystem on the device
            // For btrfs pools, create subvolume
            match pool.raid_type {
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid10 => {
                    self.create_btrfs_subvolume(&pool, &request.name, &mount_point).await?;
                }
                _ => {
                    // Create filesystem on pool device
                    if let Some(device) = pool.devices.first() {
                        self.create_filesystem(device, &request.fs_type).await?;
                        self.mount(device, &mount_point, Some(&request.fs_type), request.mount_options.as_deref()).await?;
                    }
                }
            }
        }

        // Insert into database
        sqlx::query(
            "INSERT INTO storage_volumes (id, pool_id, name, fs_type, mount_point, mount_options, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 'mounted', ?, ?)"
        )
        .bind(&volume_id)
        .bind(pool_id)
        .bind(&request.name)
        .bind(&request.fs_type)
        .bind(&mount_point)
        .bind(&request.mount_options)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(volume_id)
    }

    /// Delete a volume
    pub async fn delete_volume(&self, volume_id: &str) -> Result<()> {
        let volume: Option<StorageVolume> = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_optional(&self.db)
        .await?;

        let volume = volume.ok_or_else(|| anyhow!("Volume not found"))?;

        // In dev mode, skip actual filesystem operations
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would delete volume '{}' at {}", volume.name, volume.mount_point);
        } else {
            // Unmount if mounted
            if volume.status == "mounted" {
                self.unmount(&volume.mount_point).await?;
            }

            // Remove mount point directory
            if Path::new(&volume.mount_point).exists() {
                tokio::fs::remove_dir_all(&volume.mount_point).await
                    .context("Failed to remove mount point directory")?;
            }
        }

        // Delete from database
        sqlx::query("DELETE FROM storage_volumes WHERE id = ?")
            .bind(volume_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Mount a volume
    pub async fn mount_volume(&self, volume_id: &str) -> Result<()> {
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await?;

        // In dev mode, skip actual mount operation
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would mount volume '{}' at {}", volume.name, volume.mount_point);
        } else {
            // Get pool info to find device
            let pool: StoragePool = sqlx::query_as(
                "SELECT * FROM storage_pools WHERE id = ?"
            )
            .bind(&volume.pool_id)
            .fetch_one(&self.db)
            .await?;

            let devices: Vec<String> = serde_json::from_str(&pool.devices)?;
            if let Some(device) = devices.first() {
                self.mount(device, &volume.mount_point, Some(&volume.fs_type), volume.mount_options.as_deref()).await?;
            }
        }

        // Update status
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE storage_volumes SET status = 'mounted', updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(volume_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Unmount a volume
    pub async fn unmount_volume(&self, volume_id: &str) -> Result<()> {
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await?;

        // In dev mode, skip actual unmount operation
        if self.dev_mode {
            tracing::info!("[DEV MODE] Would unmount volume '{}' at {}", volume.name, volume.mount_point);
        } else {
            self.unmount(&volume.mount_point).await?;
        }

        // Update status
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE storage_volumes SET status = 'unmounted', updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(volume_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ============ UPDATE VOLUME ============

    /// Update volume settings (mount options)
    pub async fn update_volume(&self, volume_id: &str, request: UpdateVolumeRequest) -> Result<()> {
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await
        .context("Volume not found")?;

        let now = chrono::Utc::now().to_rfc3339();

        if let Some(ref mount_options) = request.mount_options {
            sqlx::query("UPDATE storage_volumes SET mount_options = ?, updated_at = ? WHERE id = ?")
                .bind(mount_options)
                .bind(&now)
                .bind(volume_id)
                .execute(&self.db)
                .await?;

            // If volume is mounted, remount with new options
            if volume.status == "mounted" && !self.dev_mode {
                let _ = Command::new("mount")
                    .args(["-o", &format!("remount,{}", mount_options), &volume.mount_point])
                    .output()
                    .await;
            }

            if self.dev_mode {
                tracing::info!("[DEV MODE] Would update mount options for volume '{}' to '{}'", volume.name, mount_options);
            }
        }

        Ok(())
    }

    // ============ RESIZE VOLUME ============

    /// Resize a volume
    pub async fn resize_volume(&self, volume_id: &str, request: ResizeVolumeRequest) -> Result<()> {
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await
        .context("Volume not found")?;

        let pool: StoragePool = sqlx::query_as(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(&volume.pool_id)
        .fetch_one(&self.db)
        .await
        .context("Pool not found")?;

        let devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        let device = devices.first().ok_or_else(|| anyhow!("No devices in pool"))?;
        let part_device = format!("{}1", device);

        if self.dev_mode {
            let size_str = request.size.map(|s| format!("{}", s)).unwrap_or_else(|| "max".to_string());
            tracing::info!("[DEV MODE] Would resize volume '{}' to {}", volume.name, size_str);

            // Update DB with new size if specified
            if let Some(new_size) = request.size {
                let now = chrono::Utc::now().to_rfc3339();
                sqlx::query("UPDATE storage_volumes SET size_bytes = ?, updated_at = ? WHERE id = ?")
                    .bind(new_size as i64)
                    .bind(&now)
                    .bind(volume_id)
                    .execute(&self.db)
                    .await?;
            }
            return Ok(());
        }

        match volume.fs_type.as_str() {
            "ext4" => {
                if let Some(new_size) = request.size {
                    // Get current usage
                    let (_, used, _) = self.get_mount_stats(&volume.mount_point).await;
                    if new_size < used {
                        return Err(anyhow!("Cannot shrink below used space ({} used)", used));
                    }
                    // Shrink requires unmount
                    if volume.status == "mounted" {
                        return Err(anyhow!("ext4 shrink requires volume to be unmounted first"));
                    }
                    let size_k = format!("{}K", new_size / 1024);
                    let output = Command::new("resize2fs")
                        .args([&part_device, &size_k])
                        .output()
                        .await?;
                    if !output.status.success() {
                        return Err(anyhow!("resize2fs failed: {}", String::from_utf8_lossy(&output.stderr)));
                    }
                } else {
                    // Grow to max (works online)
                    let output = Command::new("resize2fs")
                        .arg(&part_device)
                        .output()
                        .await?;
                    if !output.status.success() {
                        return Err(anyhow!("resize2fs failed: {}", String::from_utf8_lossy(&output.stderr)));
                    }
                }
            }
            "btrfs" => {
                // btrfs resize works online (grow + shrink)
                let size_arg = match request.size {
                    Some(s) => format!("{}", s),
                    None => "max".to_string(),
                };
                let output = Command::new("btrfs")
                    .args(["filesystem", "resize", &size_arg, &volume.mount_point])
                    .output()
                    .await?;
                if !output.status.success() {
                    return Err(anyhow!("btrfs resize failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            "xfs" => {
                // xfs only supports grow, online
                if request.size.is_some() {
                    return Err(anyhow!("XFS does not support shrinking"));
                }
                let output = Command::new("xfs_growfs")
                    .arg(&volume.mount_point)
                    .output()
                    .await?;
                if !output.status.success() {
                    return Err(anyhow!("xfs_growfs failed: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            "f2fs" => {
                return Err(anyhow!("F2FS does not support online resize"));
            }
            _ => {
                return Err(anyhow!("Unsupported filesystem for resize: {}", volume.fs_type));
            }
        }

        Ok(())
    }

    // ============ FILESYSTEM CHECK (FSCK) ============

    /// Start a filesystem check (returns task_id immediately)
    pub async fn fsck_volume_start(&self, volume_id: &str, repair: bool) -> Result<FsckStatus> {
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await
        .context("Volume not found")?;

        // Must be unmounted
        if volume.status == "mounted" {
            return Err(anyhow!("Volume must be unmounted before filesystem check"));
        }

        let task_id = Uuid::new_v4().to_string();

        Ok(FsckStatus {
            task_id,
            volume_id: volume_id.to_string(),
            status: "running".to_string(),
            errors_found: 0,
            repaired: 0,
        })
    }

    /// Execute filesystem check in background
    pub async fn fsck_volume_execute(
        db: SqlitePool,
        volume_id: String,
        task_id: String,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        repair: bool,
        dev_mode: bool,
    ) {
        let volume: StorageVolume = match sqlx::query_as::<_, StorageVolume>(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(&volume_id)
        .fetch_one(&db)
        .await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Fsck: volume not found: {}", e);
                return;
            }
        };

        let pool: StoragePool = match sqlx::query_as::<_, StoragePool>(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(&volume.pool_id)
        .fetch_one(&db)
        .await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Fsck: pool not found: {}", e);
                return;
            }
        };

        let devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        let device = match devices.first() {
            Some(d) => format!("{}1", d),
            None => {
                tracing::error!("Fsck: no devices in pool");
                return;
            }
        };

        // Send initial progress
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: volume_id.clone(),
            status: "running".to_string(),
            progress: 0,
            total_steps: 1,
            progress_percent: 0,
            current_step: Some("Starting filesystem check...".to_string()),
            error_message: None,
        });

        if dev_mode {
            // Simulate fsck progress
            for pct in (0..=100).step_by(10) {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: volume_id.clone(),
                    status: "running".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: pct,
                    current_step: Some(format!("Checking filesystem... {}%", pct)),
                    error_message: None,
                });
            }
        } else {
            let result = match volume.fs_type.as_str() {
                "ext4" => {
                    let mut args = vec!["-f".to_string()];
                    if repair {
                        args.push("-y".to_string());
                    } else {
                        args.push("-n".to_string());
                    }
                    args.push(device.clone());
                    Command::new("e2fsck").args(&args).output().await
                }
                "btrfs" => {
                    let mut args = vec!["check".to_string()];
                    if repair {
                        args.push("--repair".to_string());
                    }
                    // btrfs check uses the raw device
                    let raw_device = devices.first().cloned().unwrap_or_default();
                    args.push(raw_device);
                    Command::new("btrfs").args(&args).output().await
                }
                "xfs" => {
                    let mut args = Vec::new();
                    if !repair {
                        args.push("-n".to_string());
                    }
                    // xfs_repair uses the raw device
                    let raw_device = devices.first().cloned().unwrap_or_default();
                    args.push(raw_device);
                    Command::new("xfs_repair").args(&args).output().await
                }
                _ => {
                    let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                        task_id: task_id.clone(),
                        package_id: volume_id.clone(),
                        status: "failed".to_string(),
                        progress: 1,
                        total_steps: 1,
                        progress_percent: 100,
                        current_step: None,
                        error_message: Some(format!("Unsupported filesystem for check: {}", volume.fs_type)),
                    });
                    return;
                }
            };

            match result {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Fsck returned non-zero (may indicate errors found): {}", stderr);
                    }
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    tracing::info!("Fsck output: {}", stdout);
                }
                Err(e) => {
                    let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                        task_id: task_id.clone(),
                        package_id: volume_id.clone(),
                        status: "failed".to_string(),
                        progress: 1,
                        total_steps: 1,
                        progress_percent: 100,
                        current_step: None,
                        error_message: Some(e.to_string()),
                    });
                    return;
                }
            }
        }

        // Send completion
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: volume_id.clone(),
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 100,
            current_step: Some("Filesystem check completed".to_string()),
            error_message: None,
        });
    }

    // ============ SECURE WIPE ============

    /// Quick wipe (synchronous, existing logic)
    pub async fn wipe_disk_quick(&self, device_name: &str) -> Result<()> {
        // This is the existing wipe_disk logic
        self.wipe_disk(device_name).await
    }

    /// Start a background wipe (Zeros or Secure mode)
    pub fn wipe_disk_start(&self, device_name: &str) -> Result<WipeStatus> {
        let device_path = if device_name.starts_with("/dev/") {
            device_name.to_string()
        } else {
            format!("/dev/{}", device_name)
        };

        if self.is_protected_device(&device_path) {
            return Err(anyhow!("Cannot wipe system disk: {}", device_path));
        }

        let task_id = Uuid::new_v4().to_string();

        Ok(WipeStatus {
            task_id,
            device_name: device_name.to_string(),
            status: "running".to_string(),
            progress: 0.0,
        })
    }

    /// Execute background wipe (Zeros or Secure)
    pub async fn wipe_disk_execute(
        device_name: String,
        mode: WipeMode,
        task_id: String,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        dev_mode: bool,
    ) {
        let device_path = if device_name.starts_with("/dev/") {
            device_name.clone()
        } else {
            format!("/dev/{}", device_name)
        };

        // Send initial progress
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: device_name.clone(),
            status: "running".to_string(),
            progress: 0,
            total_steps: 1,
            progress_percent: 0,
            current_step: Some("Starting wipe...".to_string()),
            error_message: None,
        });

        if dev_mode {
            let steps = match mode {
                WipeMode::Zeros => 10,
                WipeMode::Secure => 20,
                _ => 5,
            };
            for i in 0..=steps {
                let pct = (i * 100) / steps;
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let step_msg = match mode {
                    WipeMode::Zeros => format!("Writing zeros... {}%", pct),
                    WipeMode::Secure => format!("Secure erasing (pass {}/{})... {}%", (i * 3 / steps) + 1, 3, pct),
                    _ => format!("Wiping... {}%", pct),
                };
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: device_name.clone(),
                    status: "running".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: pct,
                    current_step: Some(step_msg),
                    error_message: None,
                });
            }
        } else {
            let result = match mode {
                WipeMode::Zeros => {
                    // dd if=/dev/zero of=/dev/sdX bs=1M status=progress
                    let mut child = match Command::new("dd")
                        .args(["if=/dev/zero", &format!("of={}", device_path), "bs=1M", "status=progress"])
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                                task_id: task_id.clone(),
                                package_id: device_name.clone(),
                                status: "failed".to_string(),
                                progress: 1, total_steps: 1, progress_percent: 100,
                                current_step: None,
                                error_message: Some(e.to_string()),
                            });
                            return;
                        }
                    };
                    // dd writes progress to stderr — we just wait for completion
                    child.wait().await
                }
                WipeMode::Secure => {
                    // shred -vfz -n 3 /dev/sdX
                    let mut child = match Command::new("shred")
                        .args(["-vfz", "-n", "3", &device_path])
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                                task_id: task_id.clone(),
                                package_id: device_name.clone(),
                                status: "failed".to_string(),
                                progress: 1, total_steps: 1, progress_percent: 100,
                                current_step: None,
                                error_message: Some(e.to_string()),
                            });
                            return;
                        }
                    };
                    child.wait().await
                }
                _ => unreachable!("Quick mode handled before background execution"),
            };

            if let Err(e) = result {
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: device_name.clone(),
                    status: "failed".to_string(),
                    progress: 1, total_steps: 1, progress_percent: 100,
                    current_step: None,
                    error_message: Some(e.to_string()),
                });
                return;
            }
        }

        // Send completion
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: device_name.clone(),
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 100,
            current_step: Some("Wipe completed".to_string()),
            error_message: None,
        });
    }

    // ============ HELPER METHODS ============

    /// Check if a device path is protected (system disk)
    fn is_protected_device(&self, device_path: &str) -> bool {
        for prefix in PROTECTED_DEVICE_PREFIXES {
            if device_path.starts_with(prefix) {
                return true;
            }
        }
        false
    }

    /// Check if a device contains system partitions
    fn is_system_device(&self, device_path: &str, children: &Option<Vec<LsblkDevice>>) -> bool {
        // Check device prefix
        if self.is_protected_device(device_path) {
            return true;
        }

        // Check if any partition is mounted on protected paths
        if let Some(parts) = children {
            for part in parts {
                if let Some(mount) = &part.mountpoint {
                    if PROTECTED_MOUNTS.contains(&mount.as_str()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Determine disk type from lsblk output
    fn determine_disk_type(&self, device: &LsblkDevice) -> DiskType {
        if device.name.starts_with("nvme") {
            return DiskType::Nvme;
        }
        if device.name.starts_with("mmcblk") {
            return DiskType::Sd;
        }

        match device.tran.as_deref() {
            Some("usb") => DiskType::Usb,
            Some("sata") | Some("ata") => {
                // Could be SSD or HDD - check for rotation
                // For now, assume SSD if model contains "SSD"
                if device.model.as_ref().map(|m| m.to_uppercase().contains("SSD")).unwrap_or(false) {
                    DiskType::Ssd
                } else {
                    DiskType::Hdd
                }
            }
            _ => DiskType::Unknown,
        }
    }

    /// Parse partitions from lsblk children
    fn parse_partitions(&self, disk_name: &str, children: &Option<Vec<LsblkDevice>>) -> Vec<Partition> {
        let mut partitions = Vec::new();

        if let Some(parts) = children {
            for (idx, part) in parts.iter().enumerate() {
                if part.device_type != "part" {
                    continue;
                }

                let is_system = part.mountpoint.as_ref()
                    .map(|m| PROTECTED_MOUNTS.contains(&m.as_str()))
                    .unwrap_or(false);

                partitions.push(Partition {
                    device_path: format!("/dev/{}", part.name),
                    number: (idx + 1) as u32,
                    size: part.size.unwrap_or(0),
                    fs_type: part.fstype.clone(),
                    label: part.label.clone(),
                    uuid: part.uuid.clone(),
                    mount_point: part.mountpoint.clone(),
                    is_system,
                });
            }
        }

        partitions
    }

    /// Get device by-id path
    async fn get_device_by_id(&self, device_name: &str) -> Option<String> {
        let by_id_path = "/dev/disk/by-id";

        if let Ok(mut entries) = tokio::fs::read_dir(by_id_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(link) = tokio::fs::read_link(entry.path()).await {
                    let link_str = link.to_string_lossy();
                    if link_str.ends_with(device_name) || link_str.contains(&format!("/{}", device_name)) {
                        return Some(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }

        None
    }

    /// Get basic S.M.A.R.T. info (temperature and health only)
    async fn get_basic_smart_info(&self, device_path: &str) -> (Option<i32>, Option<String>) {
        let output = Command::new("smartctl")
            .args(["-H", "-A", "-j", device_path])
            .output()
            .await;

        if let Ok(output) = output {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(smart) = serde_json::from_str::<SmartctlOutput>(&json_str) {
                let temp = smart.temperature.map(|t| t.current);
                let health = smart.smart_status.map(|s| {
                    if s.passed { "PASSED".to_string() } else { "FAILED".to_string() }
                });
                return (temp, health);
            }
        }

        (None, None)
    }

    /// Convert StorageVolume to VolumeInfo with filesystem stats
    async fn volume_to_info(&self, volume: StorageVolume) -> VolumeInfo {
        let (size, used, available) = self.get_mount_stats(&volume.mount_point).await;
        let usage_percent = if size > 0 {
            ((used as f64 / size as f64) * 100.0) as u8
        } else {
            0
        };

        VolumeInfo {
            id: volume.id,
            pool_id: volume.pool_id,
            name: volume.name,
            fs_type: volume.fs_type,
            mount_point: volume.mount_point,
            size,
            used,
            available,
            usage_percent,
            status: match volume.status.as_str() {
                "mounted" => VolumeStatus::Mounted,
                "unmounted" => VolumeStatus::Unmounted,
                "creating" => VolumeStatus::Creating,
                _ => VolumeStatus::Error,
            },
            mount_options: volume.mount_options,
            created_at: volume.created_at,
        }
    }

    /// Get filesystem stats for a mount point
    async fn get_mount_stats(&self, mount_point: &str) -> (u64, u64, u64) {
        // In dev mode, return fake stats
        if self.dev_mode {
            // Return fake stats: 500GB total, 125GB used, 375GB available
            let total = 500 * 1024 * 1024 * 1024u64;
            let used = 125 * 1024 * 1024 * 1024u64;
            let available = total - used;
            return (total, used, available);
        }

        let output = Command::new("df")
            .args(["-B1", mount_point])
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse df output (skip header line)
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let size = parts[1].parse().unwrap_or(0);
                    let used = parts[2].parse().unwrap_or(0);
                    let available = parts[3].parse().unwrap_or(0);
                    return (size, used, available);
                }
            }
        }

        (0, 0, 0)
    }

    /// Calculate pool usage from volumes
    async fn calculate_pool_usage(&self, pool_id: &str) -> (u64, u64) {
        let volumes = self.list_volumes_for_pool(pool_id).await.unwrap_or_default();
        let used: u64 = volumes.iter().map(|v| v.used).sum();
        let available: u64 = volumes.iter().map(|v| v.available).sum();
        (used, available)
    }

    /// Calculate total size of devices
    async fn calculate_devices_total_size(&self, devices: &[String]) -> u64 {
        let disks = self.list_disks().await.unwrap_or_default();
        let mut total = 0u64;

        for device in devices {
            for disk in &disks {
                if &disk.device_path == device || disk.device_by_id.as_ref() == Some(device) {
                    total += disk.size;
                    break;
                }
            }
        }

        total
    }

    /// Create a basic (single device) pool
    async fn create_basic_pool(&self, device: &str) -> Result<String> {
        // Create a single partition using the whole disk
        let output = Command::new("sgdisk")
            .args(["-n", "1:0:0", "-t", "1:8300", device])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to create partition: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Notify kernel
        let _ = Command::new("partprobe").arg(device).output().await;

        // Wait for partition to appear
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok("{}".to_string()) // Empty metadata for basic pool
    }

    /// Create a btrfs pool
    async fn create_btrfs_pool(&self, devices: &[String], raid_type: &RaidType) -> Result<String> {
        let profile = match raid_type {
            RaidType::BtrfsSingle => "single",
            RaidType::BtrfsRaid0 => "raid0",
            RaidType::BtrfsRaid1 => "raid1",
            RaidType::BtrfsRaid5 => "raid5",
            RaidType::BtrfsRaid6 => "raid6",
            RaidType::BtrfsRaid10 => "raid10",
            _ => "single",
        };

        let mut args = vec!["-f", "-d", profile, "-m", profile];
        for device in devices {
            args.push(device);
        }

        let output = Command::new("mkfs.btrfs")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to create btrfs: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Get btrfs UUID
        let uuid_output = Command::new("btrfs")
            .args(["filesystem", "show", devices.first().unwrap()])
            .output()
            .await?;

        let uuid = String::from_utf8_lossy(&uuid_output.stdout)
            .lines()
            .find(|l| l.contains("uuid:"))
            .and_then(|l| l.split("uuid:").nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Ok(serde_json::json!({ "btrfs_uuid": uuid }).to_string())
    }

    /// Create an mdadm RAID pool
    async fn create_mdadm_pool(&self, devices: &[String], raid_type: &RaidType) -> Result<String> {
        // Check if mdadm is available
        let check = Command::new("which").arg("mdadm").output().await?;
        if !check.status.success() {
            return Err(anyhow!("mdadm is not available. Use btrfs RAID instead."));
        }

        let level = match raid_type {
            RaidType::Jbod => "linear",
            RaidType::Raid0 => "0",
            RaidType::Raid1 => "1",
            RaidType::Raid5 => "5",
            RaidType::Raid6 => "6",
            RaidType::Raid10 => "10",
            _ => return Err(anyhow!("Unsupported RAID type for mdadm")),
        };

        // Find next available md device
        let md_device = self.find_next_md_device().await?;

        let mut args = vec![
            "--create".to_string(),
            md_device.clone(),
            "--level".to_string(),
            level.to_string(),
            "--raid-devices".to_string(),
            devices.len().to_string(),
        ];

        for device in devices {
            args.push(device.clone());
        }

        let output = Command::new("mdadm")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to create RAID: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(serde_json::json!({ "md_device": md_device }).to_string())
    }

    /// Find next available /dev/mdX device
    async fn find_next_md_device(&self) -> Result<String> {
        for i in 0..128 {
            let device = format!("/dev/md{}", i);
            if !Path::new(&device).exists() {
                return Ok(device);
            }
        }
        Err(anyhow!("No available md device"))
    }

    /// Create a btrfs subvolume
    async fn create_btrfs_subvolume(&self, pool: &StoragePoolInfo, name: &str, mount_point: &str) -> Result<()> {
        // First mount the pool root if not mounted
        let pool_mount = format!("{}/{}", self.pools_base_path, pool.name);

        if let Some(device) = pool.devices.first() {
            // Mount pool root
            tokio::fs::create_dir_all(&pool_mount).await?;
            self.mount(device, &pool_mount, Some("btrfs"), None).await?;

            // Create subvolume
            let output = Command::new("btrfs")
                .args(["subvolume", "create", &format!("{}/{}", pool_mount, name)])
                .output()
                .await?;

            if !output.status.success() {
                return Err(anyhow!("Failed to create subvolume: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }

        Ok(())
    }

    /// Create filesystem on a device
    async fn create_filesystem(&self, device: &str, fs_type: &str) -> Result<()> {
        let mkfs_cmd = match fs_type {
            "ext4" => "mkfs.ext4",
            "btrfs" => "mkfs.btrfs",
            "xfs" => "mkfs.xfs",
            "f2fs" => "mkfs.f2fs",
            _ => return Err(anyhow!("Unsupported filesystem type: {}", fs_type)),
        };

        // Get partition device (e.g., /dev/sda1)
        let part_device = format!("{}1", device);

        let output = Command::new(mkfs_cmd)
            .args(["-f", &part_device])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to create filesystem: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    /// Mount a device
    async fn mount(&self, device: &str, mount_point: &str, fs_type: Option<&str>, mount_options: Option<&str>) -> Result<()> {
        tokio::fs::create_dir_all(mount_point).await?;

        let mut args: Vec<&str> = Vec::new();
        if let Some(fs) = fs_type {
            args.push("-t");
            args.push(fs);
        }
        // We need to own the string for mount_options
        let opts_string;
        if let Some(opts) = mount_options {
            if !opts.is_empty() {
                opts_string = opts.to_string();
                args.push("-o");
                args.push(&opts_string);
            }
        }
        args.push(device);
        args.push(mount_point);

        let output = Command::new("mount")
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to mount: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    /// Unmount a path
    async fn unmount(&self, path: &str) -> Result<()> {
        let output = Command::new("umount")
            .arg(path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("not mounted") {
                return Err(anyhow!("Failed to unmount: {}", stderr));
            }
        }

        Ok(())
    }

    // ============ HEALTH MONITORING ============

    /// Get pool health information
    pub async fn get_pool_health(&self, pool_id: &str) -> Result<PoolHealthInfo> {
        let pool: StoragePool = sqlx::query_as(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(pool_id)
        .fetch_one(&self.db)
        .await
        .context("Pool not found")?;

        let devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        let raid_type: RaidType = pool.raid_type.parse().unwrap_or(RaidType::Basic);
        let metadata: serde_json::Value = pool.metadata
            .as_deref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or(serde_json::json!({}));

        if self.dev_mode {
            return Ok(self.get_fake_pool_health(pool_id, &pool.status, &pool.raid_type, &devices));
        }

        // Parse health based on pool type
        let (device_health, rebuild_progress) = match raid_type {
            RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid5 | RaidType::BtrfsRaid6 | RaidType::BtrfsRaid10 => {
                self.parse_btrfs_health(&devices).await
            }
            RaidType::Jbod | RaidType::Raid0 | RaidType::Raid1 | RaidType::Raid5 | RaidType::Raid6 | RaidType::Raid10 => {
                let md_device = metadata.get("md_device")
                    .and_then(|v| v.as_str())
                    .unwrap_or("/dev/md0");
                self.parse_mdadm_health(md_device).await
            }
            _ => (devices.iter().map(|d| DeviceHealthInfo {
                device_path: d.clone(),
                state: "active".to_string(),
                errors: DeviceErrorInfo { read_errors: 0, write_errors: 0, corruption_errors: 0, generation_errors: 0 },
            }).collect(), None),
        };

        // Load last scrub from metadata
        let last_scrub = metadata.get("last_scrub")
            .and_then(|v| serde_json::from_value::<ScrubInfo>(v.clone()).ok());

        let status = match pool.status.as_str() {
            "normal" => PoolStatus::Normal,
            "degraded" => PoolStatus::Degraded,
            "rebuilding" => PoolStatus::Rebuilding,
            "expanding" => PoolStatus::Expanding,
            "creating" => PoolStatus::Creating,
            _ => PoolStatus::Error,
        };

        Ok(PoolHealthInfo {
            pool_id: pool_id.to_string(),
            status,
            raid_type: pool.raid_type,
            devices: device_health,
            rebuild_progress,
            last_scrub,
        })
    }

    /// Fake pool health for dev mode
    fn get_fake_pool_health(&self, pool_id: &str, status: &str, raid_type: &str, devices: &[String]) -> PoolHealthInfo {
        PoolHealthInfo {
            pool_id: pool_id.to_string(),
            status: match status {
                "normal" => PoolStatus::Normal,
                "degraded" => PoolStatus::Degraded,
                "rebuilding" => PoolStatus::Rebuilding,
                _ => PoolStatus::Normal,
            },
            raid_type: raid_type.to_string(),
            devices: devices.iter().map(|d| DeviceHealthInfo {
                device_path: d.clone(),
                state: "active".to_string(),
                errors: DeviceErrorInfo { read_errors: 0, write_errors: 0, corruption_errors: 0, generation_errors: 0 },
            }).collect(),
            rebuild_progress: None,
            last_scrub: Some(ScrubInfo {
                date: chrono::Utc::now().checked_sub_signed(chrono::Duration::days(3))
                    .unwrap_or(chrono::Utc::now()).to_rfc3339(),
                duration_secs: Some(1234),
                result: "ok".to_string(),
                errors_found: 0,
            }),
        }
    }

    /// Parse btrfs device stats for health info
    async fn parse_btrfs_health(&self, devices: &[String]) -> (Vec<DeviceHealthInfo>, Option<f32>) {
        let mut health = Vec::new();

        for device in devices {
            let output = Command::new("btrfs")
                .args(["device", "stats", device])
                .output()
                .await;

            let mut errors = DeviceErrorInfo {
                read_errors: 0, write_errors: 0,
                corruption_errors: 0, generation_errors: 0,
            };

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let val: u64 = parts.last().and_then(|v| v.parse().ok()).unwrap_or(0);
                        if line.contains("read_io_errs") { errors.read_errors = val; }
                        else if line.contains("write_io_errs") { errors.write_errors = val; }
                        else if line.contains("corruption_errs") { errors.corruption_errors = val; }
                        else if line.contains("generation_errs") { errors.generation_errors = val; }
                    }
                }
            }

            let has_errors = errors.read_errors > 0 || errors.write_errors > 0
                || errors.corruption_errors > 0 || errors.generation_errors > 0;

            health.push(DeviceHealthInfo {
                device_path: device.clone(),
                state: if has_errors { "error".to_string() } else { "active".to_string() },
                errors,
            });
        }

        (health, None)
    }

    /// Parse /proc/mdstat for mdadm RAID health
    async fn parse_mdadm_health(&self, md_device: &str) -> (Vec<DeviceHealthInfo>, Option<f32>) {
        let md_name = md_device.trim_start_matches("/dev/");
        let mdstat = tokio::fs::read_to_string("/proc/mdstat").await.unwrap_or_default();

        let mut devices = Vec::new();
        let mut rebuild_progress = None;
        let mut in_section = false;

        for line in mdstat.lines() {
            if line.starts_with(md_name) {
                in_section = true;
                // Parse device list: md0 : active raid1 sda1[0] sdb1[1]
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts[4..] {
                    let dev_name = part.split('[').next().unwrap_or(part);
                    let state = if part.contains("(F)") { "faulty" }
                        else if part.contains("(S)") { "spare" }
                        else { "active" };
                    devices.push(DeviceHealthInfo {
                        device_path: format!("/dev/{}", dev_name),
                        state: state.to_string(),
                        errors: DeviceErrorInfo { read_errors: 0, write_errors: 0, corruption_errors: 0, generation_errors: 0 },
                    });
                }
            } else if in_section && line.contains("recovery") {
                // Parse: [==>..................] recovery = 12.6% ...
                if let Some(pct_str) = line.split('=').nth(1) {
                    if let Some(pct) = pct_str.trim().split('%').next() {
                        rebuild_progress = pct.trim().parse().ok();
                    }
                }
            } else if in_section && line.trim().is_empty() {
                in_section = false;
            }
        }

        (devices, rebuild_progress)
    }

    // ============ HEALTH MONITOR BACKGROUND TASK ============

    /// Start background health monitor (runs every 60s, broadcasts alerts on status change)
    pub fn start_health_monitor(
        db: SqlitePool,
        storage_tx: broadcast::Sender<StorageAlertEvent>,
        dev_mode: bool,
    ) {
        tokio::spawn(async move {
            let mut previous_statuses: HashMap<String, String> = HashMap::new();
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let service = StorageService::new(db.clone());
                let pools = match service.list_pools().await {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("Health monitor: failed to list pools: {}", e);
                        continue;
                    }
                };

                for pool in &pools {
                    let health = match service.get_pool_health(&pool.id).await {
                        Ok(h) => h,
                        Err(_) => continue,
                    };

                    let new_status = health.status.to_string();
                    let prev_status = previous_statuses.get(&pool.id).cloned()
                        .unwrap_or_else(|| "normal".to_string());

                    if new_status != prev_status {
                        let alert_type = match new_status.as_str() {
                            "degraded" => "degraded",
                            "error" => "error",
                            "normal" if prev_status == "rebuilding" => "rebuilt",
                            _ => "status_change",
                        };

                        let alert = StorageAlertEvent {
                            pool_id: pool.id.clone(),
                            pool_name: pool.name.clone(),
                            alert_type: alert_type.to_string(),
                            previous_status: prev_status.clone(),
                            new_status: new_status.clone(),
                            message: format!("Pool '{}' status changed from {} to {}",
                                pool.name, prev_status, new_status),
                        };

                        tracing::warn!("Storage alert: {}", alert.message);
                        let _ = storage_tx.send(alert);

                        // Update DB status if changed
                        let now = chrono::Utc::now().to_rfc3339();
                        let _ = sqlx::query("UPDATE storage_pools SET status = ?, updated_at = ? WHERE id = ?")
                            .bind(&new_status)
                            .bind(&now)
                            .bind(&pool.id)
                            .execute(&db)
                            .await;
                    }

                    previous_statuses.insert(pool.id.clone(), new_status);
                }
            }
        });
    }

    // ============ SCRUB OPERATIONS ============

    /// Start a scrub operation (returns task_id immediately)
    pub async fn scrub_pool_start(&self, pool_id: &str) -> Result<ScrubStatus> {
        let pool: StoragePool = sqlx::query_as(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(pool_id)
        .fetch_one(&self.db)
        .await
        .context("Pool not found")?;

        let task_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        Ok(ScrubStatus {
            task_id,
            pool_id: pool_id.to_string(),
            status: "running".to_string(),
            progress: 0.0,
            errors_found: 0,
            started_at: now,
        })
    }

    /// Execute scrub in background (btrfs scrub or mdadm check)
    pub async fn scrub_pool_execute(
        db: SqlitePool,
        pool_id: String,
        task_id: String,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        dev_mode: bool,
    ) {
        let pool: StoragePool = match sqlx::query_as::<_, StoragePool>(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(&pool_id)
        .fetch_one(&db)
        .await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Scrub: pool not found: {}", e);
                return;
            }
        };

        let raid_type: RaidType = pool.raid_type.parse().unwrap_or(RaidType::Basic);
        let metadata: serde_json::Value = pool.metadata
            .as_deref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or(serde_json::json!({}));
        let devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        let started_at = chrono::Utc::now();

        // Send initial progress
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: pool_id.clone(),
            status: "running".to_string(),
            progress: 0,
            total_steps: 1,
            progress_percent: 0,
            current_step: Some("Starting scrub...".to_string()),
            error_message: None,
        });

        if dev_mode {
            // Simulate scrub progress
            for pct in (0..=100).step_by(10) {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: pool_id.clone(),
                    status: "running".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: pct,
                    current_step: Some(format!("Scrubbing... {}%", pct)),
                    error_message: None,
                });
            }
        } else {
            // Real scrub
            let result = match raid_type {
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid5 | RaidType::BtrfsRaid6 | RaidType::BtrfsRaid10 => {
                    Self::scrub_btrfs(&devices, &task_id, &pool_id, &task_tx).await
                }
                RaidType::Jbod | RaidType::Raid0 | RaidType::Raid1 | RaidType::Raid5 | RaidType::Raid6 | RaidType::Raid10 => {
                    let md_device = metadata.get("md_device")
                        .and_then(|v| v.as_str())
                        .unwrap_or("/dev/md0");
                    Self::scrub_mdadm(md_device, &task_id, &pool_id, &task_tx).await
                }
                _ => Ok(0u64),
            };

            if let Err(e) = result {
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: pool_id.clone(),
                    status: "failed".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: 100,
                    current_step: None,
                    error_message: Some(e.to_string()),
                });
                return;
            }
        }

        // Save last scrub info to pool metadata
        let duration = (chrono::Utc::now() - started_at).num_seconds().max(0) as u64;
        let scrub_info = ScrubInfo {
            date: started_at.to_rfc3339(),
            duration_secs: Some(duration),
            result: "ok".to_string(),
            errors_found: 0,
        };

        let mut meta = metadata.clone();
        meta["last_scrub"] = serde_json::to_value(&scrub_info).unwrap_or_default();

        let now = chrono::Utc::now().to_rfc3339();
        let _ = sqlx::query("UPDATE storage_pools SET metadata = ?, updated_at = ? WHERE id = ?")
            .bind(meta.to_string())
            .bind(&now)
            .bind(&pool_id)
            .execute(&db)
            .await;

        // Send completion
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: pool_id.clone(),
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 100,
            current_step: Some("Scrub completed".to_string()),
            error_message: None,
        });
    }

    /// Run btrfs scrub and monitor progress
    async fn scrub_btrfs(
        devices: &[String],
        task_id: &str,
        pool_id: &str,
        task_tx: &broadcast::Sender<crate::api::ws::TaskProgressEvent>,
    ) -> Result<u64> {
        let mount_point = devices.first().ok_or_else(|| anyhow!("No devices in pool"))?;

        // Start scrub
        let output = Command::new("btrfs")
            .args(["scrub", "start", mount_point])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to start btrfs scrub: {}", String::from_utf8_lossy(&output.stderr)));
        }

        // Poll scrub status until complete
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let status = Command::new("btrfs")
                .args(["scrub", "status", mount_point])
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&status.stdout);

            // Check if scrub is finished
            if stdout.contains("finished") || stdout.contains("aborted") {
                break;
            }

            // Try to extract progress (btrfs scrub status shows bytes scrubbed)
            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                task_id: task_id.to_string(),
                package_id: pool_id.to_string(),
                status: "running".to_string(),
                progress: 1,
                total_steps: 1,
                progress_percent: 50, // btrfs doesn't give precise %
                current_step: Some("Scrubbing filesystem...".to_string()),
                error_message: None,
            });
        }

        Ok(0)
    }

    /// Run mdadm check and monitor progress
    async fn scrub_mdadm(
        md_device: &str,
        task_id: &str,
        pool_id: &str,
        task_tx: &broadcast::Sender<crate::api::ws::TaskProgressEvent>,
    ) -> Result<u64> {
        let md_name = md_device.trim_start_matches("/dev/");

        // Trigger check via sysfs
        let check_path = format!("/sys/block/{}/md/sync_action", md_name);
        tokio::fs::write(&check_path, "check").await
            .context("Failed to start mdadm check")?;

        // Poll progress
        let progress_path = format!("/sys/block/{}/md/sync_completed", md_name);
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let action = tokio::fs::read_to_string(&check_path).await.unwrap_or_default();
            if action.trim() == "idle" {
                break;
            }

            // Parse progress: "123456 / 789012"
            let completed = tokio::fs::read_to_string(&progress_path).await.unwrap_or_default();
            let parts: Vec<&str> = completed.split('/').collect();
            let pct = if parts.len() == 2 {
                let current: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let total: f64 = parts[1].trim().parse().unwrap_or(1.0);
                ((current / total) * 100.0) as i32
            } else {
                50
            };

            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                task_id: task_id.to_string(),
                package_id: pool_id.to_string(),
                status: "running".to_string(),
                progress: 1,
                total_steps: 1,
                progress_percent: pct,
                current_step: Some(format!("Checking RAID array... {}%", pct)),
                error_message: None,
            });
        }

        // Check for mismatches
        let mismatch_path = format!("/sys/block/{}/md/mismatch_cnt", md_name);
        let mismatches: u64 = tokio::fs::read_to_string(&mismatch_path).await
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);

        Ok(mismatches)
    }

    // ============ SMART SCHEDULED TESTS ============

    /// List all SMART test schedules
    pub async fn list_smart_schedules(&self) -> Result<Vec<SmartTestScheduleInfo>> {
        let schedules: Vec<SmartTestSchedule> = sqlx::query_as(
            "SELECT * FROM smart_test_schedules ORDER BY device_name, test_type"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(schedules.into_iter().map(|s| {
            let last_result = s.last_result.as_deref()
                .and_then(|r| serde_json::from_str::<SmartTestResult>(r).ok());
            SmartTestScheduleInfo {
                id: s.id,
                device_path: s.device_path,
                device_name: s.device_name,
                test_type: s.test_type,
                interval_hours: s.interval_hours,
                last_run: s.last_run,
                next_run: s.next_run,
                last_result,
                enabled: s.enabled,
            }
        }).collect())
    }

    /// Create a new SMART test schedule
    pub async fn create_smart_schedule(&self, request: CreateSmartScheduleRequest) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let interval_hours = request.interval_hours.unwrap_or(168); // default weekly
        let next_run = now + chrono::Duration::hours(interval_hours);

        // Extract device_name from device_path
        let device_name = request.device_path.trim_start_matches("/dev/").to_string();

        sqlx::query(
            "INSERT INTO smart_test_schedules (id, device_path, device_name, test_type, interval_hours, next_run, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)"
        )
        .bind(&id)
        .bind(&request.device_path)
        .bind(&device_name)
        .bind(&request.test_type)
        .bind(interval_hours)
        .bind(next_run.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.db)
        .await?;

        Ok(id)
    }

    /// Delete a SMART test schedule
    pub async fn delete_smart_schedule(&self, schedule_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM smart_test_schedules WHERE id = ?")
            .bind(schedule_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Schedule not found"));
        }
        Ok(())
    }

    /// Toggle a SMART test schedule enabled/disabled
    pub async fn toggle_smart_schedule(&self, schedule_id: &str, enabled: bool) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query("UPDATE smart_test_schedules SET enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(&now)
            .bind(schedule_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Schedule not found"));
        }
        Ok(())
    }

    /// Start a SMART test on-demand (returns task_id immediately)
    pub fn smart_test_start(&self, device_name: &str, test_type: &str) -> SmartTestStatus {
        let task_id = Uuid::new_v4().to_string();
        SmartTestStatus {
            task_id,
            device_name: device_name.to_string(),
            test_type: test_type.to_string(),
            status: "running".to_string(),
            progress: 0,
        }
    }

    /// Execute a SMART test in background
    pub async fn smart_test_execute(
        device_name: String,
        test_type: String,
        task_id: String,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        dev_mode: bool,
    ) {
        let device_path = format!("/dev/{}", device_name);

        // Send initial progress
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: device_name.clone(),
            status: "running".to_string(),
            progress: 0,
            total_steps: 1,
            progress_percent: 0,
            current_step: Some(format!("Starting {} SMART test...", test_type)),
            error_message: None,
        });

        if dev_mode {
            // Simulate test progress
            let duration_ms = match test_type.as_str() {
                "short" => 5000u64,
                "long" => 15000,
                _ => 5000,
            };
            let steps = 20u64;
            let step_ms = duration_ms / steps;

            for i in 1..=steps {
                tokio::time::sleep(tokio::time::Duration::from_millis(step_ms)).await;
                let pct = ((i as f64 / steps as f64) * 100.0) as i32;
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: device_name.clone(),
                    status: "running".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: pct,
                    current_step: Some(format!("Running {} test... {}%", test_type, pct)),
                    error_message: None,
                });
            }
        } else {
            // Start real SMART test
            let test_arg = match test_type.as_str() {
                "short" => "short",
                "long" => "long",
                "conveyance" => "conveyance",
                "offline" => "offline",
                _ => "short",
            };

            let output = Command::new("smartctl")
                .args(["-t", test_arg, &device_path])
                .output()
                .await;

            if let Err(e) = output {
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: device_name.clone(),
                    status: "failed".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: 100,
                    current_step: None,
                    error_message: Some(format!("Failed to start SMART test: {}", e)),
                });
                return;
            }

            // Poll smartctl -l selftest to track progress
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                let status_output = Command::new("smartctl")
                    .args(["-l", "selftest", "-j", &device_path])
                    .output()
                    .await;

                match status_output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Check if test is still running (remaining > 0%)
                        if stdout.contains("Self-test routine in progress") || stdout.contains("in progress") {
                            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                                task_id: task_id.clone(),
                                package_id: device_name.clone(),
                                status: "running".to_string(),
                                progress: 1,
                                total_steps: 1,
                                progress_percent: 50,
                                current_step: Some(format!("Running {} test...", test_type)),
                                error_message: None,
                            });
                        } else {
                            // Test completed
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        // Send completion
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: device_name.clone(),
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 100,
            current_step: Some(format!("{} test completed", test_type)),
            error_message: None,
        });
    }

    /// Get SMART test history for a device
    pub async fn get_smart_test_history(&self, device_name: &str) -> Result<Vec<SmartTestHistoryEntry>> {
        if self.dev_mode {
            return Ok(vec![
                SmartTestHistoryEntry {
                    num: 1,
                    test_type: "Short offline".to_string(),
                    status: "Completed without error".to_string(),
                    remaining_percent: 0,
                    lifetime_hours: 1234,
                    lba_of_first_error: None,
                },
                SmartTestHistoryEntry {
                    num: 2,
                    test_type: "Extended offline".to_string(),
                    status: "Completed without error".to_string(),
                    remaining_percent: 0,
                    lifetime_hours: 1100,
                    lba_of_first_error: None,
                },
                SmartTestHistoryEntry {
                    num: 3,
                    test_type: "Short offline".to_string(),
                    status: "Completed without error".to_string(),
                    remaining_percent: 0,
                    lifetime_hours: 900,
                    lba_of_first_error: None,
                },
            ]);
        }

        let device_path = format!("/dev/{}", device_name);
        let output = Command::new("smartctl")
            .args(["-l", "selftest", "-j", &device_path])
            .output()
            .await
            .context("Failed to run smartctl")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

        let mut entries = Vec::new();
        if let Some(tests) = json.get("ata_smart_self_test_log").and_then(|l| l.get("standard")).and_then(|s| s.get("table")).and_then(|t| t.as_array()) {
            for (i, test) in tests.iter().enumerate() {
                entries.push(SmartTestHistoryEntry {
                    num: (i + 1) as u32,
                    test_type: test.get("type").and_then(|v| v.get("string")).and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                    status: test.get("status").and_then(|v| v.get("string")).and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                    remaining_percent: test.get("status").and_then(|v| v.get("value")).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    lifetime_hours: test.get("lifetime_hours").and_then(|v| v.as_u64()).unwrap_or(0),
                    lba_of_first_error: test.get("lba").and_then(|v| v.as_u64()),
                });
            }
        }

        Ok(entries)
    }

    /// Start background SMART scheduler (runs every 60s, checks for due schedules)
    pub fn start_smart_scheduler(
        db: SqlitePool,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        dev_mode: bool,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Find due schedules
                let schedules: Vec<SmartTestSchedule> = match sqlx::query_as(
                    "SELECT * FROM smart_test_schedules WHERE enabled = 1 AND next_run <= datetime('now')"
                )
                .fetch_all(&db)
                .await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("SMART scheduler: failed to query schedules: {}", e);
                        continue;
                    }
                };

                for schedule in schedules {
                    tracing::info!("SMART scheduler: running {} test on {}", schedule.test_type, schedule.device_name);

                    let task_id = Uuid::new_v4().to_string();
                    let device_name = schedule.device_name.clone();
                    let test_type = schedule.test_type.clone();
                    let tx = task_tx.clone();
                    let db_clone = db.clone();
                    let schedule_id = schedule.id.clone();
                    let interval_hours = schedule.interval_hours;

                    // Spawn test execution
                    tokio::spawn(async move {
                        Self::smart_test_execute(device_name, test_type.clone(), task_id, tx, dev_mode).await;

                        // Update schedule after completion
                        let now = chrono::Utc::now();
                        let next_run = now + chrono::Duration::hours(interval_hours);
                        let result = SmartTestResult {
                            status: "completed".to_string(),
                            errors: 0,
                            duration_secs: 0,
                            completed_at: now.to_rfc3339(),
                        };
                        let result_json = serde_json::to_string(&result).unwrap_or_default();

                        let _ = sqlx::query(
                            "UPDATE smart_test_schedules SET last_run = ?, next_run = ?, last_result = ?, updated_at = ? WHERE id = ?"
                        )
                        .bind(now.to_rfc3339())
                        .bind(next_run.to_rfc3339())
                        .bind(&result_json)
                        .bind(now.to_rfc3339())
                        .bind(&schedule_id)
                        .execute(&db_clone)
                        .await;
                    });
                }
            }
        });
    }

    // ============ DISK POWER MANAGEMENT ============

    /// Get power settings for a disk
    pub async fn get_disk_power_settings(&self, device_name: &str) -> Result<DiskPowerSettings> {
        let device_path = format!("/dev/{}", device_name);

        let settings: Option<DiskPowerSettings> = sqlx::query_as(
            "SELECT * FROM disk_power_settings WHERE device_path = ?"
        )
        .bind(&device_path)
        .fetch_optional(&self.db)
        .await?;

        Ok(settings.unwrap_or(DiskPowerSettings {
            device_path,
            device_name: device_name.to_string(),
            apm_level: None,
            spindown_minutes: None,
            write_cache: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Set power settings for a disk
    pub async fn set_disk_power_settings(&self, device_name: &str, request: UpdateDiskPowerRequest) -> Result<()> {
        let device_path = format!("/dev/{}", device_name);

        // Reject NVMe and SD cards
        if device_name.starts_with("nvme") || device_name.starts_with("mmcblk") {
            return Err(anyhow!("Power management not supported for NVMe/SD devices"));
        }

        // Validate values
        if let Some(apm) = request.apm_level {
            if !(1..=254).contains(&apm) {
                return Err(anyhow!("APM level must be between 1 and 254"));
            }
        }
        if let Some(spindown) = request.spindown_minutes {
            if spindown != 0 && !(5..=240).contains(&spindown) {
                return Err(anyhow!("Spindown minutes must be 0 (disabled) or between 5 and 240"));
            }
        }

        // Apply settings
        if !self.dev_mode {
            if let Some(apm) = request.apm_level {
                let _ = Command::new("hdparm")
                    .args(["-B", &apm.to_string(), &device_path])
                    .output()
                    .await;
            }
            if let Some(spindown) = request.spindown_minutes {
                // Convert minutes to hdparm value (1-240 = 5s increments, so minutes*60/5 = minutes*12)
                let hdparm_val = if spindown == 0 { 0 } else { (spindown * 12).min(240) };
                let _ = Command::new("hdparm")
                    .args(["-S", &hdparm_val.to_string(), &device_path])
                    .output()
                    .await;
            }
            if let Some(write_cache) = request.write_cache {
                let val = if write_cache { "1" } else { "0" };
                let _ = Command::new("hdparm")
                    .args(["-W", val, &device_path])
                    .output()
                    .await;
            }
        } else {
            tracing::info!("[DEV MODE] Would set power settings for {}: apm={:?}, spindown={:?}, write_cache={:?}",
                device_name, request.apm_level, request.spindown_minutes, request.write_cache);
        }

        // Save to DB (upsert)
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO disk_power_settings (device_path, device_name, apm_level, spindown_minutes, write_cache, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(device_path) DO UPDATE SET
                apm_level = COALESCE(excluded.apm_level, apm_level),
                spindown_minutes = COALESCE(excluded.spindown_minutes, spindown_minutes),
                write_cache = COALESCE(excluded.write_cache, write_cache),
                updated_at = excluded.updated_at"
        )
        .bind(&device_path)
        .bind(device_name)
        .bind(request.apm_level)
        .bind(request.spindown_minutes)
        .bind(request.write_cache)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Apply all saved power settings on boot
    pub async fn apply_power_settings_on_boot(db: &SqlitePool, dev_mode: bool) {
        let settings: Vec<DiskPowerSettings> = match sqlx::query_as(
            "SELECT * FROM disk_power_settings"
        )
        .fetch_all(db)
        .await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to load power settings on boot: {}", e);
                return;
            }
        };

        for setting in settings {
            if dev_mode {
                tracing::info!("[DEV MODE] Would apply power settings for {} on boot: apm={:?}, spindown={:?}, write_cache={:?}",
                    setting.device_name, setting.apm_level, setting.spindown_minutes, setting.write_cache);
                continue;
            }

            if let Some(apm) = setting.apm_level {
                let _ = Command::new("hdparm")
                    .args(["-B", &apm.to_string(), &setting.device_path])
                    .output()
                    .await;
            }
            if let Some(spindown) = setting.spindown_minutes {
                let hdparm_val = if spindown == 0 { 0 } else { (spindown * 12).min(240) };
                let _ = Command::new("hdparm")
                    .args(["-S", &hdparm_val.to_string(), &setting.device_path])
                    .output()
                    .await;
            }
            if let Some(write_cache) = setting.write_cache {
                let val = if write_cache { "1" } else { "0" };
                let _ = Command::new("hdparm")
                    .args(["-W", val, &setting.device_path])
                    .output()
                    .await;
            }

            tracing::info!("Applied power settings for {}", setting.device_name);
        }
    }

    // ============ BTRFS SNAPSHOTS ============

    /// List snapshots for a volume
    pub async fn list_snapshots(&self, volume_id: &str) -> Result<Vec<BtrfsSnapshotInfo>> {
        let snapshots: Vec<BtrfsSnapshot> = sqlx::query_as(
            "SELECT * FROM btrfs_snapshots WHERE volume_id = ? ORDER BY created_at DESC"
        )
        .bind(volume_id)
        .fetch_all(&self.db)
        .await?;

        Ok(snapshots.into_iter().map(|s| BtrfsSnapshotInfo {
            id: s.id,
            volume_id: s.volume_id,
            name: s.name,
            path: s.path,
            snapshot_type: s.snapshot_type,
            size_bytes: s.size_bytes.map(|b| b as u64),
            created_at: s.created_at,
        }).collect())
    }

    /// Create a btrfs snapshot
    pub async fn create_snapshot(&self, volume_id: &str, request: CreateSnapshotRequest) -> Result<String> {
        // Verify volume exists and is btrfs
        let volume: StorageVolume = sqlx::query_as(
            "SELECT * FROM storage_volumes WHERE id = ?"
        )
        .bind(volume_id)
        .fetch_one(&self.db)
        .await
        .context("Volume not found")?;

        if volume.fs_type != "btrfs" {
            return Err(anyhow!("Snapshots are only supported for btrfs volumes"));
        }

        if volume.status != "mounted" {
            return Err(anyhow!("Volume must be mounted to create snapshots"));
        }

        let snapshot_id = Uuid::new_v4().to_string();
        let snapshot_dir = format!("{}/.snapshots", volume.mount_point);
        let snapshot_path = format!("{}/{}", snapshot_dir, request.name);

        if !self.dev_mode {
            // Create snapshots directory
            tokio::fs::create_dir_all(&snapshot_dir).await
                .context("Failed to create snapshots directory")?;

            // Create read-only snapshot
            let output = Command::new("btrfs")
                .args(["subvolume", "snapshot", "-r", &volume.mount_point, &snapshot_path])
                .output()
                .await
                .context("Failed to run btrfs snapshot")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to create snapshot: {}", String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            tracing::info!("[DEV MODE] Would create btrfs snapshot '{}' at {}", request.name, snapshot_path);
        }

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO btrfs_snapshots (id, volume_id, name, path, snapshot_type, created_at)
             VALUES (?, ?, ?, ?, 'manual', ?)"
        )
        .bind(&snapshot_id)
        .bind(volume_id)
        .bind(&request.name)
        .bind(&snapshot_path)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(snapshot_id)
    }

    /// Delete a btrfs snapshot
    pub async fn delete_snapshot(&self, volume_id: &str, snapshot_id: &str) -> Result<()> {
        let snapshot: BtrfsSnapshot = sqlx::query_as(
            "SELECT * FROM btrfs_snapshots WHERE id = ? AND volume_id = ?"
        )
        .bind(snapshot_id)
        .bind(volume_id)
        .fetch_one(&self.db)
        .await
        .context("Snapshot not found")?;

        if !self.dev_mode {
            let output = Command::new("btrfs")
                .args(["subvolume", "delete", &snapshot.path])
                .output()
                .await
                .context("Failed to run btrfs subvolume delete")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to delete snapshot: {}", String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            tracing::info!("[DEV MODE] Would delete btrfs snapshot '{}' at {}", snapshot.name, snapshot.path);
        }

        sqlx::query("DELETE FROM btrfs_snapshots WHERE id = ?")
            .bind(snapshot_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ============ RAID GROW/EXPAND ============

    /// Start growing a pool by adding disks (returns task_id immediately)
    pub async fn grow_pool_start(&self, pool_id: &str, request: &GrowPoolRequest) -> Result<GrowPoolStatus> {
        let pool: StoragePool = sqlx::query_as(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(pool_id)
        .fetch_one(&self.db)
        .await
        .context("Pool not found")?;

        let raid_type: RaidType = pool.raid_type.parse().unwrap_or(RaidType::Basic);

        // Validate
        if pool.status != "normal" {
            return Err(anyhow!("Pool must be in normal status to add disks (current: {})", pool.status));
        }
        if raid_type == RaidType::Basic {
            return Err(anyhow!("Cannot grow a basic (single disk) pool"));
        }
        if raid_type == RaidType::Raid10 && request.devices.len() % 2 != 0 {
            return Err(anyhow!("RAID 10 requires an even number of new disks"));
        }

        // Check new devices are not protected or already in a pool
        for device in &request.devices {
            if self.is_protected_device(device) {
                return Err(anyhow!("Cannot use system device: {}", device));
            }
        }
        let existing_devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        for device in &request.devices {
            if existing_devices.contains(device) {
                return Err(anyhow!("Device {} is already in this pool", device));
            }
        }

        // Set status to expanding
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE storage_pools SET status = 'expanding', updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(pool_id)
            .execute(&self.db)
            .await?;

        let task_id = Uuid::new_v4().to_string();
        Ok(GrowPoolStatus {
            task_id,
            pool_id: pool_id.to_string(),
            status: "running".to_string(),
            progress: 0.0,
        })
    }

    /// Execute pool grow in background
    pub async fn grow_pool_execute(
        db: SqlitePool,
        pool_id: String,
        new_devices: Vec<String>,
        wipe_devices: bool,
        task_id: String,
        task_tx: broadcast::Sender<crate::api::ws::TaskProgressEvent>,
        dev_mode: bool,
    ) {
        let pool: StoragePool = match sqlx::query_as::<_, StoragePool>(
            "SELECT * FROM storage_pools WHERE id = ?"
        )
        .bind(&pool_id)
        .fetch_one(&db)
        .await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Grow pool: pool not found: {}", e);
                return;
            }
        };

        let raid_type: RaidType = pool.raid_type.parse().unwrap_or(RaidType::Basic);
        let metadata: serde_json::Value = pool.metadata
            .as_deref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or(serde_json::json!({}));
        let mut existing_devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();

        // Send initial progress
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: pool_id.clone(),
            status: "running".to_string(),
            progress: 0,
            total_steps: 1,
            progress_percent: 0,
            current_step: Some("Preparing to add disks...".to_string()),
            error_message: None,
        });

        if dev_mode {
            // Simulate grow progress
            for pct in (0..=100).step_by(5) {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: pool_id.clone(),
                    status: "running".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: pct,
                    current_step: Some(format!("Expanding pool... {}%", pct)),
                    error_message: None,
                });
            }
        } else {
            // Wipe new devices if requested
            if wipe_devices {
                for device in &new_devices {
                    let _ = Command::new("sgdisk").args(["--zap-all", device]).output().await;
                }
            }

            let result = match raid_type {
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid5 | RaidType::BtrfsRaid6 | RaidType::BtrfsRaid10 => {
                    Self::grow_btrfs_pool(&pool, &new_devices, &task_id, &pool_id, &task_tx).await
                }
                RaidType::Jbod | RaidType::Raid0 | RaidType::Raid1 | RaidType::Raid5 | RaidType::Raid6 | RaidType::Raid10 => {
                    Self::grow_mdadm_pool(&metadata, &raid_type, &new_devices, &existing_devices, &task_id, &pool_id, &task_tx).await
                }
                _ => Err(anyhow!("Cannot grow this pool type")),
            };

            if let Err(e) = result {
                // Revert status to normal on error
                let now = chrono::Utc::now().to_rfc3339();
                let _ = sqlx::query("UPDATE storage_pools SET status = 'normal', updated_at = ? WHERE id = ?")
                    .bind(&now)
                    .bind(&pool_id)
                    .execute(&db)
                    .await;

                let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                    task_id: task_id.clone(),
                    package_id: pool_id.clone(),
                    status: "failed".to_string(),
                    progress: 1,
                    total_steps: 1,
                    progress_percent: 100,
                    current_step: None,
                    error_message: Some(e.to_string()),
                });
                return;
            }
        }

        // Update DB: append new devices, recalculate size, set status normal
        existing_devices.extend(new_devices.clone());
        let devices_json = serde_json::to_string(&existing_devices).unwrap_or_default();

        // Recalculate total size
        let mut total_size: i64 = pool.total_size.unwrap_or(0);
        if dev_mode {
            // In dev mode, just add fake size for each new device
            for _ in &new_devices {
                total_size += 1000 * 1024 * 1024 * 1024; // 1TB per device
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let _ = sqlx::query("UPDATE storage_pools SET devices = ?, total_size = ?, status = 'normal', updated_at = ? WHERE id = ?")
            .bind(&devices_json)
            .bind(total_size)
            .bind(&now)
            .bind(&pool_id)
            .execute(&db)
            .await;

        // Send completion
        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.clone(),
            package_id: pool_id.clone(),
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 100,
            current_step: Some("Pool expansion completed".to_string()),
            error_message: None,
        });
    }

    /// Grow a btrfs pool by adding devices and rebalancing
    async fn grow_btrfs_pool(
        pool: &StoragePool,
        new_devices: &[String],
        task_id: &str,
        pool_id: &str,
        task_tx: &broadcast::Sender<crate::api::ws::TaskProgressEvent>,
    ) -> Result<()> {
        let existing_devices: Vec<String> = serde_json::from_str(&pool.devices).unwrap_or_default();
        let mount_point = format!("/storage/pools/{}", pool.name);

        // Add each new device
        for device in new_devices {
            let output = Command::new("btrfs")
                .args(["device", "add", device, &mount_point])
                .output()
                .await?;

            if !output.status.success() {
                return Err(anyhow!("Failed to add device {}: {}", device, String::from_utf8_lossy(&output.stderr)));
            }
        }

        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.to_string(),
            package_id: pool_id.to_string(),
            status: "running".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 30,
            current_step: Some("Devices added, starting balance...".to_string()),
            error_message: None,
        });

        // Start balance
        let output = Command::new("btrfs")
            .args(["balance", "start", &mount_point])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No space") {
                return Err(anyhow!("Failed to start balance: {}", stderr));
            }
        }

        // Poll balance status
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let status = Command::new("btrfs")
                .args(["balance", "status", &mount_point])
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&status.stdout);
            if stdout.contains("No balance found") || stdout.contains("completed") {
                break;
            }

            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                task_id: task_id.to_string(),
                package_id: pool_id.to_string(),
                status: "running".to_string(),
                progress: 1,
                total_steps: 1,
                progress_percent: 60,
                current_step: Some("Rebalancing data...".to_string()),
                error_message: None,
            });
        }

        Ok(())
    }

    /// Grow an mdadm pool by adding devices and growing the array
    async fn grow_mdadm_pool(
        metadata: &serde_json::Value,
        raid_type: &RaidType,
        new_devices: &[String],
        existing_devices: &[String],
        task_id: &str,
        pool_id: &str,
        task_tx: &broadcast::Sender<crate::api::ws::TaskProgressEvent>,
    ) -> Result<()> {
        let md_device = metadata.get("md_device")
            .and_then(|v| v.as_str())
            .unwrap_or("/dev/md0");

        // Add each new device to the array
        for device in new_devices {
            let output = Command::new("mdadm")
                .args(["--add", md_device, device])
                .output()
                .await?;

            if !output.status.success() {
                return Err(anyhow!("Failed to add device {}: {}", device, String::from_utf8_lossy(&output.stderr)));
            }
        }

        let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
            task_id: task_id.to_string(),
            package_id: pool_id.to_string(),
            status: "running".to_string(),
            progress: 1,
            total_steps: 1,
            progress_percent: 20,
            current_step: Some("Devices added, growing array...".to_string()),
            error_message: None,
        });

        // For RAID 5/6/10, grow the array to use the new devices
        match raid_type {
            RaidType::Raid5 | RaidType::Raid6 | RaidType::Raid10 => {
                let new_total = existing_devices.len() + new_devices.len();
                let output = Command::new("mdadm")
                    .args(["--grow", md_device, "--raid-devices", &new_total.to_string()])
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(anyhow!("Failed to grow array: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            _ => {} // JBOD, RAID0, RAID1 - devices are used immediately after --add
        }

        // Monitor reshape progress via sysfs
        let md_name = md_device.trim_start_matches("/dev/");
        let progress_path = format!("/sys/block/{}/md/sync_completed", md_name);
        let action_path = format!("/sys/block/{}/md/sync_action", md_name);

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let action = tokio::fs::read_to_string(&action_path).await.unwrap_or_default();
            if action.trim() == "idle" {
                break;
            }

            let completed = tokio::fs::read_to_string(&progress_path).await.unwrap_or_default();
            let parts: Vec<&str> = completed.split('/').collect();
            let pct = if parts.len() == 2 {
                let current: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let total: f64 = parts[1].trim().parse().unwrap_or(1.0);
                (20.0 + (current / total) * 80.0) as i32
            } else {
                50
            };

            let _ = task_tx.send(crate::api::ws::TaskProgressEvent {
                task_id: task_id.to_string(),
                package_id: pool_id.to_string(),
                status: "running".to_string(),
                progress: 1,
                total_steps: 1,
                progress_percent: pct,
                current_step: Some(format!("Reshaping array... {}%", pct)),
                error_message: None,
            });
        }

        Ok(())
    }
}

// ============ LSBLK JSON STRUCTURES ============

#[derive(Debug, Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Deserialize)]
struct LsblkDevice {
    name: String,
    size: Option<u64>,
    #[serde(rename = "type")]
    device_type: String,
    mountpoint: Option<String>,
    fstype: Option<String>,
    label: Option<String>,
    uuid: Option<String>,
    model: Option<String>,
    serial: Option<String>,
    tran: Option<String>,
    rm: Option<bool>,
    hotplug: Option<bool>,
    children: Option<Vec<LsblkDevice>>,
}

// ============ BLKID PARSED INFO ============

/// Parsed blkid info for a single device
struct BlkidInfo {
    fs_type: Option<String>,
    label: Option<String>,
    uuid: Option<String>,
}

// ============ SMARTCTL JSON STRUCTURES ============

#[derive(Debug, Deserialize)]
struct SmartctlOutput {
    model_name: Option<String>,
    serial_number: Option<String>,
    firmware_version: Option<String>,
    smart_status: Option<SmartStatus>,
    temperature: Option<SmartTemperature>,
    power_on_time: Option<PowerOnTime>,
    power_cycle_count: Option<u64>,
    ata_smart_attributes: Option<SmartAttributes>,
}

#[derive(Debug, Deserialize)]
struct SmartStatus {
    passed: bool,
}

#[derive(Debug, Deserialize)]
struct SmartTemperature {
    current: i32,
}

#[derive(Debug, Deserialize)]
struct PowerOnTime {
    hours: u64,
}

#[derive(Debug, Deserialize)]
struct SmartAttributes {
    table: Vec<SmartAttrEntry>,
}

#[derive(Debug, Deserialize)]
struct SmartAttrEntry {
    id: u8,
    name: String,
    value: u64,
    worst: u64,
    thresh: u64,
    raw: SmartAttrRaw,
}

#[derive(Debug, Deserialize)]
struct SmartAttrRaw {
    value: u64,
}
