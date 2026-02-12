import { writable } from 'svelte/store';

const API_BASE = '/api';

// Auth state
interface AuthState {
	isAuthenticated: boolean;
	user: {
		id: string;
		username: string;
		role: string;
	} | null;
}

export const auth = writable<AuthState>({
	isAuthenticated: false,
	user: null
});

// In-memory token for WebSocket auth (not persisted — cookies handle HTTP requests)
let wsToken: string | null = null;
export function getWsToken(): string | null { return wsToken; }
export function setWsToken(token: string | null) { wsToken = token; }

// Initialize auth from localStorage (user info only, token is in httpOnly cookie)
if (typeof window !== 'undefined') {
	// Migration: clean up legacy token from localStorage
	localStorage.removeItem('token');

	const storedUser = localStorage.getItem('user');

	if (storedUser) {
		auth.set({
			isAuthenticated: true,
			user: JSON.parse(storedUser)
		});
	}
}

// API client
class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(
		method: string,
		endpoint: string,
		data?: unknown
	): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;

		const headers: HeadersInit = {
			'Content-Type': 'application/json'
		};

		const options: RequestInit = {
			method,
			headers,
			credentials: 'include'
		};

		if (data) {
			options.body = JSON.stringify(data);
		}

		const response = await fetch(url, options);

		if (!response.ok) {
			if (response.status === 401) {
				// Token expired or invalid
				this.logout();
			}

			const error = await response.json().catch(() => ({ message: 'Request failed' }));
			const apiError = new Error(error.message || error.error || `HTTP ${response.status}`) as any;
			apiError.status = response.status;
			throw apiError;
		}

		// Handle empty responses (204 No Content or empty body)
		const contentLength = response.headers.get('content-length');
		if (response.status === 204 || contentLength === '0') {
			return undefined as T;
		}

		// Try to parse JSON, return undefined if empty
		const text = await response.text();
		if (!text || text.trim() === '') {
			return undefined as T;
		}

		return JSON.parse(text) as T;
	}

	async get<T>(endpoint: string): Promise<T> {
		return this.request<T>('GET', endpoint);
	}

	async post<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>('POST', endpoint, data);
	}

	async put<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>('PUT', endpoint, data);
	}

	async delete<T>(endpoint: string): Promise<T> {
		return this.request<T>('DELETE', endpoint);
	}

	async patch<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>('PATCH', endpoint, data);
	}

	// Auth methods
	async login(username: string, password: string): Promise<void> {
		const response = await this.post<{
			token: string;
			user: { id: string; username: string; email: string | null; is_admin: boolean };
		}>('/auth/login', { username, password });

		const user = {
			id: response.user.id,
			username: response.user.username,
			role: response.user.is_admin ? 'admin' : 'user'
		};

		// Token is stored in httpOnly cookie by the server for HTTP requests.
		// Keep a copy in memory for WebSocket auth (query param fallback).
		wsToken = response.token;
		localStorage.setItem('user', JSON.stringify(user));

		auth.set({
			isAuthenticated: true,
			user
		});
	}

	async logout(): Promise<void> {
		// Call backend logout endpoint to invalidate session (also clears httpOnly cookie)
		try {
			await this.post('/auth/logout');
		} catch (e) {
			// Ignore errors - we'll clear local state anyway
			console.warn('Logout API call failed:', e);
		}

		wsToken = null;
		localStorage.removeItem('user');

		auth.set({
			isAuthenticated: false,
			user: null
		});
	}

	// System endpoints
	async getSystemInfo() {
		return this.get<{
			version: string;
			hostname: string;
			os_name: string;
			os_version: string;
			kernel_version: string;
			uptime: number;
			cpu: { model: string; cores: number; usage: number };
			memory: { total: number; used: number; available: number; usage_percent: number };
			load_average: { one: number; five: number; fifteen: number };
			dev_mode: boolean;
		}>('/system/info');
	}

	async getProcesses(): Promise<ProcessListResponse> {
		return this.get<ProcessListResponse>('/system/processes');
	}

	async killProcess(pid: number): Promise<{ success: boolean; message: string }> {
		return this.post<{ success: boolean; message: string }>(`/system/processes/${pid}/kill`);
	}

	async rebootSystem(): Promise<void> {
		await this.post('/system/reboot');
	}

	async shutdownSystem(): Promise<void> {
		await this.post('/system/shutdown');
	}

	// Storage endpoints - Disks
	async getDisks(): Promise<Disk[]> {
		return this.get<Disk[]>('/storage/disks');
	}

	async getDiskSmartInfo(deviceName: string): Promise<SmartInfo> {
		return this.get<SmartInfo>(`/storage/disks/${encodeURIComponent(deviceName)}/smart`);
	}

	async wipeDisk(deviceName: string): Promise<void> {
		return this.post(`/storage/disks/${encodeURIComponent(deviceName)}/wipe`);
	}

	async getDiskCandidates(): Promise<DiskCandidate[]> {
		return this.get<DiskCandidate[]>('/storage/candidates');
	}

	// Storage endpoints - Pools
	async getPools(): Promise<StoragePool[]> {
		return this.get<StoragePool[]>('/storage/pools');
	}

	async getPool(id: string): Promise<StoragePool> {
		return this.get<StoragePool>(`/storage/pools/${id}`);
	}

	async createPool(data: CreatePoolRequest): Promise<{ id: string }> {
		return this.post<{ id: string }>('/storage/pools', data);
	}

	async updatePool(id: string, data: UpdatePoolRequest): Promise<void> {
		return this.put(`/storage/pools/${id}`, data);
	}

	async deletePool(id: string): Promise<void> {
		return this.delete(`/storage/pools/${id}`);
	}

	async scrubPool(id: string): Promise<void> {
		return this.post(`/storage/pools/${id}/scrub`);
	}

	// Storage endpoints - Volumes
	async getVolumes(): Promise<VolumeInfo[]> {
		return this.get<VolumeInfo[]>('/storage/volumes');
	}

	async getVolume(id: string): Promise<VolumeInfo> {
		return this.get<VolumeInfo>(`/storage/volumes/${id}`);
	}

	async createVolume(poolId: string, data: CreateVolumeRequest): Promise<{ id: string }> {
		return this.post<{ id: string }>(`/storage/pools/${poolId}/volumes`, data);
	}

	async deleteVolume(id: string): Promise<void> {
		return this.delete(`/storage/volumes/${id}`);
	}

	async mountVolume(id: string): Promise<void> {
		return this.post(`/storage/volumes/${id}/mount`);
	}

	async unmountVolume(id: string): Promise<void> {
		return this.post(`/storage/volumes/${id}/unmount`);
	}

	// Legacy filesystem endpoint
	async getFilesystems(): Promise<VolumeInfo[]> {
		return this.get<VolumeInfo[]>('/storage/filesystems');
	}

	// Shares endpoints
	async getShares(): Promise<ShareInfo[]> {
		return this.get<ShareInfo[]>('/shares');
	}

	async getShare(id: string): Promise<ShareInfo> {
		return this.get<ShareInfo>(`/shares/${id}`);
	}

	async createShare(data: CreateShareRequest): Promise<ShareInfo> {
		return this.post<ShareInfo>('/shares', data);
	}

	async updateShare(id: string, data: UpdateShareRequest): Promise<ShareInfo> {
		return this.put<ShareInfo>(`/shares/${id}`, data);
	}

	async deleteShare(id: string): Promise<void> {
		return this.delete(`/shares/${id}`);
	}

	async toggleShare(id: string, enabled: boolean): Promise<ShareInfo> {
		return this.post<ShareInfo>(`/shares/${id}/toggle`, { enabled });
	}

	async getSambaStatus(): Promise<SambaStatus> {
		return this.get<SambaStatus>('/shares/samba/status');
	}

	async enableSamba(): Promise<void> {
		return this.post('/shares/samba/enable');
	}

	async disableSamba(): Promise<void> {
		return this.post('/shares/samba/disable');
	}

	async getSmbConfig(): Promise<SmbGlobalConfig> {
		return this.get<SmbGlobalConfig>('/shares/samba/config');
	}

	async updateSmbConfig(config: SmbGlobalConfig): Promise<void> {
		return this.put('/shares/samba/config', config);
	}

	// Users endpoints
	async getUsers() {
		return this.get<Array<{
			id: string;
			username: string;
			email: string | null;
			is_admin: boolean;
			created_at: string;
			updated_at: string;
		}>>('/users');
	}

	async createUser(user: { username: string; password: string; email?: string; is_admin?: boolean }) {
		return this.post('/users', user);
	}

	async deleteUser(id: string) {
		return this.delete(`/users/${id}`);
	}

	async updateUser(id: string, data: { email?: string; is_admin?: boolean }) {
		return this.put(`/users/${id}`, data);
	}

	// Setup endpoints
	async getSetupStatus(): Promise<{ is_complete: boolean; needs_setup: boolean }> {
		return this.get('/setup/status');
	}

	async completeSetup(data: {
		machine_name: string;
		admin_username: string;
		admin_password: string;
	}): Promise<{
		token: string;
		user: { id: string; username: string; is_admin: boolean };
	}> {
		return this.post('/setup/complete', data);
	}

	// Auth - change password
	async changePassword(currentPassword: string, newPassword: string): Promise<void> {
		return this.post('/auth/change-password', {
			current_password: currentPassword,
			new_password: newPassword
		});
	}

	// Get current user profile
	async getProfile(): Promise<{
		id: string;
		username: string;
		email: string | null;
		is_admin: boolean;
	}> {
		return this.get('/auth/me');
	}

	// Groups endpoints
	async getGroups(): Promise<Array<{
		id: string;
		name: string;
		description: string | null;
		is_system: boolean;
		member_count: number;
		created_at: string;
		updated_at: string;
	}>> {
		return this.get('/groups');
	}

	async createGroup(data: { name: string; description?: string }): Promise<{
		id: string;
		name: string;
		description: string | null;
		is_system: boolean;
		member_count: number;
	}> {
		return this.post('/groups', data);
	}

	async updateGroup(id: string, data: { name?: string; description?: string }): Promise<{
		id: string;
		name: string;
		description: string | null;
	}> {
		return this.put(`/groups/${id}`, data);
	}

	async deleteGroup(id: string): Promise<void> {
		return this.delete(`/groups/${id}`);
	}

	async getGroupMembers(groupId: string): Promise<Array<{
		id: string;
		username: string;
		email: string | null;
		is_admin: boolean;
	}>> {
		return this.get(`/groups/${groupId}/members`);
	}

	async addGroupMember(groupId: string, userId: string): Promise<void> {
		return this.post(`/groups/${groupId}/members`, { user_id: userId });
	}

	async removeGroupMember(groupId: string, userId: string): Promise<void> {
		return this.delete(`/groups/${groupId}/members/${userId}`);
	}

	// Locations endpoints
	async getLocations(): Promise<BrowsableLocation[]> {
		return this.get<BrowsableLocation[]>('/locations');
	}

	// Files endpoints
	async getFiles(path: string = '', locationId?: string): Promise<FileItem[]> {
		const params = new URLSearchParams();
		params.set('path', path);
		if (locationId) {
			params.set('location_id', locationId);
		}
		return this.get<FileItem[]>(`/files?${params.toString()}`);
	}

	async createFolder(parentPath: string, name: string, locationId?: string): Promise<FileItem> {
		return this.post<FileItem>('/files/folder', { path: parentPath, name, location_id: locationId });
	}

	async deleteFile(path: string, locationId?: string): Promise<void> {
		const params = new URLSearchParams();
		params.set('path', path);
		if (locationId) {
			params.set('location_id', locationId);
		}
		return this.request<void>('DELETE', `/files?${params.toString()}`);
	}

	async renameFile(path: string, newName: string, locationId?: string): Promise<FileItem> {
		return this.patch<FileItem>('/files/rename', { path, new_name: newName, location_id: locationId });
	}

	async createFile(parentPath: string, name: string, locationId?: string): Promise<FileItem> {
		return this.post<FileItem>('/files/file', { path: parentPath, name, location_id: locationId });
	}

	async copyFiles(sources: string[], destination: string, locationId?: string): Promise<{ task_id: string }> {
		return this.post<{ task_id: string }>('/files/copy', { sources, destination, location_id: locationId });
	}

	async moveFiles(sources: string[], destination: string, locationId?: string): Promise<{ task_id: string }> {
		return this.post<{ task_id: string }>('/files/move', { sources, destination, location_id: locationId });
	}

	async uploadFile(file: File, path: string, locationId?: string): Promise<void> {
		const formData = new FormData();
		formData.append('file', file);
		formData.append('path', path);
		if (locationId) {
			formData.append('location_id', locationId);
		}

		const response = await fetch(`${this.baseUrl}/files/upload`, {
			method: 'POST',
			credentials: 'include',
			body: formData
		});

		if (!response.ok) {
			if (response.status === 401) {
				this.logout();
			}
			const error = await response.json().catch(() => ({ message: 'Upload failed' }));
			throw new Error(error.message || `HTTP ${response.status}`);
		}
	}

	async downloadFile(path: string, locationId?: string): Promise<void> {
		const params = new URLSearchParams();
		params.set('path', path);
		if (locationId) {
			params.set('location_id', locationId);
		}

		const response = await fetch(`${this.baseUrl}/files/download?${params.toString()}`, {
			method: 'GET',
			credentials: 'include'
		});

		if (!response.ok) {
			throw new Error('Download failed');
		}

		const blob = await response.blob();
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = path.split('/').pop() || 'download';
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	// Permissions endpoints
	async getPermissions(): Promise<FolderPermissions[]> {
		return this.get<FolderPermissions[]>('/permissions');
	}

	async getPermissionFolders(): Promise<string[]> {
		return this.get<string[]>('/permissions/folders');
	}

	async getFolderPermissions(path: string): Promise<FolderPermissions> {
		return this.get<FolderPermissions>(`/permissions/folder?path=${encodeURIComponent(path)}`);
	}

	async getUserPermissions(userId: string): Promise<PermissionResponse[]> {
		return this.get<PermissionResponse[]>(`/permissions/user/${userId}`);
	}

	async createPermission(data: CreatePermissionRequest): Promise<PermissionResponse> {
		return this.post<PermissionResponse>('/permissions', data);
	}

	async updatePermission(id: string, permission: PermissionLevel): Promise<PermissionResponse> {
		return this.put<PermissionResponse>(`/permissions/${id}`, { permission });
	}

	async deletePermission(id: string): Promise<void> {
		return this.delete(`/permissions/${id}`);
	}

	// SSH endpoints
	async getSshStatus(): Promise<SshStatus> {
		return this.get<SshStatus>('/ssh/status');
	}

	async enableSsh(): Promise<void> {
		return this.post<void>('/ssh/enable');
	}

	async disableSsh(): Promise<void> {
		return this.post<void>('/ssh/disable');
	}

	async changeSshPassword(password: string): Promise<void> {
		return this.post<void>('/ssh/password', { password });
	}

	// CUPS endpoints
	async getCupsStatus(): Promise<CupsStatus> {
		return this.get<CupsStatus>('/cups/status');
	}

	async enableCups(): Promise<void> {
		return this.post<void>('/cups/enable');
	}

	async disableCups(): Promise<void> {
		return this.post<void>('/cups/disable');
	}

	async getCupsPrinters(): Promise<CupsPrinter[]> {
		return this.get<CupsPrinter[]>('/cups/printers');
	}

	async detectPrinters(): Promise<DetectedPrinter[]> {
		return this.get<DetectedPrinter[]>('/cups/detect');
	}

	async getPrinterDrivers(uri: string): Promise<PrinterDriver[]> {
		return this.get<PrinterDriver[]>(`/cups/drivers?uri=${encodeURIComponent(uri)}`);
	}

	async addPrinter(data: AddPrinterRequest): Promise<void> {
		return this.post<void>('/cups/printers', data);
	}

	async removePrinter(name: string): Promise<void> {
		return this.delete(`/cups/printers/${encodeURIComponent(name)}`);
	}

	async updatePrinter(name: string, data: UpdatePrinterRequest): Promise<void> {
		return this.put<void>(`/cups/printers/${encodeURIComponent(name)}`, data);
	}

	async printTestPage(printerName: string): Promise<void> {
		return this.post<void>(`/cups/printers/${encodeURIComponent(printerName)}/test`);
	}

	async getPrintJobs(printer?: string): Promise<PrintJob[]> {
		const params = printer ? `?printer=${encodeURIComponent(printer)}` : '';
		return this.get<PrintJob[]>(`/cups/jobs${params}`);
	}

	async cancelPrintJob(jobId: number): Promise<void> {
		return this.delete(`/cups/jobs/${jobId}`);
	}

	// Network endpoints
	async getNetworkStatus(): Promise<NetworkStatus> {
		return this.get<NetworkStatus>('/network/status');
	}

	async updateNetworkInterface(config: UpdateInterfaceConfig): Promise<void> {
		return this.put<void>('/network/interface', config);
	}

	async updateNetworkDns(config: DnsConfig): Promise<void> {
		return this.put<void>('/network/dns', config);
	}

	async updateNetworkHostname(hostname: string): Promise<void> {
		return this.put<void>('/network/hostname', { hostname });
	}

	// System Update endpoints
	async checkForUpdate(): Promise<UpdateCheckResult> {
		return this.get<UpdateCheckResult>('/system/update/check');
	}

	async installUpdate(): Promise<{ task_id: string }> {
		return this.post<{ task_id: string }>('/system/update/install');
	}

	async getUpdateStatus(): Promise<UpdateStatusResult> {
		return this.get<UpdateStatusResult>('/system/update/status');
	}

	async getUpdateHistory(): Promise<UpdateHistoryEntry[]> {
		return this.get<UpdateHistoryEntry[]>('/system/update/history');
	}

	async getJustUpdated(): Promise<JustUpdatedResult> {
		return this.get<JustUpdatedResult>('/system/update/just-updated');
	}

	async dismissUpdate(): Promise<void> {
		return this.post<void>('/system/update/dismiss');
	}

	// Preferences endpoints
	async getPreferences(): Promise<Record<string, string>> {
		return this.get<Record<string, string>>('/preferences');
	}

	async getPreference(key: string): Promise<{ value: string }> {
		return this.get<{ value: string }>(`/preferences/${encodeURIComponent(key)}`);
	}

	async setPreference(key: string, value: string): Promise<void> {
		return this.put<void>(`/preferences/${encodeURIComponent(key)}`, { value });
	}
}

