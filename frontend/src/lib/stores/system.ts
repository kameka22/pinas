import { writable, derived } from 'svelte/store';

export interface SystemStats {
	cpuUsage: number;
	memoryUsage: number;
	memoryUsed: number;
	memoryTotal: number;
}

export interface SystemInfo {
	hostname: string;
	osName: string;
	osVersion: string;
	kernelVersion: string;
	uptime: number;
	cpu: {
		model: string;
		cores: number;
		usage: number;
	};
	memory: {
		total: number;
		used: number;
		available: number;
		usagePercent: number;
	};
	loadAverage: {
		one: number;
		five: number;
		fifteen: number;
	};
}

// Real-time stats from WebSocket
export const systemStats = writable<SystemStats>({
	cpuUsage: 0,
	memoryUsage: 0,
	memoryUsed: 0,
	memoryTotal: 0
});

// Full system info from API
export const systemInfo = writable<SystemInfo | null>(null);

// Formatted values
export const formattedMemory = derived(systemStats, ($stats) => {
	const used = formatBytes($stats.memoryUsed);
	const total = formatBytes($stats.memoryTotal);
	return `${used} / ${total}`;
});

// Helper function
function formatBytes(bytes: number): string {
	if (bytes === 0) return '0 B';
	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export { formatBytes };
