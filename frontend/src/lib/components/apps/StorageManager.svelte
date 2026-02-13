<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { api, type Disk, type StoragePool, type VolumeInfo, type DiskCandidate, type SmartInfo, type RaidType, type PoolHealthInfo, type ScrubStatus, type WipeMode, type FsckStatus, type SmartTestHistoryEntry, type SmartTestScheduleInfo, type SmartTestStatus, type DiskPowerSettings, type BtrfsSnapshotInfo, type GrowPoolStatus } from '$lib/stores/api';
	import { taskProgress } from '$lib/stores/websocket';
	import { t } from '$lib/i18n';

	// State
	let loading = true;
	let error: string | null = null;

	// Data
	let disks: Disk[] = [];
	let pools: StoragePool[] = [];
	let candidates: DiskCandidate[] = [];

	// UI state
	let sidebarSection: 'overview' | 'storage' | 'harddisk' | 'external' = 'storage';
	let activeTab: 'pools' | 'organizing' | 'settings' = 'pools';
	let expandedPools: Set<string> = new Set();
	let showCreateMenu = false;

	// Modals
	let showCreatePoolModal = false;
	let showCreateVolumeModal = false;
	let showSmartModal = false;
	let showDeletePoolModal = false;
	let showDeleteVolumeModal = false;
	let showEditPoolModal = false;
	let showWipeDiskModal = false;
	let showDiskDetailsModal = false;
	let showResizeModal = false;
	let showMountOptionsModal = false;
	let showFsckModal = false;
	let showPowerModal = false;
	let showSnapshotsModal = false;
	let showGrowPoolModal = false;
	let showCreateScheduleModal = false;

	// Pool context menu
	let poolMenuTarget: StoragePool | null = null;
	let poolMenuPosition = { x: 0, y: 0 };

	// Volume context menu
	let volumeMenuTarget: VolumeInfo | null = null;
	let volumeMenuPosition = { x: 0, y: 0 };

	// Create pool wizard state
	let createPoolStep = 1;
	let newPool = {
		name: '',
		description: '',
		raidType: 'basic' as RaidType,
		selectedDevices: [] as string[],
		wipeDevices: true
	};

	// Create volume state
	let newVolume = {
		poolId: '',
		name: '',
		fsType: 'ext4'
	};

	// SMART info state
	let smartInfo: SmartInfo | null = null;
	let smartLoading = false;
	let selectedDiskForSmart: Disk | null = null;

	// Delete confirmation state
	let poolToDelete: StoragePool | null = null;
	let volumeToDelete: VolumeInfo | null = null;

	// Edit pool state
	let poolToEdit: StoragePool | null = null;
	let editPoolData = { name: '', description: '' };

	// Wipe disk state
	let diskToWipe: Disk | null = null;
	let wipingDisk = false;

	// Disk details state
	let selectedDiskDetails: Disk | null = null;

	// Resize state
	let volumeToResize: VolumeInfo | null = null;
	let resizeData = { useMax: true, newSizeGB: 0 };

	// Mount options state
	let volumeForMountOptions: VolumeInfo | null = null;
	let mountOptionsPreset: 'default' | 'ssd' | 'nas' | 'custom' = 'default';
	let mountOptionsCustom = '';

	// Fsck state
	let volumeToCheck: VolumeInfo | null = null;
	let fsckRepair = false;
	let checkingVolumes: Set<string> = new Set();
	let fsckTaskIds: Record<string, string> = {}; // volumeId -> taskId

	// Wipe mode state
	let wipeMode: WipeMode = 'quick';
	let wipingTaskId: string | null = null;

	// Scrubbing state
	let scrubbingPools: Set<string> = new Set();

	// Health info per pool
	let poolHealth: Record<string, PoolHealthInfo> = {};
	let scrubTaskIds: Record<string, string> = {}; // poolId -> taskId

	// SMART test state
	let smartTab: 'info' | 'tests' | 'schedules' = 'info';
	let smartTestHistory: SmartTestHistoryEntry[] = [];
	let smartSchedules: SmartTestScheduleInfo[] = [];
	let runningSmartTest = false;
	let smartTestTaskId: string | null = null;

	// Power settings state
	let powerDisk: Disk | null = null;
	let powerSettings: DiskPowerSettings | null = null;
	let powerLoading = false;
	let apmPreset: 'low' | 'balanced' | 'max' | 'disabled' | 'custom' = 'balanced';

	// Snapshots state
	let snapshotVolume: VolumeInfo | null = null;
	let snapshots: BtrfsSnapshotInfo[] = [];
	let snapshotLoading = false;
	let newSnapshotName = '';

	// Grow pool state
	let growPool_: StoragePool | null = null;
	let growSelectedDevices: string[] = [];
	let growWipeDevices = true;
	let growingPools: Set<string> = new Set();
	let growTaskIds: Record<string, string> = {};

	// Schedule creation state
	let newScheduleDevice = '';
	let newScheduleTestType = 'short';
	let newScheduleInterval = 168; // weekly

	// RAID type info - translations are applied in the template via getRaidTypeName/getRaidTypeDesc
	const raidTypes: { type: RaidType; nameKey: string; minDisks: number; descKey: string }[] = [
		{ type: 'basic', nameKey: 'basic', minDisks: 1, descKey: 'basicDesc' },
		{ type: 'jbod', nameKey: 'jbod', minDisks: 1, descKey: 'jbodDesc' },
		{ type: 'raid0', nameKey: 'raid0', minDisks: 2, descKey: 'raid0Desc' },
		{ type: 'raid1', nameKey: 'raid1', minDisks: 2, descKey: 'raid1Desc' },
		{ type: 'raid5', nameKey: 'raid5', minDisks: 3, descKey: 'raid5Desc' },
		{ type: 'raid10', nameKey: 'raid10', minDisks: 4, descKey: 'raid10Desc' },
		{ type: 'raid6', nameKey: 'raid6', minDisks: 4, descKey: 'raid6Desc' },
		{ type: 'btrfs-single', nameKey: 'btrfsSingle', minDisks: 1, descKey: 'btrfsSingleDesc' },
		{ type: 'btrfs-raid0', nameKey: 'btrfsRaid0', minDisks: 2, descKey: 'btrfsRaid0Desc' },
		{ type: 'btrfs-raid1', nameKey: 'btrfsRaid1', minDisks: 2, descKey: 'btrfsRaid1Desc' },
		{ type: 'btrfs-raid5', nameKey: 'btrfsRaid5', minDisks: 3, descKey: 'btrfsRaid5Desc' },
		{ type: 'btrfs-raid6', nameKey: 'btrfsRaid6', minDisks: 4, descKey: 'btrfsRaid6Desc' },
		{ type: 'btrfs-raid10', nameKey: 'btrfsRaid10', minDisks: 4, descKey: 'btrfsRaid10Desc' },
	];

	// Helper to get RAID type translated name
	function getRaidTypeName(nameKey: string): string {
		return $t.storageManager?.raidTypes?.[nameKey] || nameKey;
	}

	// Helper to get RAID type translated description
	function getRaidTypeDesc(descKey: string): string {
		return $t.storageManager?.raidTypes?.[descKey] || descKey;
	}

	// Helper to get status translation
	function getStatusLabel(status: string): string {
		return $t.storageManager?.status?.[status] || status.charAt(0).toUpperCase() + status.slice(1);
	}

	// Utility functions
	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'normal':
			case 'mounted':
				return 'bg-blue-500 text-white';
			case 'degraded':
			case 'warning':
				return 'bg-amber-500 text-white';
			case 'error':
			case 'critical':
				return 'bg-red-500 text-white';
			case 'creating':
			case 'rebuilding':
			case 'expanding':
				return 'bg-purple-500 text-white';
			default:
				return 'bg-slate-500 text-white';
		}
	}

	function getDiskTypeIcon(type: string): string {
		switch (type) {
			case 'ssd':
			case 'nvme':
				return 'mdi:chip';
			case 'sd':
				return 'mdi:sd';
			case 'usb':
				return 'mdi:usb';
			default:
				return 'mdi:harddisk';
		}
	}

	function getRaidIcon(raidType: RaidType): string {
		if (raidType === 'basic' || raidType === 'btrfs-single') {
			return 'mdi:database';
		}
		return 'mdi:database-multiple';
	}

	function getAvailableRaidTypes(): typeof raidTypes {
		const numDevices = newPool.selectedDevices.length;
		return raidTypes.filter(r => numDevices >= r.minDisks);
	}

	function calculateAvailableSpace(): number {
		const selectedDisks = candidates.filter(c => newPool.selectedDevices.includes(c.device_path));
		const totalSize = selectedDisks.reduce((sum, d) => sum + d.size, 0);

		const smallestDisk = Math.min(...selectedDisks.map(d => d.size)) || 0;
		switch (newPool.raidType) {
			case 'raid1':
			case 'btrfs-raid1':
				return smallestDisk;
			case 'raid5':
			case 'btrfs-raid5':
				return totalSize - smallestDisk;
			case 'raid6':
			case 'btrfs-raid6':
				return totalSize - 2 * smallestDisk;
			case 'raid10':
				return totalSize / 2;
			default:
				return totalSize;
		}
	}

	// Data loading
	async function loadData() {
		loading = true;
		error = null;
		try {
			[disks, pools, candidates] = await Promise.all([
				api.getDisks(),
				api.getPools(),
				api.getDiskCandidates()
			]);
			// Auto-expand pools with volumes
			pools.forEach(p => {
				if (p.volumes.length > 0) {
					expandedPools.add(p.id);
				}
			});
			expandedPools = expandedPools; // Trigger reactivity

			// Load health info for each pool
			await loadPoolsHealth();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.loadFailed;
		} finally {
			loading = false;
		}
	}

	async function loadPoolsHealth() {
		const healthPromises = pools.map(async (pool) => {
			try {
				const health = await api.getPoolHealth(pool.id);
				poolHealth[pool.id] = health;
			} catch {
				// Health info is optional, don't fail on it
			}
		});
		await Promise.all(healthPromises);
		poolHealth = poolHealth; // Trigger reactivity
	}

	function formatDuration(secs: number): string {
		if (secs < 60) return `${secs}s`;
		if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
		const h = Math.floor(secs / 3600);
		const m = Math.floor((secs % 3600) / 60);
		return `${h}h ${m}m`;
	}

	function formatRelativeDate(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
		if (diffDays === 0) return $t.storageManager.health?.today || 'Today';
		if (diffDays === 1) return $t.storageManager.health?.yesterday || 'Yesterday';
		if (diffDays < 30) return `${diffDays} ${$t.storageManager.health?.daysAgo || 'days ago'}`;
		return date.toLocaleDateString();
	}

	// Pool operations
	function togglePoolExpand(poolId: string) {
		if (expandedPools.has(poolId)) {
			expandedPools.delete(poolId);
		} else {
			expandedPools.add(poolId);
		}
		expandedPools = expandedPools;
	}

	function openPoolMenu(event: MouseEvent, pool: StoragePool) {
		event.stopPropagation();
		poolMenuTarget = pool;
		poolMenuPosition = { x: event.clientX, y: event.clientY };
		volumeMenuTarget = null;
	}

	function openVolumeMenu(event: MouseEvent, volume: VolumeInfo) {
		event.stopPropagation();
		volumeMenuTarget = volume;
		volumeMenuPosition = { x: event.clientX, y: event.clientY };
		poolMenuTarget = null;
	}

	function closeMenus() {
		poolMenuTarget = null;
		volumeMenuTarget = null;
		showCreateMenu = false;
	}

	// Create pool
	function openCreatePool() {
		showCreateMenu = false;
		createPoolStep = 1;
		newPool = {
			name: `Storage Pool ${pools.length + 1}`,
			description: '',
			raidType: 'basic',
			selectedDevices: [],
			wipeDevices: true
		};
		showCreatePoolModal = true;
	}

	function toggleDeviceSelection(devicePath: string) {
		if (newPool.selectedDevices.includes(devicePath)) {
			newPool.selectedDevices = newPool.selectedDevices.filter(d => d !== devicePath);
		} else {
			newPool.selectedDevices = [...newPool.selectedDevices, devicePath];
		}
		// Reset RAID type if not enough devices
		const available = getAvailableRaidTypes();
		if (!available.find(r => r.type === newPool.raidType)) {
			newPool.raidType = available[0]?.type || 'basic';
		}
	}

	async function createPool() {
		try {
			await api.createPool({
				name: newPool.name,
				description: newPool.description || undefined,
				raid_type: newPool.raidType,
				devices: newPool.selectedDevices,
				wipe_devices: newPool.wipeDevices
			});
			showCreatePoolModal = false;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.createPoolFailed;
		}
	}

	// Create volume
	function openCreateVolume(poolId?: string) {
		showCreateMenu = false;
		poolMenuTarget = null;
		newVolume = {
			poolId: poolId || '',
			name: `Volume ${pools.reduce((sum, p) => sum + p.volumes.length, 0) + 1}`,
			fsType: 'ext4'
		};
		showCreateVolumeModal = true;
	}

	async function createVolume() {
		try {
			await api.createVolume(newVolume.poolId, {
				name: newVolume.name,
				fs_type: newVolume.fsType,
				mount_options: getCreateVolumeMountOptions()
			});
			showCreateVolumeModal = false;
			createVolumeMountPreset = 'default';
			createVolumeMountCustom = '';
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.createVolumeFailed;
		}
	}

	// Delete operations
	function confirmDeletePool(pool: StoragePool) {
		poolMenuTarget = null;
		poolToDelete = pool;
		showDeletePoolModal = true;
	}

	async function deletePool() {
		if (!poolToDelete) return;
		try {
			await api.deletePool(poolToDelete.id);
			showDeletePoolModal = false;
			poolToDelete = null;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.deletePoolFailed;
		}
	}

	function confirmDeleteVolume(volume: VolumeInfo) {
		volumeMenuTarget = null;
		volumeToDelete = volume;
		showDeleteVolumeModal = true;
	}

	async function deleteVolume() {
		if (!volumeToDelete) return;
		try {
			await api.deleteVolume(volumeToDelete.id);
			showDeleteVolumeModal = false;
			volumeToDelete = null;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.deleteVolumeFailed;
		}
	}

	// Volume mount/unmount
	async function toggleVolumeMount(volume: VolumeInfo) {
		volumeMenuTarget = null;
		try {
			if (volume.status === 'mounted') {
				await api.unmountVolume(volume.id);
			} else {
				await api.mountVolume(volume.id);
			}
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.toggleMountFailed;
		}
	}

	// SMART info
	async function showDiskSmart(disk: Disk) {
		selectedDiskForSmart = disk;
		smartTab = 'info';
		smartLoading = true;
		smartInfo = null;
		showSmartModal = true;
		try {
			smartInfo = await api.getDiskSmartInfo(disk.device_name);
			await loadSmartHistory(disk.device_name);
			await loadSmartSchedules();
		} catch (e) {
			// Error is shown in modal, not in main error state
		} finally {
			smartLoading = false;
		}
	}

	// Pool scrub
	async function scrubPool(pool: StoragePool) {
		poolMenuTarget = null;
		scrubbingPools.add(pool.id);
		scrubbingPools = scrubbingPools;
		try {
			const result = await api.scrubPool(pool.id);
			scrubTaskIds[pool.id] = result.task_id;
			scrubTaskIds = scrubTaskIds;
			// Scrub runs in background - progress comes via WebSocket task.progress
			// The scrubbingPools set will be cleared when task.progress shows completed/failed
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.scrubFailed;
			scrubbingPools.delete(pool.id);
			scrubbingPools = scrubbingPools;
		}
	}

	// Edit pool
	function openEditPool(pool: StoragePool) {
		poolMenuTarget = null;
		poolToEdit = pool;
		editPoolData = { name: pool.name, description: pool.description || '' };
		showEditPoolModal = true;
	}

	async function savePoolEdit() {
		if (!poolToEdit) return;
		try {
			await api.updatePool(poolToEdit.id, {
				name: editPoolData.name,
				description: editPoolData.description || undefined
			});
			showEditPoolModal = false;
			poolToEdit = null;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.editPoolFailed;
		}
	}

	// Wipe disk
	function confirmWipeDiskWithReset(disk: Disk) {
		diskToWipe = disk;
		wipeMode = 'quick';
		showWipeDiskModal = true;
	}

	async function wipeDisk() {
		if (!diskToWipe) return;
		wipingDisk = true;
		try {
			const result = await api.wipeDisk(diskToWipe.device_name, wipeMode);
			if (wipeMode !== 'quick' && result && typeof result === 'object' && 'task_id' in result) {
				wipingTaskId = (result as any).task_id;
				// Background wipe — track via WebSocket
			}
			showWipeDiskModal = false;
			diskToWipe = null;
			wipeMode = 'quick';
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.wipeDiskFailed;
		} finally {
			wipingDisk = false;
		}
	}

	// Disk details (partitions)
	function showDiskDetails(disk: Disk) {
		selectedDiskDetails = disk;
		showDiskDetailsModal = true;
	}

	// Resize volume
	function openResizeVolume(volume: VolumeInfo) {
		volumeMenuTarget = null;
		volumeToResize = volume;
		resizeData = { useMax: true, newSizeGB: Math.round(volume.size / (1024 * 1024 * 1024)) };
		showResizeModal = true;
	}

	async function resizeVolume() {
		if (!volumeToResize) return;
		try {
			await api.resizeVolume(volumeToResize.id, {
				size: resizeData.useMax ? undefined : resizeData.newSizeGB * 1024 * 1024 * 1024
			});
			showResizeModal = false;
			volumeToResize = null;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.resizeFailed;
		}
	}

	// Mount options
	function openMountOptions(volume: VolumeInfo) {
		volumeMenuTarget = null;
		volumeForMountOptions = volume;
		const opts = volume.mount_options || '';
		if (!opts || opts === 'defaults') {
			mountOptionsPreset = 'default';
			mountOptionsCustom = '';
		} else if (opts === 'noatime,discard') {
			mountOptionsPreset = 'ssd';
			mountOptionsCustom = '';
		} else if (opts === 'noatime,nofail') {
			mountOptionsPreset = 'nas';
			mountOptionsCustom = '';
		} else {
			mountOptionsPreset = 'custom';
			mountOptionsCustom = opts;
		}
		showMountOptionsModal = true;
	}

	async function saveMountOptions() {
		if (!volumeForMountOptions) return;
		let opts = '';
		switch (mountOptionsPreset) {
			case 'default': opts = 'defaults'; break;
			case 'ssd': opts = 'noatime,discard'; break;
			case 'nas': opts = 'noatime,nofail'; break;
			case 'custom': opts = mountOptionsCustom; break;
		}
		try {
			await api.updateVolume(volumeForMountOptions.id, { mount_options: opts });
			showMountOptionsModal = false;
			volumeForMountOptions = null;
			await loadData();
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.updateVolumeFailed;
		}
	}

	// Mount options for create volume
	let createVolumeMountPreset: 'default' | 'ssd' | 'nas' | 'custom' = 'default';
	let createVolumeMountCustom = '';

	function getCreateVolumeMountOptions(): string | undefined {
		switch (createVolumeMountPreset) {
			case 'ssd': return 'noatime,discard';
			case 'nas': return 'noatime,nofail';
			case 'custom': return createVolumeMountCustom || undefined;
			default: return undefined;
		}
	}

	// Filesystem check
	function openFsckVolume(volume: VolumeInfo) {
		volumeMenuTarget = null;
		volumeToCheck = volume;
		fsckRepair = false;
		showFsckModal = true;
	}

	async function checkFilesystem() {
		if (!volumeToCheck) return;
		try {
			const result = await api.checkVolume(volumeToCheck.id, { repair: fsckRepair });
			fsckTaskIds[volumeToCheck.id] = result.task_id;
			fsckTaskIds = fsckTaskIds;
			checkingVolumes.add(volumeToCheck.id);
			checkingVolumes = checkingVolumes;
			showFsckModal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : $t.storageManager.errors.checkFailed;
		}
	}

	// SMART test functions
	async function loadSmartHistory(deviceName: string) {
		try {
			smartTestHistory = await api.getSmartTestHistory(deviceName);
		} catch (e) {
			smartTestHistory = [];
		}
	}

	async function loadSmartSchedules() {
		try {
			smartSchedules = await api.getSmartSchedules();
		} catch (e) {
			smartSchedules = [];
		}
	}

	async function runSmartTest(testType: string) {
		if (!selectedDiskForSmart) return;
		runningSmartTest = true;
		try {
			const result = await api.runSmartTest(selectedDiskForSmart.device_name, testType);
			smartTestTaskId = result.task_id;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start SMART test';
		}
	}

	async function openSmartWithTab(disk: Disk, tab: 'info' | 'tests' | 'schedules') {
		selectedDiskForSmart = disk;
		smartTab = tab;
		smartLoading = true;
		smartInfo = null;
		showSmartModal = true;
		try {
			smartInfo = await api.getDiskSmartInfo(disk.device_name);
			if (tab === 'tests') await loadSmartHistory(disk.device_name);
			if (tab === 'schedules') await loadSmartSchedules();
		} catch (e) {
			// Shown in modal
		} finally {
			smartLoading = false;
		}
	}

	async function createSmartSchedule() {
		if (!newScheduleDevice) return;
		try {
			await api.createSmartSchedule({
				device_path: newScheduleDevice,
				test_type: newScheduleTestType,
				interval_hours: newScheduleInterval
			});
			await loadSmartSchedules();
			showCreateScheduleModal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create schedule';
		}
	}

	async function deleteSmartSchedule(id: string) {
		try {
			await api.deleteSmartSchedule(id);
			await loadSmartSchedules();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete schedule';
		}
	}

	async function toggleSchedule(id: string, enabled: boolean) {
		try {
			await api.toggleSmartSchedule(id, enabled);
			await loadSmartSchedules();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to toggle schedule';
		}
	}

	// Power management functions
	async function openPowerSettings(disk: Disk) {
		powerDisk = disk;
		powerLoading = true;
		showPowerModal = true;
		try {
			powerSettings = await api.getDiskPowerSettings(disk.device_name);
			// Determine preset from current values
			if (!powerSettings.apm_level) apmPreset = 'disabled';
			else if (powerSettings.apm_level <= 128) apmPreset = 'low';
			else if (powerSettings.apm_level <= 192) apmPreset = 'balanced';
			else apmPreset = 'max';
		} catch (e) {
			powerSettings = null;
		} finally {
			powerLoading = false;
		}
	}

	async function savePowerSettings() {
		if (!powerDisk || !powerSettings) return;
		let apmLevel: number | null = null;
		switch (apmPreset) {
			case 'low': apmLevel = 128; break;
			case 'balanced': apmLevel = 192; break;
			case 'max': apmLevel = 254; break;
			case 'disabled': apmLevel = null; break;
			case 'custom': apmLevel = powerSettings.apm_level; break;
		}
		try {
			await api.setDiskPowerSettings(powerDisk.device_name, {
				apm_level: apmLevel,
				spindown_minutes: powerSettings.spindown_minutes,
				write_cache: powerSettings.write_cache,
			});
			showPowerModal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save power settings';
		}
	}

	// Snapshot functions
	async function openSnapshots(volume: VolumeInfo) {
		volumeMenuTarget = null;
		snapshotVolume = volume;
		snapshotLoading = true;
		snapshots = [];
		showSnapshotsModal = true;
		const now = new Date();
		newSnapshotName = `snap-${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}-${String(now.getHours()).padStart(2,'0')}${String(now.getMinutes()).padStart(2,'0')}`;
		try {
			snapshots = await api.listSnapshots(volume.id);
		} catch (e) {
			// Shown in modal
		} finally {
			snapshotLoading = false;
		}
	}

	async function createSnapshot() {
		if (!snapshotVolume || !newSnapshotName) return;
		try {
			await api.createSnapshot(snapshotVolume.id, { name: newSnapshotName });
			snapshots = await api.listSnapshots(snapshotVolume.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create snapshot';
		}
	}

	async function deleteSnapshot(snapId: string) {
		if (!snapshotVolume) return;
		try {
			await api.deleteSnapshot(snapshotVolume.id, snapId);
			snapshots = await api.listSnapshots(snapshotVolume.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete snapshot';
		}
	}

	// Grow pool functions
	function openGrowPool(pool: StoragePool) {
		poolMenuTarget = null;
		growPool_ = pool;
		growSelectedDevices = [];
		growWipeDevices = true;
		showGrowPoolModal = true;
	}

	async function startGrowPool() {
		if (!growPool_ || growSelectedDevices.length === 0) return;
		try {
			const result = await api.growPool(growPool_.id, {
				devices: growSelectedDevices,
				wipe_devices: growWipeDevices,
			});
			growTaskIds[growPool_.id] = result.task_id;
			growTaskIds = growTaskIds;
			growingPools.add(growPool_.id);
			growingPools = growingPools;
			showGrowPoolModal = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start pool expansion';
		}
	}

	// Click outside handler
	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.context-menu') && !target.closest('.menu-trigger') && !target.closest('.dropdown-menu')) {
			closeMenus();
		}
	}

	onMount(() => {
		loadData();
		document.addEventListener('click', handleClickOutside);

		// Subscribe to task progress to track scrub/grow/smart completion
		const unsubscribe = taskProgress.subscribe((tasks) => {
			for (const [poolId, taskId] of Object.entries(scrubTaskIds)) {
				const progress = tasks[taskId];
				if (progress && (progress.status === 'completed' || progress.status === 'failed')) {
					scrubbingPools.delete(poolId);
					scrubbingPools = scrubbingPools;
					delete scrubTaskIds[poolId];
					scrubTaskIds = scrubTaskIds;
					loadPoolsHealth();
				}
			}
			for (const [volumeId, taskId] of Object.entries(fsckTaskIds)) {
				const progress = tasks[taskId];
				if (progress && (progress.status === 'completed' || progress.status === 'failed')) {
					checkingVolumes.delete(volumeId);
					checkingVolumes = checkingVolumes;
					delete fsckTaskIds[volumeId];
					fsckTaskIds = fsckTaskIds;
					loadData();
				}
			}
			for (const [poolId, taskId] of Object.entries(growTaskIds)) {
				const progress = tasks[taskId];
				if (progress && (progress.status === 'completed' || progress.status === 'failed')) {
					growingPools.delete(poolId);
					growingPools = growingPools;
					delete growTaskIds[poolId];
					growTaskIds = growTaskIds;
					loadData();
				}
			}
			// Track SMART test progress
			if (smartTestTaskId) {
				const progress = tasks[smartTestTaskId];
				if (progress && (progress.status === 'completed' || progress.status === 'failed')) {
					runningSmartTest = false;
					smartTestTaskId = null;
					if (selectedDiskForSmart) loadSmartHistory(selectedDiskForSmart.device_name);
				}
			}
		});

		return () => {
			document.removeEventListener('click', handleClickOutside);
			unsubscribe();
		};
	});
