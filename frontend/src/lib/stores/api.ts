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
	token: string | null;
}

export const auth = writable<AuthState>({
	isAuthenticated: false,
	user: null,
	token: null
});

// Initialize auth from localStorage
if (typeof window !== 'undefined') {
	const storedToken = localStorage.getItem('token');
	const storedUser = localStorage.getItem('user');

	if (storedToken && storedUser) {
		auth.set({
			isAuthenticated: true,
			token: storedToken,
			user: JSON.parse(storedUser)
		});
	}
}

// API client
class ApiClient {
	private baseUrl: string;
	private token: string | null = null;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;

		// Subscribe to auth changes
		auth.subscribe((state) => {
			this.token = state.token;
		});
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

		if (this.token) {
			headers['Authorization'] = `Bearer ${this.token}`;
		}

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
			throw new Error(error.message || `HTTP ${response.status}`);
		}

		return response.json();
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

		localStorage.setItem('token', response.token);
		localStorage.setItem('user', JSON.stringify(user));

		auth.set({
			isAuthenticated: true,
			token: response.token,
			user
		});
	}

	async logout(): Promise<void> {
		// Call backend logout endpoint to invalidate session
		try {
			await this.post('/auth/logout');
		} catch (e) {
			// Ignore errors - we'll clear local state anyway
			console.warn('Logout API call failed:', e);
		}

		localStorage.removeItem('token');
		localStorage.removeItem('user');

		auth.set({
			isAuthenticated: false,
			token: null,
			user: null
		});
	}

	// System endpoints
	async getSystemInfo() {
		return this.get<{
			hostname: string;
			os_name: string;
			os_version: string;
			kernel_version: string;
			uptime: number;
			cpu: { model: string; cores: number; usage: number };
			memory: { total: number; used: number; available: number; usage_percent: number };
			load_average: { one: number; five: number; fifteen: number };
		}>('/system/info');
	}

	// Storage endpoints
	async getDisks() {
		return this.get<Array<{
			device: string;
			name: string;
			size: number;
			used: number;
			mount_point: string | null;
			filesystem: string | null;
			is_removable: boolean;
		}>>('/storage/disks');
	}

	async getFilesystems() {
		return this.get<Array<{
			device: string;
			mount_point: string;
			filesystem: string;
			total: number;
			used: number;
			available: number;
		}>>('/storage/filesystems');
	}

	// Shares endpoints
	async getShares() {
		return this.get<Array<{
			id: string;
			name: string;
			path: string;
			protocol: string;
			enabled: boolean;
		}>>('/shares');
	}

	async createShare(share: { name: string; path: string; protocol: string }) {
		return this.post('/shares', share);
	}

	async deleteShare(id: string) {
		return this.delete(`/shares/${id}`);
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

	// Files endpoints
	async getFiles(path: string = ''): Promise<FileItem[]> {
		const encodedPath = encodeURIComponent(path);
		return this.get<FileItem[]>(`/files?path=${encodedPath}`);
	}

	async createFolder(parentPath: string, name: string): Promise<FileItem> {
		return this.post<FileItem>('/files/folder', { path: parentPath, name });
	}

	async deleteFile(path: string): Promise<void> {
		const encodedPath = encodeURIComponent(path);
		return this.request<void>('DELETE', `/files?path=${encodedPath}`);
	}

	async renameFile(path: string, newName: string): Promise<FileItem> {
		return this.patch<FileItem>('/files/rename', { path, new_name: newName });
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

export const api = new ApiClient(API_BASE);