// File item type
export interface FileItem {
	name: string;
	path: string;
	type: 'file' | 'folder';
	size: number | null;
	modified: string;
	mime_type?: string;
}

// Browsable location types for File Manager sidebar
export type BrowsableLocationType = 'home' | 'share' | 'volume' | 'media';

export interface BrowsableLocation {
	type: BrowsableLocationType;
	id: string;
	name: string;
	path: string;
	icon: string;
	// Share-specific fields
	share_type?: string;
	enabled?: boolean;
	// Volume-specific fields
	status?: string;
	fs_type?: string;
	usage_percent?: number;
	pool_name?: string;
	// Media-specific fields
	device?: string;
}

// Storage types
export type DiskType = 'hdd' | 'ssd' | 'nvme' | 'sd' | 'usb' | 'unknown';
export type RaidType = 'basic' | 'jbod' | 'raid0' | 'raid1' | 'raid5' | 'raid10' | 'btrfs-single' | 'btrfs-raid0' | 'btrfs-raid1' | 'btrfs-raid10';
export type PoolStatus = 'normal' | 'degraded' | 'rebuilding' | 'error' | 'creating';
export type VolumeStatus = 'mounted' | 'unmounted' | 'error' | 'creating';

export interface Partition {
	device_path: string;
	number: number;
	size: number;
	fs_type: string | null;
	label: string | null;
	uuid: string | null;
	mount_point: string | null;
	is_system: boolean;
}