</script>

<div class="storage-manager">
	<!-- Sidebar -->
	<aside class="sidebar">
		<button
			class="sidebar-item"
			class:active={sidebarSection === 'overview'}
			onclick={() => sidebarSection = 'overview'}
		>
			<Icon icon="mdi:view-dashboard-outline" class="w-5 h-5" />
			<span>{$t.storageManager.sidebar.overview}</span>
		</button>
		<button
			class="sidebar-item"
			class:active={sidebarSection === 'storage'}
			onclick={() => sidebarSection = 'storage'}
		>
			<Icon icon="mdi:database" class="w-5 h-5" />
			<span>{$t.storageManager.sidebar.storage}</span>
		</button>
		<button
			class="sidebar-item"
			class:active={sidebarSection === 'harddisk'}
			onclick={() => sidebarSection = 'harddisk'}
		>
			<Icon icon="mdi:harddisk" class="w-5 h-5" />
			<span>{$t.storageManager.sidebar.hardDisk}</span>
		</button>
		<button
			class="sidebar-item"
			class:active={sidebarSection === 'external'}
			onclick={() => sidebarSection = 'external'}
		>
			<Icon icon="mdi:usb" class="w-5 h-5" />
			<span>{$t.storageManager.sidebar.externalStorage}</span>
		</button>
	</aside>

	<!-- Main content -->
	<main class="main-content">
		{#if sidebarSection === 'storage'}
			<!-- Tabs -->
			<nav class="tabs">
				<button class="tab" class:active={activeTab === 'pools'} onclick={() => activeTab = 'pools'}>
					{$t.storageManager.tabs.poolsVolumes}
				</button>
				<button class="tab" class:active={activeTab === 'organizing'} onclick={() => activeTab = 'organizing'}>
					{$t.storageManager.tabs.dataOrganizing}
				</button>
				<button class="tab" class:active={activeTab === 'settings'} onclick={() => activeTab = 'settings'}>
					{$t.storageManager.tabs.advancedSettings}
				</button>
			</nav>

			{#if activeTab === 'pools'}
				<!-- Create button -->
				<div class="toolbar">
					<div class="create-dropdown">
						<button class="btn-primary menu-trigger" onclick={() => showCreateMenu = !showCreateMenu}>
							{$t.storageManager.pools.create}
							<Icon icon={showCreateMenu ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						</button>
						{#if showCreateMenu}
							<div class="dropdown-menu">
								<button onclick={openCreatePool}>
									<Icon icon="mdi:database-plus" class="w-4 h-4" />
									{$t.storageManager.pools.createPool}
								</button>
								<button onclick={() => openCreateVolume()} disabled={pools.length === 0}>
									<Icon icon="mdi:folder-plus" class="w-4 h-4" />
									{$t.storageManager.pools.createVolume}
								</button>
							</div>
						{/if}
					</div>
				</div>

				<!-- Loading/Error states -->
				{#if loading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
						<p>{$t.storageManager.messages.loading}</p>
					</div>
				{:else if error}
					<div class="error-state">
						<Icon icon="mdi:alert-circle" class="w-8 h-8 text-red-500" />
						<p>{error}</p>
						<button class="btn-secondary" onclick={loadData}>{$t.common.retry}</button>
					</div>
				{:else if pools.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:database-off" class="w-16 h-16 text-slate-300" />
						<p>{$t.storageManager.pools.noPoolsConfigured}</p>
						<button class="btn-primary" onclick={openCreatePool}>{$t.storageManager.pools.createPool}</button>
					</div>
				{:else}
					<!-- Pool list -->
					<div class="pool-list">
						{#each pools as pool}
							<div class="pool-card">
								<!-- Pool header -->
								<div class="pool-header" onclick={() => togglePoolExpand(pool.id)}>
									<button class="expand-btn" class:expanded={expandedPools.has(pool.id)}>
										<Icon icon="mdi:chevron-down" class="w-5 h-5" />
									</button>
									<div class="pool-icon">
										<Icon icon={getRaidIcon(pool.raid_type)} class="w-8 h-8 text-slate-600" />
									</div>
									<div class="pool-info">
										<div class="pool-title">
											<span class="pool-name">{pool.name}</span>
											{#if pool.description}
												<span class="pool-desc">{pool.description}</span>
											{/if}
										</div>
										<div class="pool-meta">
											<span class="status-badge {getStatusColor(pool.status)}">
												{getStatusLabel(pool.status)}
											</span>
											<span class="meta-item">{pool.raid_type.toUpperCase()}</span>
											<span class="meta-item">{pool.devices.length} {pool.devices.length > 1 ? $t.storageManager.pools.disks : $t.storageManager.pools.disk}</span>
											<span class="meta-item">{formatBytes(pool.total_size)}</span>
										</div>
										{#if poolHealth[pool.id]}
											<div class="pool-health-row">
												{#if poolHealth[pool.id].rebuild_progress != null}
													<span class="health-item rebuild">
														<Icon icon="mdi:sync" class="w-3.5 h-3.5 animate-spin" />
														{$t.storageManager.health?.rebuilding || 'Rebuilding'}: {poolHealth[pool.id].rebuild_progress?.toFixed(1)}%
													</span>
												{/if}
												{#if scrubbingPools.has(pool.id)}
													<span class="health-item scrubbing">
														<Icon icon="mdi:broom" class="w-3.5 h-3.5 animate-spin" />
														{$t.storageManager.contextMenu.scrubbing}
													</span>
												{:else if poolHealth[pool.id].last_scrub}
													<span class="health-item scrub-info">
														<Icon icon="mdi:check-circle-outline" class="w-3.5 h-3.5" />
														{$t.storageManager.health?.lastScrub || 'Last scrub'}: {formatRelativeDate(poolHealth[pool.id].last_scrub?.date || '')}
														{#if poolHealth[pool.id].last_scrub?.errors_found && poolHealth[pool.id].last_scrub.errors_found > 0}
															<span class="scrub-errors">({poolHealth[pool.id].last_scrub?.errors_found} {$t.storageManager.health?.errorsFound || 'errors'})</span>
														{/if}
													</span>
												{/if}
												{#if poolHealth[pool.id].devices.some(d => d.errors.read_errors > 0 || d.errors.write_errors > 0 || d.errors.corruption_errors > 0)}
													<span class="health-item errors">
														<Icon icon="mdi:alert-circle" class="w-3.5 h-3.5" />
														{$t.storageManager.health?.deviceErrors || 'Device errors detected'}
													</span>
												{/if}
											</div>
										{/if}
									</div>
									<button class="menu-btn menu-trigger" onclick={(e) => openPoolMenu(e, pool)}>
										<Icon icon="mdi:dots-horizontal" class="w-5 h-5" />
									</button>
								</div>

								<!-- Volumes (expanded) -->
								{#if expandedPools.has(pool.id)}
									<div class="volumes-container">
										{#each pool.volumes as volume}
											<div class="volume-item">
												<div class="volume-icon">
													<Icon icon="mdi:folder-outline" class="w-6 h-6 text-slate-400" />
												</div>
												<div class="volume-info">
													<div class="volume-title">
														<span class="volume-name">{volume.name}</span>
													</div>
													<div class="volume-meta">
														<span class="status-badge {getStatusColor(volume.status)}">
															{getStatusLabel(volume.status)}
														</span>
														<span class="meta-item">{volume.fs_type}</span>
														<span class="meta-item">{formatBytes(volume.size)}</span>
														{#if volume.mount_options}
															<span class="meta-item mount-opts">
																<Icon icon="mdi:cog" class="w-3 h-3" style="display:inline;vertical-align:middle" />
																{volume.mount_options}
															</span>
														{/if}
														{#if checkingVolumes.has(volume.id)}
															<span class="meta-item checking">
																<Icon icon="mdi:shield-check" class="w-3.5 h-3.5 animate-spin" style="display:inline;vertical-align:middle" />
																{$t.storageManager.modals2?.fsck?.checkRunning || 'Checking...'}
															</span>
														{/if}
													</div>
												</div>
												<div class="volume-usage">
													<span class="usage-text">{$t.storageManager.volumes.used}: {formatBytes(volume.used)}</span>
													<div class="usage-bar">
														<div class="usage-fill" style="width: {volume.usage_percent}%"></div>
													</div>
												</div>
												<button class="menu-btn menu-trigger" onclick={(e) => openVolumeMenu(e, volume)}>
													<Icon icon="mdi:dots-horizontal" class="w-5 h-5" />
												</button>
											</div>
										{/each}
										{#if pool.volumes.length === 0}
											<div class="no-volumes">
												<span>{$t.storageManager.pools.noVolumes}</span>
												<button class="btn-text" onclick={() => openCreateVolume(pool.id)}>{$t.storageManager.pools.createVolumeLink}</button>
											</div>
										{/if}
									</div>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			{:else if activeTab === 'organizing'}
				<div class="coming-soon">
					<Icon icon="mdi:folder-sync" class="w-16 h-16 text-slate-300" />
					<p>{$t.storageManager.messages.dataOrganizingComingSoon}</p>
				</div>
			{:else if activeTab === 'settings'}
				<div class="coming-soon">
					<Icon icon="mdi:cog" class="w-16 h-16 text-slate-300" />
					<p>{$t.storageManager.messages.advancedSettingsComingSoon}</p>
				</div>
			{/if}

		{:else if sidebarSection === 'harddisk'}
			{#if loading}
				<div class="loading-state">
					<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
				</div>
			{:else}
				<!-- System Disk -->
				{#if disks.some(d => d.is_system)}
					<h2 class="section-title">
						<Icon icon="mdi:lock" class="w-4 h-4" style="display:inline;vertical-align:middle;margin-right:4px;opacity:0.5" />
						{$t.storageManager.disks.systemDisk}
					</h2>
					<div class="disk-list">
						{#each disks.filter(d => d.is_system) as disk}
							<div class="disk-card system">
								<div class="disk-icon-large">
									<Icon icon={getDiskTypeIcon(disk.disk_type)} class="w-10 h-10" />
								</div>
								<div class="disk-details">
									<div class="disk-header">
										<span class="disk-model">{disk.model}</span>
										<span class="system-badge">{$t.storageManager.disks.system}</span>
									</div>
									<div class="disk-meta">
										<span>{disk.device_path}</span>
										<span>{formatBytes(disk.size)}</span>
										{#if disk.serial}
											<span>SN: {disk.serial}</span>
										{/if}
									</div>
									<div class="disk-health">
										{#if disk.health_status}
											<span class="health-badge {disk.health_status === 'PASSED' ? 'healthy' : 'warning'}">
												{disk.health_status}
											</span>
										{/if}
										{#if disk.temperature}
											<span class="temp {disk.temperature > 50 ? 'hot' : disk.temperature > 40 ? 'warm' : 'cool'}">
												{disk.temperature}°C
											</span>
										{/if}
									</div>
								</div>
								<div class="disk-actions">
									<button class="btn-secondary" onclick={() => showDiskDetails(disk)}>
										<Icon icon="mdi:information-outline" class="w-4 h-4" />
										{$t.storageManager.disks.details}
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Storage Disks -->
				<h2 class="section-title" style="margin-top: {disks.some(d => d.is_system) ? '24px' : '0'}">
					{$t.storageManager.disks.title}
				</h2>
				{#if disks.filter(d => !d.is_system).length === 0}
					<div class="empty-state">
						<Icon icon="mdi:harddisk-remove" class="w-12 h-12 text-slate-300" />
						<p>{$t.storageManager.disks.noExternalDevices}</p>
					</div>
				{:else}
					<div class="disk-list">
						{#each disks.filter(d => !d.is_system) as disk}
							<div class="disk-card">
								<div class="disk-icon-large">
									<Icon icon={getDiskTypeIcon(disk.disk_type)} class="w-10 h-10" />
								</div>
								<div class="disk-details">
									<div class="disk-header">
										<span class="disk-model">{disk.model}</span>
									</div>
									<div class="disk-meta">
										<span>{disk.device_path}</span>
										<span>{formatBytes(disk.size)}</span>
										{#if disk.serial}
											<span>SN: {disk.serial}</span>
										{/if}
									</div>
									<div class="disk-health">
										{#if disk.health_status}
											<span class="health-badge {disk.health_status === 'PASSED' ? 'healthy' : 'warning'}">
												{disk.health_status}
											</span>
										{/if}
										{#if disk.temperature}
											<span class="temp {disk.temperature > 50 ? 'hot' : disk.temperature > 40 ? 'warm' : 'cool'}">
												{disk.temperature}°C
											</span>
										{/if}
									</div>
								</div>
								<div class="disk-actions">
									<button class="btn-secondary" onclick={() => showDiskDetails(disk)}>
										<Icon icon="mdi:information-outline" class="w-4 h-4" />
										{$t.storageManager.disks.details}
									</button>
									<button class="btn-secondary" onclick={() => showDiskSmart(disk)}>
										<Icon icon="mdi:chart-line" class="w-4 h-4" />
										{$t.storageManager.disks.smart}
									</button>
									{#if disk.disk_type === 'hdd' || disk.disk_type === 'ssd'}
										<button class="btn-secondary" onclick={() => openPowerSettings(disk)}>
											<Icon icon="mdi:lightning-bolt" class="w-4 h-4" />
											{$t.storageManager?.power?.title || 'Power'}
										</button>
									{/if}
									<button class="btn-secondary btn-warning" onclick={() => confirmWipeDiskWithReset(disk)}>
										<Icon icon="mdi:eraser" class="w-4 h-4" />
										{$t.storageManager.disks.wipe}
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			{/if}

		{:else if sidebarSection === 'overview'}
			<h2 class="section-title">{$t.storageManager.overview.title}</h2>
			<div class="overview-grid">
				<div class="overview-card">
					<Icon icon="mdi:harddisk" class="w-8 h-8 text-blue-500" />
					<div class="overview-stat">
						<span class="stat-value">{disks.length}</span>
						<span class="stat-label">{$t.storageManager.overview.disks}</span>
					</div>
				</div>
				<div class="overview-card">
					<Icon icon="mdi:database" class="w-8 h-8 text-green-500" />
					<div class="overview-stat">
						<span class="stat-value">{pools.length}</span>
						<span class="stat-label">{$t.storageManager.overview.pools}</span>
					</div>
				</div>
				<div class="overview-card">
					<Icon icon="mdi:folder-multiple" class="w-8 h-8 text-purple-500" />
					<div class="overview-stat">
						<span class="stat-value">{pools.reduce((sum, p) => sum + p.volumes.length, 0)}</span>
						<span class="stat-label">{$t.storageManager.overview.volumes}</span>
					</div>
				</div>
				<div class="overview-card">
					<Icon icon="mdi:database-check" class="w-8 h-8 text-amber-500" />
					<div class="overview-stat">
						<span class="stat-value">{formatBytes(pools.reduce((sum, p) => sum + p.total_size, 0))}</span>
						<span class="stat-label">{$t.storageManager.overview.totalCapacity}</span>
					</div>
				</div>
			</div>

		{:else if sidebarSection === 'external'}
			<h2 class="section-title">{$t.storageManager.sidebar.externalStorage}</h2>
			{#if disks.filter(d => d.is_removable).length === 0}
				<div class="empty-state">
					<Icon icon="mdi:usb-off" class="w-16 h-16 text-slate-300" />
					<p>{$t.storageManager.disks.noExternalDevices}</p>
				</div>
			{:else}
				<div class="disk-list">
					{#each disks.filter(d => d.is_removable) as disk}
						<div class="disk-card">
							<div class="disk-icon-large">
								<Icon icon={getDiskTypeIcon(disk.disk_type)} class="w-10 h-10" />
							</div>
							<div class="disk-details">
								<div class="disk-header">
									<span class="disk-model">{disk.model}</span>
								</div>
								<div class="disk-meta">
									<span>{disk.device_path}</span>
									<span>{formatBytes(disk.size)}</span>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</main>

	<!-- Pool context menu -->
	{#if poolMenuTarget}
		<div class="context-menu" style="left: {poolMenuPosition.x}px; top: {poolMenuPosition.y}px;">
			<button onclick={() => openCreateVolume(poolMenuTarget?.id)}>
				<Icon icon="mdi:folder-plus" class="w-4 h-4" />
				{$t.storageManager.contextMenu.createVolume}
			</button>
			<button onclick={() => poolMenuTarget && openEditPool(poolMenuTarget)}>
				<Icon icon="mdi:pencil" class="w-4 h-4" />
				{$t.storageManager.contextMenu.edit}
			</button>
			<button
				onclick={() => poolMenuTarget && scrubPool(poolMenuTarget)}
				disabled={poolMenuTarget && scrubbingPools.has(poolMenuTarget.id)}
			>
				<Icon icon="mdi:broom" class="w-4 h-4" />
				{scrubbingPools.has(poolMenuTarget?.id || '') ? $t.storageManager.contextMenu.scrubbing : $t.storageManager.contextMenu.scrub}
			</button>
			{#if poolMenuTarget && poolMenuTarget.raid_type !== 'basic' && poolMenuTarget.status === 'normal'}
				<button
					onclick={() => poolMenuTarget && openGrowPool(poolMenuTarget)}
					disabled={growingPools.has(poolMenuTarget?.id || '')}
				>
					<Icon icon="mdi:harddisk-plus" class="w-4 h-4" />
					{growingPools.has(poolMenuTarget?.id || '') ? $t.storageManager?.grow?.expanding || 'Expanding...' : $t.storageManager?.grow?.addDisks || 'Add Disks'}
				</button>
			{/if}
			<div class="menu-divider"></div>
			<button class="danger" onclick={() => poolMenuTarget && confirmDeletePool(poolMenuTarget)}>
				<Icon icon="mdi:delete" class="w-4 h-4" />
				{$t.storageManager.contextMenu.delete}
			</button>
		</div>
	{/if}

	<!-- Volume context menu -->
	{#if volumeMenuTarget}
		<div class="context-menu" style="left: {volumeMenuPosition.x}px; top: {volumeMenuPosition.y}px;">
			<button onclick={() => volumeMenuTarget && toggleVolumeMount(volumeMenuTarget)}>
				<Icon icon={volumeMenuTarget.status === 'mounted' ? 'mdi:eject' : 'mdi:play'} class="w-4 h-4" />
				{volumeMenuTarget.status === 'mounted' ? $t.storageManager.volumes.unmount : $t.storageManager.volumes.mount}
			</button>
			<button onclick={() => volumeMenuTarget && openResizeVolume(volumeMenuTarget)}>
				<Icon icon="mdi:resize" class="w-4 h-4" />
				{$t.storageManager.contextMenu.resize}
			</button>
			<button onclick={() => volumeMenuTarget && openMountOptions(volumeMenuTarget)}>
				<Icon icon="mdi:cog" class="w-4 h-4" />
				{$t.storageManager.contextMenu.editMountOptions}
			</button>
			<button
				onclick={() => volumeMenuTarget && openFsckVolume(volumeMenuTarget)}
				disabled={volumeMenuTarget.status === 'mounted'}
			>
				<Icon icon="mdi:shield-check" class="w-4 h-4" />
				{$t.storageManager.contextMenu.checkFilesystem}
			</button>
			{#if volumeMenuTarget.fs_type === 'btrfs'}
				<button onclick={() => volumeMenuTarget && openSnapshots(volumeMenuTarget)}>
					<Icon icon="mdi:camera" class="w-4 h-4" />
					{$t.storageManager?.snapshots?.title || 'Snapshots'}
				</button>
			{/if}
			<div class="menu-divider"></div>
			<button class="danger" onclick={() => volumeMenuTarget && confirmDeleteVolume(volumeMenuTarget)}>
				<Icon icon="mdi:delete" class="w-4 h-4" />
				{$t.storageManager.contextMenu.delete}
			</button>
		</div>
	{/if}
</div>

<!-- Create Pool Modal -->
{#if showCreatePoolModal}
	<div class="modal-overlay" onclick={() => showCreatePoolModal = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<button class="back-btn" onclick={() => createPoolStep > 1 ? createPoolStep-- : showCreatePoolModal = false}>
					<Icon icon="mdi:chevron-left" class="w-5 h-5" />
				</button>
				<h2>{createPoolStep === 1 ? $t.storageManager.modals.createPool.title : $t.storageManager.modals.createPool.preview}</h2>
				<button class="close-btn" onclick={() => showCreatePoolModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-content">
				{#if createPoolStep === 1}
					<!-- Step 1: Select disks and RAID type -->
					<div class="form-section">
						<label class="form-label">{$t.storageManager.modals.createPool.poolName}</label>
						<input type="text" class="form-input" bind:value={newPool.name} placeholder="Storage Pool 1" />
					</div>

					<div class="form-section">
						<label class="form-label">{$t.storageManager.modals.createPool.selectHardDisk}</label>
						{#if candidates.length === 0}
							<div class="no-candidates">
								<Icon icon="mdi:harddisk-remove" class="w-8 h-8 text-slate-400" />
								<p>{$t.storageManager.modals.createPool.noDisksAvailable}</p>
							</div>
						{:else}
							<div class="disk-selector">
								{#each candidates as candidate}
									<button
										class="disk-option"
										class:selected={newPool.selectedDevices.includes(candidate.device_path)}
										onclick={() => toggleDeviceSelection(candidate.device_path)}
									>
										<Icon icon={getDiskTypeIcon(candidate.disk_type)} class="w-6 h-6" />
										<div class="disk-option-info">
											<span class="disk-option-name">{formatBytes(candidate.size)} - {candidate.model}</span>
											<span class="disk-option-path">{candidate.device_path}</span>
										</div>
										{#if candidate.is_empty}
											<span class="empty-badge">{$t.storageManager.messages.empty}</span>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					</div>

					{#if newPool.selectedDevices.length > 0}
						<div class="form-section">
							<label class="form-label">{$t.storageManager.modals.createPool.selectRaidType}</label>
							<div class="raid-selector">
								{#each getAvailableRaidTypes() as raid}
									<button
										class="raid-option"
										class:selected={newPool.raidType === raid.type}
										onclick={() => newPool.raidType = raid.type}
									>
										<div class="raid-header">
											<span class="raid-name">{getRaidTypeName(raid.nameKey)}</span>
											{#if raid.type === 'basic'}
												<span class="rec-badge">{$t.storageManager.modals.createPool.recommended}</span>
											{/if}
										</div>
										<span class="raid-space">{$t.storageManager.modals.createPool.available}: {formatBytes(calculateAvailableSpace())}</span>
										<p class="raid-desc">{getRaidTypeDesc(raid.descKey)}</p>
									</button>
								{/each}
							</div>
						</div>

						<div class="form-section">
							<label class="checkbox-label">
								<input type="checkbox" bind:checked={newPool.wipeDevices} />
								<span>{$t.storageManager.modals.createPool.wipeDisks}</span>
							</label>
						</div>
					{/if}

				{:else if createPoolStep === 2}
					<!-- Step 2: Preview -->
					<div class="preview-card">
						<div class="preview-header">
							<Icon icon={getRaidIcon(newPool.raidType)} class="w-10 h-10 text-slate-600" />
							<h3>{newPool.name}</h3>
						</div>
						<div class="preview-details">
							<div class="preview-row">
								<span>{$t.storageManager.modals.createPool.total}</span>
								<span>{formatBytes(calculateAvailableSpace())}</span>
							</div>
							<div class="preview-row">
								<span>{$t.storageManager.modals.createPool.raidType}</span>
								<span>{getRaidTypeName(raidTypes.find(r => r.type === newPool.raidType)?.nameKey || '')}</span>
							</div>
							<div class="preview-row">
								<span>{$t.storageManager.modals.createPool.useHardDisk}</span>
								<span>{newPool.selectedDevices.length} {newPool.selectedDevices.length > 1 ? $t.storageManager.pools.disks : $t.storageManager.pools.disk}</span>
							</div>
						</div>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showCreatePoolModal = false}>{$t.common.cancel}</button>
				{#if createPoolStep === 1}
					<button
						class="btn-primary"
						disabled={newPool.selectedDevices.length === 0 || !newPool.name}
						onclick={() => createPoolStep = 2}
					>
						{$t.common.next}
					</button>
				{:else}
					<button class="btn-primary" onclick={createPool}>{$t.common.create}</button>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Create Volume Modal -->
{#if showCreateVolumeModal}
	<div class="modal-overlay" onclick={() => showCreateVolumeModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.createVolume.title}</h2>
				<button class="close-btn" onclick={() => showCreateVolumeModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-content">
				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals.createVolume.storagePool}</label>
					<select class="form-input" bind:value={newVolume.poolId}>
						<option value="">{$t.storageManager.modals.createVolume.selectPool}</option>
						{#each pools as pool}
							<option value={pool.id}>{pool.name}</option>
						{/each}
					</select>
				</div>

				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals.createVolume.volumeName}</label>
					<input type="text" class="form-input" bind:value={newVolume.name} placeholder="Volume 1" />
				</div>

				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals.createVolume.fileSystem}</label>
					<select class="form-input" bind:value={newVolume.fsType}>
						<option value="ext4">ext4</option>
						<option value="btrfs">Btrfs</option>
						<option value="xfs">XFS</option>
						<option value="f2fs">F2FS</option>
					</select>
				</div>

				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals2?.mountOptions?.mountOptions || 'Mount Options'}</label>
					<div class="mount-presets-inline">
						<button class="preset-chip" class:selected={createVolumeMountPreset === 'default'} onclick={() => createVolumeMountPreset = 'default'}>
							{$t.storageManager.modals2?.mountOptions?.presetDefault || 'Default'}
						</button>
						<button class="preset-chip" class:selected={createVolumeMountPreset === 'ssd'} onclick={() => createVolumeMountPreset = 'ssd'}>
							SSD
						</button>
						<button class="preset-chip" class:selected={createVolumeMountPreset === 'nas'} onclick={() => createVolumeMountPreset = 'nas'}>
							NAS
						</button>
						<button class="preset-chip" class:selected={createVolumeMountPreset === 'custom'} onclick={() => createVolumeMountPreset = 'custom'}>
							{$t.storageManager.modals2?.mountOptions?.customOptions || 'Custom'}
						</button>
					</div>
					{#if createVolumeMountPreset === 'custom'}
						<input
							type="text"
							class="form-input"
							style="margin-top: 8px;"
							bind:value={createVolumeMountCustom}
							placeholder={$t.storageManager.modals2?.mountOptions?.customPlaceholder || 'e.g. noatime,discard'}
						/>
					{/if}
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showCreateVolumeModal = false}>{$t.common.cancel}</button>
				<button
					class="btn-primary"
					disabled={!newVolume.poolId || !newVolume.name}
					onclick={createVolume}
				>
					{$t.common.create}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Pool Confirmation Modal -->
{#if showDeletePoolModal && poolToDelete}
	<div class="modal-overlay" onclick={() => showDeletePoolModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.deletePool.title}</h2>
				<button class="close-btn" onclick={() => showDeletePoolModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<p class="warning-text">
					{$t.storageManager.modals.deletePool.confirmMessage} <strong>{poolToDelete.name}</strong>?
					{#if poolToDelete.volumes.length > 0}
						{$t.storageManager.modals.deletePool.willDeleteVolumes} {poolToDelete.volumes.length} {poolToDelete.volumes.length > 1 ? $t.storageManager.modals.deletePool.volumes : $t.storageManager.modals.deletePool.volume}.
					{/if}
					{$t.storageManager.modals.deletePool.cannotBeUndone}
				</p>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showDeletePoolModal = false}>{$t.common.cancel}</button>
				<button class="btn-danger" onclick={deletePool}>{$t.common.delete}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Volume Confirmation Modal -->
{#if showDeleteVolumeModal && volumeToDelete}
	<div class="modal-overlay" onclick={() => showDeleteVolumeModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.deleteVolume.title}</h2>
				<button class="close-btn" onclick={() => showDeleteVolumeModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<p class="warning-text">
					{$t.storageManager.modals.deleteVolume.confirmMessage} <strong>{volumeToDelete.name}</strong>?
					{$t.storageManager.modals.deleteVolume.dataLost} {$t.storageManager.modals.deleteVolume.cannotBeUndone}
				</p>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showDeleteVolumeModal = false}>{$t.common.cancel}</button>
				<button class="btn-danger" onclick={deleteVolume}>{$t.common.delete}</button>
			</div>
		</div>
	</div>
{/if}

<!-- SMART Info Modal -->
{#if showSmartModal}
	<div class="modal-overlay" onclick={() => showSmartModal = false}>
		<div class="modal modal-lg" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.smartInfo.title}</h2>
				<button class="close-btn" onclick={() => showSmartModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<!-- Tabs -->
			<div class="modal-tabs">
				<button class:active={smartTab === 'info'} onclick={() => smartTab = 'info'}>
					<Icon icon="mdi:information" class="w-4 h-4" />
					{$t.storageManager?.smartTests?.tabInfo || 'Info'}
				</button>
				<button class:active={smartTab === 'tests'} onclick={() => { smartTab = 'tests'; if (selectedDiskForSmart) loadSmartHistory(selectedDiskForSmart.device_name); }}>
					<Icon icon="mdi:play-circle" class="w-4 h-4" />
					{$t.storageManager?.smartTests?.tabTests || 'Self-Tests'}
				</button>
				<button class:active={smartTab === 'schedules'} onclick={() => { smartTab = 'schedules'; loadSmartSchedules(); }}>
					<Icon icon="mdi:calendar-clock" class="w-4 h-4" />
					{$t.storageManager?.smartTests?.tabSchedules || 'Schedules'}
				</button>
			</div>
			<div class="modal-content">
				{#if smartLoading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
						<p>{$t.storageManager.modals.smartInfo.loadingData}</p>
					</div>
				{:else if smartTab === 'info'}
					{#if smartInfo}
						<div class="smart-header">
							<div class="smart-device">
								<span class="smart-model">{smartInfo.model}</span>
								<span class="smart-path">{smartInfo.device_path}</span>
							</div>
							<span class="health-badge {smartInfo.health_status === 'PASSED' ? 'healthy' : 'warning'}">
								{smartInfo.health_status}
							</span>
						</div>
						<div class="smart-stats">
							<div class="smart-stat">
								<span class="stat-label">{$t.storageManager.modals.smartInfo.temperature}</span>
								<span class="stat-value">{smartInfo.temperature ?? 'N/A'}°C</span>
							</div>
							<div class="smart-stat">
								<span class="stat-label">{$t.storageManager.modals.smartInfo.powerOnHours}</span>
								<span class="stat-value">{smartInfo.power_on_hours?.toLocaleString() ?? 'N/A'}</span>
							</div>
							<div class="smart-stat">
								<span class="stat-label">{$t.storageManager.modals.smartInfo.powerCycles}</span>
								<span class="stat-value">{smartInfo.power_cycle_count?.toLocaleString() ?? 'N/A'}</span>
							</div>
							<div class="smart-stat">
								<span class="stat-label">{$t.storageManager.modals.smartInfo.serial}</span>
								<span class="stat-value">{smartInfo.serial ?? 'N/A'}</span>
							</div>
						</div>
						{#if smartInfo.attributes.length > 0}
							<h3 class="attributes-title">{$t.storageManager.modals.smartInfo.attributes}</h3>
							<div class="smart-attributes">
								{#each smartInfo.attributes as attr}
									<div class="smart-attr">
										<span class="attr-name">{attr.name}</span>
										<span class="attr-value">{attr.value}</span>
									</div>
								{/each}
							</div>
						{/if}
					{:else}
						<div class="error-state">
							<Icon icon="mdi:alert-circle" class="w-8 h-8 text-red-500" />
							<p>{$t.storageManager.modals.smartInfo.loadFailed}</p>
						</div>
					{/if}
				{:else if smartTab === 'tests'}
					<!-- Self-Test Controls -->
					<div class="smart-test-controls">
						<h3>{$t.storageManager?.smartTests?.runTest || 'Run Test'}</h3>
						<div class="test-buttons">
							<button class="btn-primary" onclick={() => runSmartTest('short')} disabled={runningSmartTest}>
								<Icon icon="mdi:flash" class="w-4 h-4" />
								{$t.storageManager?.smartTests?.shortTest || 'Short Test'}
							</button>
							<button class="btn-secondary" onclick={() => runSmartTest('long')} disabled={runningSmartTest}>
								<Icon icon="mdi:clock-outline" class="w-4 h-4" />
								{$t.storageManager?.smartTests?.longTest || 'Long Test'}
							</button>
						</div>
						{#if runningSmartTest && smartTestTaskId}
							{@const progress = $taskProgress[smartTestTaskId]}
							{#if progress}
								<div class="test-progress">
									<div class="progress-bar">
										<div class="progress-fill" style="width: {progress.progress_percent}%"></div>
									</div>
									<span class="progress-text">{progress.current_step || `${progress.progress_percent}%`}</span>
								</div>
							{/if}
						{/if}
					</div>
					<!-- Test History -->
					<h3 class="mt-4">{$t.storageManager?.smartTests?.history || 'Test History'}</h3>
					{#if smartTestHistory.length > 0}
						<div class="smart-history-table">
							<div class="table-header">
								<span>#</span>
								<span>{$t.storageManager?.smartTests?.testType || 'Type'}</span>
								<span>{$t.common.status}</span>
								<span>{$t.storageManager?.smartTests?.lifetime || 'Hours'}</span>
							</div>
							{#each smartTestHistory as entry}
								<div class="table-row">
									<span>{entry.num}</span>
									<span>{entry.test_type}</span>
									<span class="status-cell {entry.status.includes('without error') ? 'text-green' : 'text-red'}">{entry.status}</span>
									<span>{entry.lifetime_hours.toLocaleString()}</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="empty-text">{$t.storageManager?.smartTests?.noHistory || 'No test history available'}</p>
					{/if}
				{:else if smartTab === 'schedules'}
					<div class="schedule-header">
						<h3>{$t.storageManager?.smartTests?.scheduleTitle || 'Scheduled Tests'}</h3>
						<button class="btn-primary btn-sm" onclick={() => {
							newScheduleDevice = selectedDiskForSmart?.device_path || '';
							showCreateScheduleModal = true;
						}}>
							<Icon icon="mdi:plus" class="w-4 h-4" />
							{$t.storageManager?.smartTests?.addSchedule || 'Add Schedule'}
						</button>
					</div>
					{#if smartSchedules.filter(s => s.device_name === selectedDiskForSmart?.device_name).length > 0}
						{#each smartSchedules.filter(s => s.device_name === selectedDiskForSmart?.device_name) as schedule}
							<div class="schedule-item">
								<div class="schedule-info">
									<span class="schedule-type">{schedule.test_type}</span>
									<span class="schedule-interval">
										{schedule.interval_hours === 24 ? ($t.storageManager?.smartTests?.daily || 'Daily') :
										 schedule.interval_hours === 168 ? ($t.storageManager?.smartTests?.weekly || 'Weekly') :
										 schedule.interval_hours === 720 ? ($t.storageManager?.smartTests?.monthly || 'Monthly') :
										 `${schedule.interval_hours}h`}
									</span>
									{#if schedule.last_run}
										<span class="schedule-last">{$t.storageManager?.smartTests?.lastRun || 'Last'}: {new Date(schedule.last_run).toLocaleDateString()}</span>
									{/if}
								</div>
								<div class="schedule-actions">
									<button class="toggle-btn" class:active={schedule.enabled} onclick={() => toggleSchedule(schedule.id, !schedule.enabled)}>
										{schedule.enabled ? ($t.common.enabled) : ($t.common.disabled)}
									</button>
									<button class="btn-icon danger" onclick={() => deleteSmartSchedule(schedule.id)}>
										<Icon icon="mdi:delete" class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					{:else}
						<p class="empty-text">{$t.storageManager?.smartTests?.noSchedules || 'No schedules configured'}</p>
					{/if}
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showSmartModal = false}>{$t.common.close}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Create SMART Schedule Modal -->
{#if showCreateScheduleModal}
	<div class="modal-overlay" onclick={() => showCreateScheduleModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager?.smartTests?.addSchedule || 'Add Schedule'}</h2>
				<button class="close-btn" onclick={() => showCreateScheduleModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="form-group">
					<label>{$t.storageManager?.smartTests?.testType || 'Test Type'}</label>
					<select bind:value={newScheduleTestType}>
						<option value="short">{$t.storageManager?.smartTests?.shortTest || 'Short'}</option>
						<option value="long">{$t.storageManager?.smartTests?.longTest || 'Long'}</option>
					</select>
				</div>
				<div class="form-group">
					<label>{$t.storageManager?.smartTests?.interval || 'Interval'}</label>
					<select bind:value={newScheduleInterval}>
						<option value={24}>{$t.storageManager?.smartTests?.daily || 'Daily'}</option>
						<option value={168}>{$t.storageManager?.smartTests?.weekly || 'Weekly'}</option>
						<option value={720}>{$t.storageManager?.smartTests?.monthly || 'Monthly'}</option>
					</select>
				</div>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showCreateScheduleModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary" onclick={createSmartSchedule}>{$t.common.create}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Power Settings Modal -->
{#if showPowerModal && powerDisk}
	<div class="modal-overlay" onclick={() => showPowerModal = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager?.power?.title || 'Power Settings'} - {powerDisk.model}</h2>
				<button class="close-btn" onclick={() => showPowerModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				{#if powerLoading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
					</div>
				{:else if powerSettings}
					<div class="form-group">
						<label>{$t.storageManager?.power?.apmLevel || 'APM Level'}</label>
						<div class="power-presets">
							<button class:active={apmPreset === 'low'} onclick={() => apmPreset = 'low'}>
								<Icon icon="mdi:leaf" class="w-4 h-4" />
								{$t.storageManager?.power?.lowPower || 'Low Power'}
							</button>
							<button class:active={apmPreset === 'balanced'} onclick={() => apmPreset = 'balanced'}>
								<Icon icon="mdi:scale-balance" class="w-4 h-4" />
								{$t.storageManager?.power?.balanced || 'Balanced'}
							</button>
							<button class:active={apmPreset === 'max'} onclick={() => apmPreset = 'max'}>
								<Icon icon="mdi:rocket-launch" class="w-4 h-4" />
								{$t.storageManager?.power?.maxPerformance || 'Performance'}
							</button>
							<button class:active={apmPreset === 'disabled'} onclick={() => apmPreset = 'disabled'}>
								<Icon icon="mdi:close-circle" class="w-4 h-4" />
								{$t.common.disabled}
							</button>
						</div>
					</div>
					<div class="form-group">
						<label>{$t.storageManager?.power?.spindown || 'Spindown Timer'}</label>
						<select bind:value={powerSettings.spindown_minutes}>
							<option value={0}>{$t.common.disabled}</option>
							<option value={5}>5 min</option>
							<option value={10}>10 min</option>
							<option value={20}>20 min</option>
							<option value={30}>30 min</option>
							<option value={60}>60 min</option>
							<option value={120}>120 min</option>
						</select>
					</div>
					<div class="form-group">
						<label class="checkbox-label">
							<input type="checkbox" bind:checked={powerSettings.write_cache} />
							{$t.storageManager?.power?.writeCache || 'Write Cache'}
							<span class="help-text">{$t.storageManager?.power?.writeCacheWarning || 'Disabling write cache reduces performance but improves data safety'}</span>
						</label>
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showPowerModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary" onclick={savePowerSettings}>{$t.common.save}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Snapshots Modal -->
{#if showSnapshotsModal && snapshotVolume}
	<div class="modal-overlay" onclick={() => showSnapshotsModal = false}>
		<div class="modal modal-lg" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager?.snapshots?.title || 'Snapshots'} - {snapshotVolume.name}</h2>
				<button class="close-btn" onclick={() => showSnapshotsModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<!-- Create snapshot -->
				<div class="snapshot-create">
					<input type="text" bind:value={newSnapshotName} placeholder={$t.storageManager?.snapshots?.namePlaceholder || 'Snapshot name'} />
					<button class="btn-primary" onclick={createSnapshot} disabled={!newSnapshotName}>
						<Icon icon="mdi:camera" class="w-4 h-4" />
						{$t.storageManager?.snapshots?.create || 'Create Snapshot'}
					</button>
				</div>
				<!-- Snapshot list -->
				{#if snapshotLoading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
					</div>
				{:else if snapshots.length > 0}
					<div class="snapshot-list">
						{#each snapshots as snap}
							<div class="snapshot-item">
								<div class="snapshot-info">
									<Icon icon="mdi:camera" class="w-4 h-4 text-blue-400" />
									<span class="snapshot-name">{snap.name}</span>
									<span class="snapshot-type badge">{snap.snapshot_type}</span>
									<span class="snapshot-date">{new Date(snap.created_at).toLocaleString()}</span>
									{#if snap.size_bytes}
										<span class="snapshot-size">{formatBytes(snap.size_bytes)}</span>
									{/if}
								</div>
								<button class="btn-icon danger" onclick={() => deleteSnapshot(snap.id)}>
									<Icon icon="mdi:delete" class="w-4 h-4" />
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<p class="empty-text">{$t.storageManager?.snapshots?.noSnapshots || 'No snapshots yet'}</p>
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showSnapshotsModal = false}>{$t.common.close}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Grow Pool Modal -->
{#if showGrowPoolModal && growPool_}
	<div class="modal-overlay" onclick={() => showGrowPoolModal = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager?.grow?.addDisks || 'Add Disks'} - {growPool_.name}</h2>
				<button class="close-btn" onclick={() => showGrowPoolModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="grow-info">
					<p class="pool-info-text">
						<Icon icon="mdi:database" class="w-4 h-4" />
						{growPool_.raid_type.toUpperCase()} - {growPool_.devices.length} {$t.storageManager?.grow?.currentDisks || 'disk(s)'}
						- {formatBytes(growPool_.total_size)}
					</p>
				</div>

				{#if growPool_.raid_type === 'raid10' || growPool_.raid_type === 'btrfs-raid10'}
					<div class="warning-box">
						<Icon icon="mdi:alert" class="w-4 h-4" />
						{$t.storageManager?.grow?.raidConstraint || 'RAID 10 requires an even number of new disks'}
					</div>
				{/if}

				<h3>{$t.storageManager?.grow?.selectNewDisks || 'Select new disks'}</h3>
				{#if candidates.length > 0}
					<div class="disk-candidates">
						{#each candidates.filter(c => !growPool_?.devices.includes(c.device_path)) as candidate}
							<label class="disk-candidate-item">
								<input
									type="checkbox"
									value={candidate.device_path}
									bind:group={growSelectedDevices}
								/>
								<Icon icon={getDiskTypeIcon(candidate.disk_type)} class="w-5 h-5" />
								<span class="candidate-model">{candidate.model}</span>
								<span class="candidate-size">{formatBytes(candidate.size)}</span>
							</label>
						{/each}
					</div>
				{:else}
					<p class="empty-text">{$t.storageManager?.grow?.noCandidates || 'No available disks'}</p>
				{/if}

				<label class="checkbox-label mt-3">
					<input type="checkbox" bind:checked={growWipeDevices} />
					{$t.storageManager?.grow?.wipeDevices || 'Wipe new disks before adding'}
				</label>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showGrowPoolModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary" onclick={startGrowPool} disabled={growSelectedDevices.length === 0}>
					<Icon icon="mdi:harddisk-plus" class="w-4 h-4" />
					{$t.storageManager?.grow?.startExpand || 'Start Expansion'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Edit Pool Modal -->
{#if showEditPoolModal && poolToEdit}
	<div class="modal-overlay" onclick={() => showEditPoolModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.editPool.title}</h2>
				<button class="close-btn" onclick={() => showEditPoolModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals.editPool.poolName}</label>
					<input type="text" class="form-input" bind:value={editPoolData.name} />
				</div>
				<div class="form-section">
					<label class="form-label">{$t.storageManager.modals.editPool.description}</label>
					<textarea class="form-input form-textarea" bind:value={editPoolData.description} rows="3"></textarea>
				</div>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showEditPoolModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary" onclick={savePoolEdit} disabled={!editPoolData.name}>{$t.common.save}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Wipe Disk Confirmation Modal -->
{#if showWipeDiskModal && diskToWipe}
	<div class="modal-overlay" onclick={() => showWipeDiskModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.wipeDisk.title}</h2>
				<button class="close-btn" onclick={() => showWipeDiskModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="warning-banner">
					<Icon icon="mdi:alert" class="w-6 h-6" />
					<span>{$t.storageManager.modals.wipeDisk.warning}</span>
				</div>
				<p class="warning-text">
					{$t.storageManager.modals.wipeDisk.confirmMessage} <strong>{diskToWipe.model}</strong> ({diskToWipe.device_path})?
					{$t.storageManager.modals.wipeDisk.allDataLost} {$t.storageManager.modals.wipeDisk.cannotBeUndone}
				</p>

				<div class="form-section" style="margin-top: 16px;">
					<label class="form-label">{$t.storageManager.modals2?.wipeDisk?.wipeMode || 'Wipe Mode'}</label>
					<div class="preset-selector">
						<button
							class="preset-option"
							class:selected={wipeMode === 'quick'}
							onclick={() => wipeMode = 'quick'}
						>
							<div class="preset-name">{$t.storageManager.modals2?.wipeDisk?.wipeQuick || 'Quick'}</div>
							<div class="preset-desc">{$t.storageManager.modals2?.wipeDisk?.wipeQuickDesc || 'Erase partition table only (instant)'}</div>
						</button>
						<button
							class="preset-option"
							class:selected={wipeMode === 'zeros'}
							onclick={() => wipeMode = 'zeros'}
						>
							<div class="preset-name">{$t.storageManager.modals2?.wipeDisk?.wipeZeros || 'Zero Fill'}</div>
							<div class="preset-desc">{$t.storageManager.modals2?.wipeDisk?.wipeZerosDesc || 'Write zeros to entire disk (1 pass)'}</div>
						</button>
						<button
							class="preset-option"
							class:selected={wipeMode === 'secure'}
							onclick={() => wipeMode = 'secure'}
						>
							<div class="preset-name">{$t.storageManager.modals2?.wipeDisk?.wipeSecure || 'Secure Erase'}</div>
							<div class="preset-desc">{$t.storageManager.modals2?.wipeDisk?.wipeSecureDesc || 'Overwrite 3 times + zeros (very slow)'}</div>
						</button>
					</div>
					{#if wipeMode !== 'quick'}
						<p class="help-text" style="margin-top: 8px; font-size: 12px; color: #64748b;">
							{$t.storageManager.modals2?.wipeDisk?.wipeDuration || 'Duration depends on disk size and mode.'}
						</p>
					{/if}
				</div>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showWipeDiskModal = false} disabled={wipingDisk}>{$t.common.cancel}</button>
				<button class="btn-danger" onclick={wipeDisk} disabled={wipingDisk}>
					{#if wipingDisk}
						<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
						{$t.storageManager.modals.wipeDisk.wiping}
					{:else}
						{$t.storageManager.disks.wipe}
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Disk Details Modal (Partitions) -->
{#if showDiskDetailsModal && selectedDiskDetails}
	<div class="modal-overlay" onclick={() => showDiskDetailsModal = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals.diskDetails.title}</h2>
				<button class="close-btn" onclick={() => showDiskDetailsModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="disk-info-header">
					<div class="disk-icon-large">
						<Icon icon={getDiskTypeIcon(selectedDiskDetails.disk_type)} class="w-10 h-10" />
					</div>
					<div class="disk-info-content">
						<span class="disk-model">{selectedDiskDetails.model}</span>
						<span class="disk-path">{selectedDiskDetails.device_path}</span>
					</div>
				</div>

				<div class="disk-specs">
					<div class="spec-row">
						<span class="spec-label">{$t.storageManager.modals.diskDetails.totalSize}</span>
						<span class="spec-value">{formatBytes(selectedDiskDetails.size)}</span>
					</div>
					<div class="spec-row">
						<span class="spec-label">{$t.storageManager.modals.diskDetails.type}</span>
						<span class="spec-value">{selectedDiskDetails.disk_type.toUpperCase()}</span>
					</div>
					{#if selectedDiskDetails.serial}
						<div class="spec-row">
							<span class="spec-label">{$t.storageManager.modals.diskDetails.serial}</span>
							<span class="spec-value">{selectedDiskDetails.serial}</span>
						</div>
					{/if}
					{#if selectedDiskDetails.device_by_id}
						<div class="spec-row">
							<span class="spec-label">{$t.storageManager.modals.diskDetails.deviceId}</span>
							<span class="spec-value spec-value-small">{selectedDiskDetails.device_by_id}</span>
						</div>
					{/if}
				</div>

				<h3 class="partitions-title">{$t.storageManager.modals.diskDetails.partitions}</h3>
				{#if selectedDiskDetails.partitions.length === 0}
					<div class="no-partitions">
						<Icon icon="mdi:harddisk" class="w-8 h-8 text-slate-300" />
						<p>{$t.storageManager.modals.diskDetails.noPartitions}</p>
					</div>
				{:else}
					<div class="partitions-list">
						{#each selectedDiskDetails.partitions as partition}
							<div class="partition-item">
								<div class="partition-header">
									<span class="partition-name">{partition.device_path}</span>
									{#if partition.is_system}
										<span class="system-badge">{$t.storageManager.disks.system}</span>
									{/if}
								</div>
								<div class="partition-details">
									<span class="partition-size">{formatBytes(partition.size)}</span>
									{#if partition.fs_type}
										<span class="partition-fs">{partition.fs_type}</span>
									{/if}
									{#if partition.label}
										<span class="partition-label">{partition.label}</span>
									{/if}
								</div>
								{#if partition.mount_point}
									<div class="partition-mount">
										<Icon icon="mdi:folder-open" class="w-4 h-4" />
										<span>{partition.mount_point}</span>
									</div>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showDiskDetailsModal = false}>{$t.common.close}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Resize Volume Modal -->
{#if showResizeModal && volumeToResize}
	<div class="modal-overlay" onclick={() => showResizeModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals2?.resize?.title || 'Resize Volume'}</h2>
				<button class="close-btn" onclick={() => showResizeModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="form-section">
					<div class="info-row">
						<span class="info-label">{$t.storageManager.modals2?.resize?.currentSize || 'Current Size'}</span>
						<span class="info-value">{formatBytes(volumeToResize.size)}</span>
					</div>
					<div class="info-row">
						<span class="info-label">{$t.storageManager.modals2?.resize?.usedSpace || 'Used Space'}</span>
						<span class="info-value">{formatBytes(volumeToResize.used)} ({volumeToResize.usage_percent}%)</span>
					</div>
				</div>

				{#if volumeToResize.fs_type === 'f2fs'}
					<div class="warning-banner">
						<Icon icon="mdi:alert" class="w-5 h-5" />
						<span>{$t.storageManager.modals2?.resize?.f2fsNotSupported || 'F2FS does not support online resize.'}</span>
					</div>
				{:else}
					<div class="form-section">
						<label class="checkbox-label">
							<input type="checkbox" bind:checked={resizeData.useMax} />
							<span>{$t.storageManager.modals2?.resize?.useMaxSpace || 'Use all available space'}</span>
						</label>
					</div>

					{#if !resizeData.useMax}
						<div class="form-section">
							<label class="form-label">{$t.storageManager.modals2?.resize?.newSize || 'New Size'} (GB)</label>
							<input type="number" class="form-input" bind:value={resizeData.newSizeGB} min="1" />
						</div>

						{#if volumeToResize.fs_type === 'ext4' && resizeData.newSizeGB * 1024 * 1024 * 1024 < volumeToResize.size && volumeToResize.status === 'mounted'}
							<div class="warning-banner">
								<Icon icon="mdi:alert" class="w-5 h-5" />
								<span>{$t.storageManager.modals2?.resize?.shrinkWarning || 'Shrinking ext4 requires the volume to be unmounted first.'}</span>
							</div>
						{/if}

						{#if volumeToResize.fs_type === 'xfs' && resizeData.newSizeGB * 1024 * 1024 * 1024 < volumeToResize.size}
							<div class="warning-banner">
								<Icon icon="mdi:alert" class="w-5 h-5" />
								<span>{$t.storageManager.modals2?.resize?.xfsNoShrink || 'XFS only supports growing, not shrinking.'}</span>
							</div>
						{/if}
					{/if}
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showResizeModal = false}>{$t.common.cancel}</button>
				<button
					class="btn-primary"
					onclick={resizeVolume}
					disabled={volumeToResize.fs_type === 'f2fs'}
				>
					{$t.storageManager.contextMenu?.resize || 'Resize'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Mount Options Modal -->
{#if showMountOptionsModal && volumeForMountOptions}
	<div class="modal-overlay" onclick={() => showMountOptionsModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals2?.mountOptions?.title || 'Mount Options'}</h2>
				<button class="close-btn" onclick={() => showMountOptionsModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				<div class="preset-selector">
					<button
						class="preset-option"
						class:selected={mountOptionsPreset === 'default'}
						onclick={() => mountOptionsPreset = 'default'}
					>
						<div class="preset-name">{$t.storageManager.modals2?.mountOptions?.presetDefault || 'Default'}</div>
						<div class="preset-desc">{$t.storageManager.modals2?.mountOptions?.presetDefaultDesc || 'Standard mount options'}</div>
					</button>
					<button
						class="preset-option"
						class:selected={mountOptionsPreset === 'ssd'}
						onclick={() => mountOptionsPreset = 'ssd'}
					>
						<div class="preset-name">{$t.storageManager.modals2?.mountOptions?.presetSSD || 'SSD Optimized'}</div>
						<div class="preset-desc">{$t.storageManager.modals2?.mountOptions?.presetSSDDesc || 'noatime,discard'}</div>
					</button>
					<button
						class="preset-option"
						class:selected={mountOptionsPreset === 'nas'}
						onclick={() => mountOptionsPreset = 'nas'}
					>
						<div class="preset-name">{$t.storageManager.modals2?.mountOptions?.presetNAS || 'NAS Optimized'}</div>
						<div class="preset-desc">{$t.storageManager.modals2?.mountOptions?.presetNASDesc || 'noatime,nofail'}</div>
					</button>
					<button
						class="preset-option"
						class:selected={mountOptionsPreset === 'custom'}
						onclick={() => mountOptionsPreset = 'custom'}
					>
						<div class="preset-name">{$t.storageManager.modals2?.mountOptions?.customOptions || 'Custom'}</div>
						<div class="preset-desc">{$t.storageManager.modals2?.mountOptions?.customOptionsDesc || 'Enter custom mount options'}</div>
					</button>
				</div>

				{#if mountOptionsPreset === 'custom'}
					<div class="form-section" style="margin-top: 16px;">
						<input
							type="text"
							class="form-input"
							bind:value={mountOptionsCustom}
							placeholder={$t.storageManager.modals2?.mountOptions?.customPlaceholder || 'e.g. noatime,discard,nofail'}
						/>
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showMountOptionsModal = false}>{$t.common.cancel}</button>
				<button class="btn-primary" onclick={saveMountOptions}>{$t.common.save}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Filesystem Check Modal -->
{#if showFsckModal && volumeToCheck}
	<div class="modal-overlay" onclick={() => showFsckModal = false}>
		<div class="modal modal-sm" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{$t.storageManager.modals2?.fsck?.title || 'Check Filesystem'}</h2>
				<button class="close-btn" onclick={() => showFsckModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>
			<div class="modal-content">
				{#if volumeToCheck.status === 'mounted'}
					<div class="warning-banner">
						<Icon icon="mdi:alert" class="w-6 h-6" />
						<span>{$t.storageManager.modals2?.fsck?.mustBeUnmounted || 'Volume must be unmounted before checking.'}</span>
					</div>
				{:else}
					<div class="info-row" style="margin-bottom: 16px;">
						<span class="info-label">Volume</span>
						<span class="info-value">{volumeToCheck.name} ({volumeToCheck.fs_type})</span>
					</div>
					<div class="form-section">
						<label class="checkbox-label">
							<input type="checkbox" bind:checked={fsckRepair} />
							<span>{$t.storageManager.modals2?.fsck?.repairErrors || 'Repair errors automatically'}</span>
						</label>
					</div>
				{/if}
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" onclick={() => showFsckModal = false}>{$t.common.cancel}</button>
				<button
					class="btn-primary"
					onclick={checkFilesystem}
					disabled={volumeToCheck.status === 'mounted'}
				>
					<Icon icon="mdi:shield-check" class="w-4 h-4" />
					{$t.storageManager.modals2?.fsck?.title || 'Check'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.storage-manager {
		display: flex;
		height: 100%;
		background: white;
	}

	/* Sidebar */
	.sidebar {
		width: 200px;
		border-right: 1px solid #e2e8f0;
		padding: 12px 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.sidebar-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 14px;
		border-radius: 8px;
		font-size: 14px;
		color: #475569;
		transition: all 0.15s;
		text-align: left;
	}

	.sidebar-item:hover { background: #f1f5f9; }
	.sidebar-item.active { background: #dbeafe; color: #2563eb; }

	/* Main content */
	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px 24px;
	}

	.section-title {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 20px;
	}

	/* Tabs */
	.tabs {
		display: flex;
		gap: 4px;
		margin-bottom: 20px;
		border-bottom: 2px solid #e2e8f0;
	}

	.tab {
		padding: 12px 20px;
		font-size: 14px;
		color: #64748b;
		border-bottom: 2px solid transparent;
		margin-bottom: -2px;
		transition: all 0.15s;
	}

	.tab:hover { color: #334155; }
	.tab.active { color: #2563eb; border-bottom-color: #2563eb; }

	/* Toolbar */
	.toolbar {
		margin-bottom: 20px;
	}

	.create-dropdown {
		position: relative;
		display: inline-block;
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		left: 0;
		margin-top: 4px;
		min-width: 180px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
		padding: 6px;
		z-index: 100;
	}

	.dropdown-menu button {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		border-radius: 6px;
		font-size: 14px;
		color: #334155;
		text-align: left;
		transition: background 0.15s;
	}

	.dropdown-menu button:hover:not(:disabled) { background: #f1f5f9; }
	.dropdown-menu button:disabled { opacity: 0.5; cursor: not-allowed; }

	/* Pool list */
	.pool-list {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.pool-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		overflow: hidden;
	}

	.pool-header {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px 20px;
		cursor: pointer;
	}

	.pool-header:hover { background: #f1f5f9; }

	.expand-btn {
		color: #64748b;
		transition: transform 0.2s;
	}

	.expand-btn.expanded { transform: rotate(180deg); }

	.pool-icon {
		width: 56px;
		height: 56px;
		background: #e2e8f0;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.pool-info { flex: 1; min-width: 0; }

	.pool-title {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}

	.pool-name {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.pool-desc {
		font-size: 13px;
		color: #64748b;
	}

	.pool-meta {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.status-badge {
		padding: 3px 10px;
		border-radius: 12px;
		font-size: 12px;
		font-weight: 500;
	}

	.meta-item {
		font-size: 13px;
		color: #64748b;
	}

	.menu-btn {
		width: 36px;
		height: 36px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s;
	}

	.menu-btn:hover { background: #e2e8f0; color: #334155; }

	/* Volumes */
	.volumes-container {
		border-top: 1px solid #e2e8f0;
		padding: 8px 0;
	}

	.volume-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 20px 12px 88px;
	}

	.volume-item:hover { background: #f1f5f9; }

	.volume-icon {
		width: 40px;
		height: 40px;
		background: #f1f5f9;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.volume-info { flex: 1; min-width: 0; }

	.volume-title { margin-bottom: 4px; }

	.volume-name {
		font-size: 14px;
		font-weight: 500;
		color: #1e293b;
	}

	.volume-meta {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.volume-usage {
		text-align: right;
		min-width: 120px;
	}

	.usage-text {
		font-size: 12px;
		color: #64748b;
		display: block;
		margin-bottom: 4px;
	}

	.usage-bar {
		height: 4px;
		background: #e2e8f0;
		border-radius: 2px;
		overflow: hidden;
	}

	.usage-fill {
		height: 100%;
		background: #3b82f6;
		border-radius: 2px;
	}

	.no-volumes {
		padding: 20px 88px;
		display: flex;
		align-items: center;
		gap: 12px;
		color: #64748b;
		font-size: 14px;
	}

	/* Context menu */
	.context-menu {
		position: fixed;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
		padding: 6px;
		min-width: 180px;
		z-index: 1000;
	}

	.context-menu button {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		border-radius: 6px;
		font-size: 14px;
		color: #334155;
		text-align: left;
		transition: background 0.15s;
	}

	.context-menu button:hover:not(:disabled) { background: #f1f5f9; }
	.context-menu button:disabled { opacity: 0.5; cursor: not-allowed; }
	.context-menu button.danger { color: #dc2626; }
	.context-menu button.danger:hover { background: #fef2f2; }

	.menu-divider {
		height: 1px;
		background: #e2e8f0;
		margin: 4px 0;
	}

	/* Disk list */
	.disk-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.disk-card {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px 20px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
	}

	.disk-card.system { opacity: 0.7; }

	.disk-icon-large {
		width: 56px;
		height: 56px;
		background: #e2e8f0;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
	}

	.disk-details { flex: 1; }

	.disk-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 4px;
	}

	.disk-model {
		font-size: 15px;
		font-weight: 500;
		color: #1e293b;
	}

	.system-badge {
		padding: 2px 8px;
		background: #fef3c7;
		color: #92400e;
		border-radius: 10px;
		font-size: 11px;
		font-weight: 500;
	}

	.disk-meta {
		display: flex;
		gap: 16px;
		font-size: 13px;
		color: #64748b;
		margin-bottom: 6px;
	}

	.disk-health {
		display: flex;
		gap: 12px;
	}

	.health-badge {
		padding: 2px 10px;
		border-radius: 10px;
		font-size: 12px;
		font-weight: 500;
	}

	.health-badge.healthy { background: #dcfce7; color: #166534; }
	.health-badge.warning { background: #fef9c3; color: #854d0e; }

	.temp {
		font-size: 13px;
		font-weight: 500;
	}

	.temp.cool { color: #22c55e; }
	.temp.warm { color: #f59e0b; }
	.temp.hot { color: #ef4444; }

	.disk-actions {
		display: flex;
		gap: 8px;
	}

	/* Overview */
	.overview-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 16px;
	}

	.overview-card {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
	}

	.overview-stat {
		display: flex;
		flex-direction: column;
	}

	.overview-stat .stat-value {
		font-size: 24px;
		font-weight: 600;
		color: #1e293b;
	}

	.overview-stat .stat-label {
		font-size: 13px;
		color: #64748b;
	}

	/* States */
	.loading-state,
	.error-state,
	.empty-state,
	.coming-soon {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		padding: 60px 20px;
		color: #64748b;
	}

	.error-state p { color: #dc2626; }

	/* Buttons */
	.btn-primary {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s;
	}

	.btn-primary:hover { background: #2563eb; }
	.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

	.btn-secondary {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 14px;
		color: #475569;
		transition: all 0.15s;
	}

	.btn-secondary:hover { background: #f1f5f9; border-color: #cbd5e1; }

	.btn-danger {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		background: #dc2626;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s;
	}

	.btn-danger:hover { background: #b91c1c; }

	.btn-text {
		color: #3b82f6;
		font-size: 14px;
	}

	.btn-text:hover { text-decoration: underline; }

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 16px;
		box-shadow: 0 25px 50px rgba(0, 0, 0, 0.2);
		width: 600px;
		max-width: 90vw;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
	}

	.modal.modal-sm { width: 420px; }

	.modal-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px 20px;
		border-bottom: 1px solid #e2e8f0;
	}

	.modal-header h2 {
		flex: 1;
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.back-btn,
	.close-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s;
	}

	.back-btn:hover,
	.close-btn:hover { background: #f1f5f9; color: #334155; }

	.modal-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e2e8f0;
	}

	/* Form */
	.form-section {
		margin-bottom: 20px;
	}

	.form-label {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 8px;
	}

	.form-input {
		width: 100%;
		padding: 10px 14px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		transition: border-color 0.15s;
	}

	.form-input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: #374151;
		cursor: pointer;
	}

	.checkbox-label input[type="checkbox"] {
		width: 18px;
		height: 18px;
	}

	/* Disk selector */
	.disk-selector {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.disk-option {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		border: 2px solid #e2e8f0;
		border-radius: 10px;
		text-align: left;
		transition: all 0.15s;
	}

	.disk-option:hover { border-color: #94a3b8; }
	.disk-option.selected { border-color: #3b82f6; background: #eff6ff; }

	.disk-option-info { flex: 1; }

	.disk-option-name {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #1e293b;
	}

	.disk-option-path {
		font-size: 12px;
		color: #64748b;
	}

	.empty-badge {
		padding: 2px 8px;
		background: #dcfce7;
		color: #166534;
		border-radius: 8px;
		font-size: 11px;
		font-weight: 500;
	}

	.no-candidates {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 40px;
		background: #f8fafc;
		border-radius: 10px;
		color: #64748b;
	}

	/* RAID selector */
	.raid-selector {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
	}

	.raid-option {
		padding: 14px;
		border: 2px solid #e2e8f0;
		border-radius: 10px;
		text-align: left;
		transition: all 0.15s;
	}

	.raid-option:hover { border-color: #94a3b8; }
	.raid-option.selected { border-color: #3b82f6; background: #eff6ff; }

	.raid-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 4px;
	}

	.raid-name {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
	}

	.rec-badge {
		padding: 1px 6px;
		background: #dbeafe;
		color: #1d4ed8;
		border-radius: 6px;
		font-size: 10px;
		font-weight: 600;
	}

	.raid-space {
		display: block;
		font-size: 12px;
		color: #64748b;
		margin-bottom: 8px;
	}

	.raid-desc {
		font-size: 12px;
		color: #64748b;
		line-height: 1.4;
	}

	/* Preview */
	.preview-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 20px;
	}

	.preview-header {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 20px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e2e8f0;
	}

	.preview-header h3 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.preview-details {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.preview-row {
		display: flex;
		justify-content: space-between;
		font-size: 14px;
	}

	.preview-row span:first-child { color: #64748b; }
	.preview-row span:last-child { color: #1e293b; font-weight: 500; }

	.warning-text {
		font-size: 14px;
		color: #374151;
		line-height: 1.6;
	}

	.warning-text strong { color: #1e293b; }

	/* SMART modal */
	.smart-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 20px;
	}

	.smart-model {
		display: block;
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.smart-path {
		font-size: 13px;
		color: #64748b;
	}

	.smart-stats {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
		margin-bottom: 24px;
	}

	.smart-stat {
		background: #f8fafc;
		border-radius: 10px;
		padding: 14px;
	}

	.smart-stat .stat-label {
		display: block;
		font-size: 12px;
		color: #64748b;
		margin-bottom: 4px;
	}

	.smart-stat .stat-value {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.attributes-title {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 12px;
	}

	.smart-attributes {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}

	.smart-attr {
		display: flex;
		justify-content: space-between;
		padding: 10px 14px;
		background: #f8fafc;
		border-radius: 8px;
		font-size: 13px;
	}

	.attr-name { color: #64748b; }
	.attr-value { font-weight: 500; color: #1e293b; }

	/* Animation */
	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.animate-spin { animation: spin 1s linear infinite; }

	/* Textarea */
	.form-textarea {
		resize: vertical;
		min-height: 80px;
	}

	/* Warning button */
	.btn-warning {
		border-color: #f59e0b;
		color: #b45309;
	}

	.btn-warning:hover {
		background: #fef3c7;
		border-color: #d97706;
	}

	/* Warning banner */
	.warning-banner {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 16px;
		background: #fef3c7;
		border: 1px solid #fcd34d;
		border-radius: 10px;
		color: #92400e;
		margin-bottom: 16px;
	}

	/* Disk details modal */
	.disk-info-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 20px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e2e8f0;
	}

	.disk-info-content {
		display: flex;
		flex-direction: column;
	}

	.disk-path {
		font-size: 13px;
		color: #64748b;
	}

	.disk-specs {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-bottom: 24px;
	}

	.spec-row {
		display: flex;
		justify-content: space-between;
		padding: 10px 14px;
		background: #f8fafc;
		border-radius: 8px;
	}

	.spec-label {
		font-size: 13px;
		color: #64748b;
	}

	.spec-value {
		font-size: 13px;
		font-weight: 500;
		color: #1e293b;
	}

	.spec-value-small {
		font-size: 11px;
		max-width: 300px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.partitions-title {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 12px;
	}

	.no-partitions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 30px;
		background: #f8fafc;
		border-radius: 10px;
		color: #64748b;
		font-size: 14px;
	}

	.partitions-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.partition-item {
		padding: 14px 16px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
	}

	.partition-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}

	.partition-name {
		font-size: 14px;
		font-weight: 500;
		color: #1e293b;
	}

	.partition-details {
		display: flex;
		gap: 12px;
		font-size: 13px;
		color: #64748b;
		margin-bottom: 6px;
	}

	.partition-fs {
		padding: 1px 8px;
		background: #e2e8f0;
		border-radius: 6px;
		font-size: 12px;
	}

	.partition-label {
		font-style: italic;
	}

	.partition-mount {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: #3b82f6;
	}

	/* Pool health row */
	.pool-health-row {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-top: 6px;
	}

	.health-item {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: #64748b;
	}

	.health-item.rebuild {
		color: #7c3aed;
		font-weight: 500;
	}

	.health-item.scrubbing {
		color: #2563eb;
		font-weight: 500;
	}

	.health-item.scrub-info {
		color: #16a34a;
	}

	.health-item.errors {
		color: #dc2626;
		font-weight: 500;
	}

	.scrub-errors {
		color: #dc2626;
		font-weight: 500;
	}

	/* Mount options display */
	.mount-opts {
		font-family: monospace;
		font-size: 11px;
		background: #f1f5f9;
		padding: 1px 6px;
		border-radius: 4px;
	}

	.checking {
		color: #2563eb;
		font-weight: 500;
	}

	/* Info rows */
	.info-row {
		display: flex;
		justify-content: space-between;
		padding: 10px 0;
		border-bottom: 1px solid #f1f5f9;
	}

	.info-label {
		font-size: 13px;
		color: #64748b;
	}

	.info-value {
		font-size: 13px;
		font-weight: 500;
		color: #1e293b;
	}

	/* Preset selector */
	.preset-selector {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.preset-option {
		padding: 12px 16px;
		border: 2px solid #e2e8f0;
		border-radius: 10px;
		text-align: left;
		transition: all 0.15s;
	}

	.preset-option:hover { border-color: #94a3b8; }
	.preset-option.selected { border-color: #3b82f6; background: #eff6ff; }

	.preset-name {
		font-size: 14px;
		font-weight: 500;
		color: #1e293b;
		margin-bottom: 2px;
	}

	.preset-desc {
		font-size: 12px;
		color: #64748b;
	}

	/* Mount presets inline (for create volume) */
	.mount-presets-inline {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.preset-chip {
		padding: 6px 14px;
		border: 1px solid #e2e8f0;
		border-radius: 20px;
		font-size: 13px;
		color: #475569;
		transition: all 0.15s;
	}

	.preset-chip:hover { border-color: #94a3b8; }
	.preset-chip.selected { border-color: #3b82f6; background: #eff6ff; color: #2563eb; }

	/* Modal tabs */
	.modal-tabs {
		display: flex;
		border-bottom: 1px solid #e2e8f0;
		padding: 0 16px;
	}
	.modal-tabs button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		border: none;
		background: none;
		color: #64748b;
		font-size: 13px;
		cursor: pointer;
		border-bottom: 2px solid transparent;
		transition: all 0.15s;
	}
	.modal-tabs button:hover { color: #334155; }
	.modal-tabs button.active { color: #3b82f6; border-bottom-color: #3b82f6; }

	/* Modal sizes */
	.modal-lg { max-width: 700px; width: 90vw; }
	.modal-sm { max-width: 400px; }

	/* SMART test controls */
	.smart-test-controls { margin-bottom: 16px; }
	.smart-test-controls h3 { font-size: 14px; font-weight: 600; margin-bottom: 8px; }
	.test-buttons { display: flex; gap: 8px; margin-bottom: 12px; }
	.test-progress { margin-top: 8px; }
	.progress-bar {
		height: 6px;
		background: #e2e8f0;
		border-radius: 3px;
		overflow: hidden;
	}
	.progress-fill {
		height: 100%;
		background: #3b82f6;
		border-radius: 3px;
		transition: width 0.3s;
	}
	.progress-text { font-size: 12px; color: #64748b; margin-top: 4px; }

	/* SMART history table */
	.smart-history-table { font-size: 13px; }
	.smart-history-table .table-header,
	.smart-history-table .table-row {
		display: grid;
		grid-template-columns: 40px 1fr 1fr 80px;
		gap: 8px;
		padding: 6px 0;
		align-items: center;
	}
	.smart-history-table .table-header {
		font-weight: 600;
		color: #64748b;
		border-bottom: 1px solid #e2e8f0;
	}
	.smart-history-table .table-row { border-bottom: 1px solid #f1f5f9; }
	.status-cell.text-green { color: #16a34a; }
	.status-cell.text-red { color: #dc2626; }

	/* Schedule items */
	.schedule-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}
	.schedule-header h3 { margin: 0; font-size: 14px; }
	.schedule-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 0;
		border-bottom: 1px solid #f1f5f9;
	}
	.schedule-info { display: flex; gap: 12px; align-items: center; font-size: 13px; }
	.schedule-type { font-weight: 600; text-transform: capitalize; }
	.schedule-interval { color: #3b82f6; }
	.schedule-last { color: #64748b; }
	.schedule-actions { display: flex; gap: 8px; align-items: center; }
	.toggle-btn {
		padding: 4px 12px;
		border-radius: 12px;
		font-size: 12px;
		border: 1px solid #e2e8f0;
		background: #f8fafc;
		color: #64748b;
		cursor: pointer;
	}
	.toggle-btn.active { background: #eff6ff; color: #3b82f6; border-color: #bfdbfe; }

	/* Power presets */
	.power-presets {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}
	.power-presets button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 14px;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		background: #f8fafc;
		color: #475569;
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s;
	}
	.power-presets button:hover { border-color: #94a3b8; }
	.power-presets button.active { border-color: #3b82f6; background: #eff6ff; color: #2563eb; }

	.checkbox-label {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		font-size: 13px;
		cursor: pointer;
	}
	.checkbox-label input[type="checkbox"] { margin-top: 2px; }
	.help-text { display: block; font-size: 12px; color: #94a3b8; margin-top: 2px; }

	/* Snapshot UI */
	.snapshot-create {
		display: flex;
		gap: 8px;
		margin-bottom: 16px;
	}
	.snapshot-create input { flex: 1; }
	.snapshot-list { display: flex; flex-direction: column; gap: 4px; }
	.snapshot-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		background: #f8fafc;
		border-radius: 6px;
	}
	.snapshot-info { display: flex; gap: 10px; align-items: center; font-size: 13px; }
	.snapshot-name { font-weight: 500; }
	.snapshot-date { color: #64748b; }
	.snapshot-size { color: #94a3b8; }
	.badge {
		padding: 2px 8px;
		background: #eff6ff;
		color: #3b82f6;
		border-radius: 10px;
		font-size: 11px;
	}

	/* Grow pool */
	.grow-info { margin-bottom: 16px; }
	.pool-info-text {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		color: #475569;
		padding: 10px;
		background: #f8fafc;
		border-radius: 6px;
	}
	.warning-box {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		background: #fff7ed;
		border: 1px solid #fed7aa;
		border-radius: 6px;
		color: #c2410c;
		font-size: 13px;
		margin-bottom: 16px;
	}
	.disk-candidates { display: flex; flex-direction: column; gap: 4px; }
	.disk-candidate-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
		border: 1px solid #e2e8f0;
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
	}
	.disk-candidate-item:hover { background: #f8fafc; }
	.candidate-model { flex: 1; }
	.candidate-size { color: #64748b; }

	.empty-text { color: #94a3b8; font-size: 13px; text-align: center; padding: 20px; }
	.mt-3 { margin-top: 12px; }
	.mt-4 { margin-top: 16px; }
	.btn-sm { padding: 6px 12px !important; font-size: 12px !important; }
	.btn-icon { padding: 6px; border-radius: 4px; border: none; background: none; cursor: pointer; color: #64748b; }
	.btn-icon:hover { background: #f1f5f9; }
	.btn-icon.danger:hover { background: #fef2f2; color: #dc2626; }
</style>
