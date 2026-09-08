import { writable, derived } from 'svelte/store';

export interface SystemStats {
	cpuUsage: number;
	memoryUsage: number;
	memoryUsed: number;
	memoryTotal: number;
}

export interface SystemInfo {
	version: string;
	hostname: string;
	model: string | null;
	serial: string | null;
	osName: string;
	osVersion: string;
	kernelVersion: string;
	uptime: number;
	bootTime: number;
	cpu: {
		model: string;
		cores: number;
		usage: number;
		frequencyMhz: number;
		temperature: number | null;
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
	devMode: boolean;
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

export function formatUptime(seconds: number): string {
	const days = Math.floor(seconds / 86400);
	const hours = Math.floor((seconds % 86400) / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const hm = `${String(hours).padStart(2, '0')}h ${String(minutes).padStart(2, '0')}m`;
	return days > 0 ? `${days}d ${hm}` : hm;
}

export { formatBytes };