export interface Disk {
	device_name: string;
	device_path: string;
	device_by_id: string | null;
	model: string;
	serial: string | null;
	size: number;
	disk_type: DiskType;
	temperature: number | null;
	health_status: string | null;
	is_system: boolean;
	is_removable: boolean;
	partitions: Partition[];
}

export interface DiskCandidate {
	device_path: string;
	device_by_id: string | null;
	model: string;
	size: number;
	disk_type: DiskType;
	is_empty: boolean;
}

export interface SmartAttribute {
	id: number;
	name: string;
	value: number;
	worst: number;
	threshold: number;
	raw_value: string;
}

export interface SmartInfo {
	device_path: string;
	model: string;
	serial: string | null;
	firmware: string | null;
	health_status: string;
	temperature: number | null;
	power_on_hours: number | null;
	power_cycle_count: number | null;
	reallocated_sectors: number | null;
	pending_sectors: number | null;
	attributes: SmartAttribute[];
}

export interface VolumeInfo {
	id: string;
	pool_id: string;
	name: string;
	fs_type: string;
	mount_point: string;
	size: number;
	used: number;
	available: number;
	usage_percent: number;
	status: VolumeStatus;
	created_at: string;
}

export interface StoragePool {
	id: string;
	name: string;
	description: string | null;
	raid_type: RaidType;
	status: PoolStatus;
	devices: string[];
	total_size: number;
	used_size: number;
	available_size: number;
	volumes: VolumeInfo[];
	created_at: string;
}

