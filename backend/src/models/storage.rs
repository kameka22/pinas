use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Type de disque physique
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiskType {
    Hdd,
    Ssd,
    Nvme,
    Sd,
    Usb,
    Unknown,
}

impl Default for DiskType {
    fn default() -> Self {
        DiskType::Unknown
    }
}

/// Partition d'un disque
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub device_path: String,
    pub number: u32,
    pub size: u64,
    pub fs_type: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub mount_point: Option<String>,
    pub is_system: bool, // true si /flash ou /storage
}

/// Disque physique détecté
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    pub device_name: String,           // "sda", "nvme0n1", "mmcblk0"
    pub device_path: String,           // "/dev/sda"
    pub device_by_id: Option<String>,  // "/dev/disk/by-id/..."
    pub model: String,
    pub serial: Option<String>,
    pub size: u64,                     // bytes
    pub disk_type: DiskType,
    pub temperature: Option<i32>,      // Celsius (from S.M.A.R.T.)
    pub health_status: Option<String>, // "PASSED", "FAILED", etc.
    pub is_system: bool,               // Contient /flash ou /storage
    pub is_removable: bool,
    pub partitions: Vec<Partition>,
}

/// Données S.M.A.R.T. détaillées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartInfo {
    pub device_path: String,
    pub model: String,
    pub serial: Option<String>,
    pub firmware: Option<String>,
    pub health_status: String,         // "PASSED", "FAILED"
    pub temperature: Option<i32>,
    pub power_on_hours: Option<u64>,
    pub power_cycle_count: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub attributes: Vec<SmartAttribute>,
}

/// Attribut S.M.A.R.T. individuel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub value: u64,
    pub worst: u64,
    pub threshold: u64,
    pub raw_value: String,
}

/// Type de RAID pour les pools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RaidType {
    Basic,
    Jbod,
    Raid0,
    Raid1,
    Raid5,
    Raid6,
    Raid10,
    BtrfsSingle,
    BtrfsRaid0,
    BtrfsRaid1,
    BtrfsRaid5,
    BtrfsRaid6,
    BtrfsRaid10,
}

impl std::fmt::Display for RaidType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RaidType::Basic => write!(f, "basic"),
            RaidType::Jbod => write!(f, "jbod"),
            RaidType::Raid0 => write!(f, "raid0"),
            RaidType::Raid1 => write!(f, "raid1"),
            RaidType::Raid5 => write!(f, "raid5"),
            RaidType::Raid6 => write!(f, "raid6"),
            RaidType::Raid10 => write!(f, "raid10"),
            RaidType::BtrfsSingle => write!(f, "btrfs-single"),
            RaidType::BtrfsRaid0 => write!(f, "btrfs-raid0"),
            RaidType::BtrfsRaid1 => write!(f, "btrfs-raid1"),
            RaidType::BtrfsRaid5 => write!(f, "btrfs-raid5"),
            RaidType::BtrfsRaid6 => write!(f, "btrfs-raid6"),
            RaidType::BtrfsRaid10 => write!(f, "btrfs-raid10"),
        }
    }
}

impl std::str::FromStr for RaidType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(RaidType::Basic),
            "jbod" => Ok(RaidType::Jbod),
            "raid0" => Ok(RaidType::Raid0),
            "raid1" => Ok(RaidType::Raid1),
            "raid5" => Ok(RaidType::Raid5),
            "raid6" => Ok(RaidType::Raid6),
            "raid10" => Ok(RaidType::Raid10),
            "btrfs-single" => Ok(RaidType::BtrfsSingle),
            "btrfs-raid0" => Ok(RaidType::BtrfsRaid0),
            "btrfs-raid1" => Ok(RaidType::BtrfsRaid1),
            "btrfs-raid5" => Ok(RaidType::BtrfsRaid5),
            "btrfs-raid6" => Ok(RaidType::BtrfsRaid6),
            "btrfs-raid10" => Ok(RaidType::BtrfsRaid10),
            _ => Err(format!("Unknown RAID type: {}", s)),
        }
    }
}

/// Status d'un pool de stockage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PoolStatus {
    Normal,
    Degraded,
    Rebuilding,
    Expanding,
    Error,
    Creating,
}

impl Default for PoolStatus {
    fn default() -> Self {
        PoolStatus::Normal
    }
}

