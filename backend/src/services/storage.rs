use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;
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
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid10 => {
                    serde_json::json!({ "btrfs_uuid": format!("fake-btrfs-{}", &pool_id[..8]) }).to_string()
                }
                RaidType::Jbod | RaidType::Raid0 | RaidType::Raid1 | RaidType::Raid5 | RaidType::Raid10 => {
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
                RaidType::BtrfsSingle | RaidType::BtrfsRaid0 | RaidType::BtrfsRaid1 | RaidType::BtrfsRaid10 => {
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
                        self.mount(device, &mount_point, Some(&request.fs_type)).await?;
                    }
                }
            }
        }

        // Insert into database
        sqlx::query(
            "INSERT INTO storage_volumes (id, pool_id, name, fs_type, mount_point, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'mounted', ?, ?)"
        )
        .bind(&volume_id)
        .bind(pool_id)
        .bind(&request.name)
        .bind(&request.fs_type)
        .bind(&mount_point)
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
                self.mount(device, &volume.mount_point, Some(&volume.fs_type)).await?;
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
            self.mount(device, &pool_mount, Some("btrfs")).await?;

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
    async fn mount(&self, device: &str, mount_point: &str, fs_type: Option<&str>) -> Result<()> {
        tokio::fs::create_dir_all(mount_point).await?;

        let mut args = vec![device, mount_point];
        if let Some(fs) = fs_type {
            args.insert(0, "-t");
            args.insert(1, fs);
        }

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