export interface CreatePoolRequest {
	name: string;
	description?: string;
	raid_type: RaidType;
	devices: string[];
	wipe_devices?: boolean;
}

export interface UpdatePoolRequest {
	name?: string;
	description?: string;
}

export interface CreateVolumeRequest {
	name: string;
	fs_type: string;
}

// Process types
export interface ProcessInfo {
	pid: number;
	name: string;
	user: string;
	cpu: number;
	memory: number;
	memory_percent: number;
	status: string;
	command: string;
	start_time: number;
}

export interface ProcessListResponse {
	processes: ProcessInfo[];
	total_processes: number;
	running_processes: number;
	cpu_usage: number;
	memory_usage: number;
	total_memory: number;
	used_memory: number;
}

// Permission types
export type PermissionLevel = 'none' | 'read' | 'write';

export interface PermissionEntry {
	id: string;
	user_id?: string;
	username?: string;
	group_id?: string;
	group_name?: string;
	permission: PermissionLevel;
}

export interface FolderPermissions {
	path: string;
	permissions: PermissionEntry[];
}

export interface PermissionResponse {
	id: string;
	path: string;
	user_id: string | null;
	group_id: string | null;
	permission: PermissionLevel;
}

export interface CreatePermissionRequest {
	path: string;
	user_id?: string;
	group_id?: string;
	permission: PermissionLevel;
}