impl std::fmt::Display for PoolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolStatus::Normal => write!(f, "normal"),
            PoolStatus::Degraded => write!(f, "degraded"),
            PoolStatus::Rebuilding => write!(f, "rebuilding"),
            PoolStatus::Expanding => write!(f, "expanding"),
            PoolStatus::Error => write!(f, "error"),
            PoolStatus::Creating => write!(f, "creating"),
        }
    }
}

/// Pool de stockage (depuis la DB)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoragePool {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub raid_type: String,
    pub status: String,
    pub devices: String,          // JSON array
    pub total_size: Option<i64>,
    pub metadata: Option<String>, // JSON metadata
    pub created_at: String,
    pub updated_at: String,
}

/// Pool de stockage (pour l'API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePoolInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub raid_type: RaidType,
    pub status: PoolStatus,
    pub devices: Vec<String>,
    pub total_size: u64,
    pub used_size: u64,
    pub available_size: u64,
    pub volumes: Vec<VolumeInfo>,
    pub created_at: String,
}

/// Status d'un volume
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VolumeStatus {
    Mounted,
    Unmounted,
    Error,
    Creating,
}

impl Default for VolumeStatus {
    fn default() -> Self {
        VolumeStatus::Unmounted
    }
}

/// Volume (depuis la DB)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageVolume {
    pub id: String,
    pub pool_id: String,
    pub name: String,
    pub fs_type: String,
    pub mount_point: String,
    pub size_bytes: Option<i64>,
    pub status: String,
    pub mount_options: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Volume (pour l'API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub id: String,
    pub pool_id: String,
    pub name: String,
    pub fs_type: String,
    pub mount_point: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: u8,
    pub status: VolumeStatus,
    pub mount_options: Option<String>,
    pub created_at: String,
}

// ============ Request/Response DTOs ============

/// Requête de création de pool
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePoolRequest {
    pub name: String,
    pub description: Option<String>,
    pub raid_type: RaidType,
    pub devices: Vec<String>,  // device paths (by-id preferred)
    pub wipe_devices: bool,    // Wipe devices before creating pool
}

/// Requête de modification de pool
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePoolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Requête de création de volume
#[derive(Debug, Clone, Deserialize)]
pub struct CreateVolumeRequest {
    pub name: String,
    pub fs_type: String,       // ext4, btrfs, xfs
    pub size: Option<u64>,     // None = use all available space
    pub mount_options: Option<String>,
}

/// Requête de modification de volume
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVolumeRequest {
    pub mount_options: Option<String>,
}

/// Requête de redimensionnement de volume
#[derive(Debug, Clone, Deserialize)]
pub struct ResizeVolumeRequest {
    pub size: Option<u64>,  // None = use all available space
}

/// Requête de vérification filesystem
#[derive(Debug, Clone, Deserialize)]
pub struct FsckRequest {
    pub repair: bool,  // false = read-only check, true = repair
}

/// Statut d'un fsck en cours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsckStatus {
    pub task_id: String,
    pub volume_id: String,
    pub status: String,       // "running", "completed", "error"
    pub errors_found: u64,
    pub repaired: u64,
}

/// Mode de wipe
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WipeMode {
    Quick,     // sgdisk --zap-all (existing, instant)
    Zeros,     // dd if=/dev/zero (1 pass, ~minutes per GB)
    Secure,    // shred -vfz -n 3 (3 passes + zeros, slow)
}

impl Default for WipeMode {
    fn default() -> Self {
        WipeMode::Quick
    }
}

/// Requête de wipe disque (optionnel, default = Quick)
#[derive(Debug, Clone, Deserialize)]
pub struct WipeDiskRequest {
    pub mode: Option<WipeMode>,
}

/// Statut d'un wipe en cours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeStatus {
    pub task_id: String,
    pub device_name: String,
    pub status: String,
    pub progress: f32,
}

/// Disque candidat pour un pool
#[derive(Debug, Clone, Serialize)]
pub struct DiskCandidate {
    pub device_path: String,
    pub device_by_id: Option<String>,
    pub model: String,
    pub size: u64,
    pub disk_type: DiskType,
    pub is_empty: bool,        // No partitions or all partitions unmounted
}

// ============ Health Monitoring ============

/// Santé d'un pool (RAID health)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealthInfo {
    pub pool_id: String,
    pub status: PoolStatus,
    pub raid_type: String,
    pub devices: Vec<DeviceHealthInfo>,
    pub rebuild_progress: Option<f32>,    // 0-100 if rebuilding
    pub last_scrub: Option<ScrubInfo>,
}

