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
			user: { id: string; username: string; role: string };
		}>('/auth/login', { username, password });

		localStorage.setItem('token', response.token);
		localStorage.setItem('user', JSON.stringify(response.user));

		auth.set({
			isAuthenticated: true,
			token: response.token,
			user: response.user
		});
	}

	logout(): void {
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
			role: string;
			enabled: boolean;
		}>>('/users');
	}

	async createUser(user: { username: string; password: string; email?: string; role?: string }) {
		return this.post('/users', user);
	}

	async deleteUser(id: string) {
		return this.delete(`/users/${id}`);
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