// Share types
export interface SmbShareConfig {
	guest_ok: boolean;
	browseable: boolean;
	read_only: boolean;
	create_mask: string;
	directory_mask: string;
	veto_files?: string;
	recycle_bin: boolean;
}

export interface ShareInfo {
	id: string;
	name: string;
	path: string;
	share_type: string;
	enabled: boolean;
	description?: string;
	config: SmbShareConfig;
	permissions: PermissionEntry[];
	created_at: string;
	updated_at: string;
}

export interface CreateShareRequest {
	name: string;
	path: string;
	share_type?: string;
	description?: string;
	config?: Partial<SmbShareConfig>;
}

export interface UpdateShareRequest {
	name?: string;
	description?: string | null;
	config?: Partial<SmbShareConfig>;
}

export interface SambaStatus {
	enabled: boolean;
	running: boolean;
	share_count: number;
	connected_users: number;
	version?: string;
}

export interface SmbGlobalConfig {
	workgroup: string;
	server_string: string;
	min_protocol: string;
	max_protocol: string;
}

// Network types
export interface NetworkStatus {
	hostname: string;
	interfaces: NetworkInterface[];
	dns: DnsConfig;
	default_gateway: string;
}

export interface NetworkInterface {
	name: string;
	display_name: string;
	status: string;
	ip_address: string;
	subnet_mask: string;
	mac_address: string;
	speed: string;
	method: string;
	gateway: string;
	dns: string;
}