/// Santé d'un device dans un pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHealthInfo {
    pub device_path: String,
    pub state: String,          // "active", "spare", "faulty", "removed"
    pub errors: DeviceErrorInfo,
}

/// Erreurs d'un device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceErrorInfo {
    pub read_errors: u64,
    pub write_errors: u64,
    pub corruption_errors: u64,
    pub generation_errors: u64,
}

/// Alerte stockage envoyée via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAlertEvent {
    pub pool_id: String,
    pub pool_name: String,
    pub alert_type: String,     // "degraded", "error", "rebuilt", "scrub_error"
    pub previous_status: String,
    pub new_status: String,
    pub message: String,
}

/// Informations sur le dernier scrub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrubInfo {
    pub date: String,
    pub duration_secs: Option<u64>,
    pub result: String,          // "ok", "errors_found", "cancelled"
    pub errors_found: u64,
}

/// Statut d'un scrub en cours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrubStatus {
    pub task_id: String,
    pub pool_id: String,
    pub status: String,          // "running", "completed", "error"
    pub progress: f32,           // 0-100
    pub errors_found: u64,
    pub started_at: String,
}

// ============ SMART Scheduled Tests ============

/// SMART test schedule (DB row)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SmartTestSchedule {
    pub id: String,
    pub device_path: String,
    pub device_name: String,
    pub test_type: String,
    pub interval_hours: i64,
    pub last_run: Option<String>,
    pub next_run: String,
    pub last_result: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// SMART test schedule (API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTestScheduleInfo {
    pub id: String,
    pub device_path: String,
    pub device_name: String,
    pub test_type: String,
    pub interval_hours: i64,
    pub last_run: Option<String>,
    pub next_run: String,
    pub last_result: Option<SmartTestResult>,
    pub enabled: bool,
}

/// Result of a SMART self-test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTestResult {
    pub status: String,
    pub errors: u64,
    pub duration_secs: u64,
    pub completed_at: String,
}

/// Request to create a SMART schedule
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSmartScheduleRequest {
    pub device_path: String,
    pub test_type: String,
    pub interval_hours: Option<i64>,
}

/// Request to run a SMART test on-demand
#[derive(Debug, Clone, Deserialize)]
pub struct RunSmartTestRequest {
    pub test_type: String,
}

/// Status of a running SMART test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTestStatus {
    pub task_id: String,
    pub device_name: String,
    pub test_type: String,
    pub status: String,
    pub progress: u8,
}

/// SMART self-test history entry (from smartctl -l selftest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTestHistoryEntry {
    pub num: u32,
    pub test_type: String,
    pub status: String,
    pub remaining_percent: u8,
    pub lifetime_hours: u64,
    pub lba_of_first_error: Option<u64>,
}

/// Toggle schedule request
#[derive(Debug, Clone, Deserialize)]
pub struct ToggleSmartScheduleRequest {
    pub enabled: bool,
}

// ============ Disk Power Management ============

/// Power settings for a disk (DB row + API)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiskPowerSettings {
    pub device_path: String,
    pub device_name: String,
    pub apm_level: Option<i32>,
    pub spindown_minutes: Option<i32>,
    pub write_cache: Option<bool>,
    pub updated_at: String,
}

/// Request to update disk power settings
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDiskPowerRequest {
    pub apm_level: Option<i32>,
    pub spindown_minutes: Option<i32>,
    pub write_cache: Option<bool>,
}

// ============ Btrfs Snapshots ============

/// Btrfs snapshot (DB row)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BtrfsSnapshot {
    pub id: String,
    pub volume_id: String,
    pub name: String,
    pub path: String,
    pub snapshot_type: String,
    pub size_bytes: Option<i64>,
    pub created_at: String,
}

/// Btrfs snapshot (API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsSnapshotInfo {
    pub id: String,
    pub volume_id: String,
    pub name: String,
    pub path: String,
    pub snapshot_type: String,
    pub size_bytes: Option<u64>,
    pub created_at: String,
}

/// Request to create a snapshot
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSnapshotRequest {
    pub name: String,
}

// ============ RAID Grow/Expand ============

/// Request to grow a pool by adding disks
#[derive(Debug, Clone, Deserialize)]
pub struct GrowPoolRequest {
    pub devices: Vec<String>,
    pub wipe_devices: bool,
}

/// Status of a pool grow operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowPoolStatus {
    pub task_id: String,
    pub pool_id: String,
    pub status: String,
    pub progress: f32,
}