export interface DnsConfig {
	manual: boolean;
	primary: string;
	secondary: string;
}

export interface UpdateInterfaceConfig {
	name: string;
	method: string;
	ip_address?: string;
	subnet_mask?: string;
	gateway?: string;
	dns?: string;
}

// SSH types
export interface SshStatus {
	enabled: boolean;
	running: boolean;
	port: number;
}

// CUPS types
export interface CupsStatus {
	enabled: boolean;
	running: boolean;
	printer_count: number;
}

export interface CupsPrinter {
	name: string;
	uri: string;
	state: string;
	state_message: string;
	shared: boolean;
	is_default: boolean;
	model: string;
	location: string;
}

export interface DetectedPrinter {
	uri: string;
	model: string;
}

export interface PrinterDriver {
	id: string;
	name: string;
}

export interface PrintJob {
	id: number;
	printer: string;
	title: string;
	user: string;
	state: string;
	size: number;
	created_at: string;
}

export interface AddPrinterRequest {
	name: string;
	uri: string;
	driver: string;
	location?: string;
	shared?: boolean;
}

export interface UpdatePrinterRequest {
	shared?: boolean;
	is_default?: boolean;
	location?: string;
}

// System Update types
export interface UpdateCheckResult {
	available: boolean;
	current_version: string;
	latest_version: string;
	update_type: string | null;
	reboot_required: boolean | null;
	changelog: Record<string, string> | null;
	download_size: number | null;
	published_at: string | null;
}

export interface UpdateStatusResult {
	id?: string;
	version?: string;
	previous_version?: string;
	update_type?: string;
	status: string;
	changelog?: string;
	error_message?: string;
	started_at?: string;
	completed_at?: string;
	created_at?: string;
}

export interface UpdateHistoryEntry {
	id: string;
	version: string;
	previous_version: string;
	update_type: string;
	status: string;
	changelog: string | null;
	error_message: string | null;
	started_at: string | null;
	completed_at: string | null;
	created_at: string;
}

export interface JustUpdatedResult {
	just_updated: boolean;
	version: string | null;
	previous_version: string | null;
	changelog: Record<string, string> | null;
}

export const api = new ApiClient(API_BASE);
