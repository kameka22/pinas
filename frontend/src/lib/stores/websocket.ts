import { writable } from 'svelte/store';
import { systemStats } from './system';
import { fileTasks } from './taskManager';
import type { FileTaskType, FileTaskStatus } from './taskManager';

export interface TaskProgress {
	task_id: string;
	package_id: string;
	status: string;
	progress: number;
	total_steps: number;
	progress_percent: number;
	current_step: string | null;
	error_message: string | null;
}

/** Store tracking active task progress events keyed by task_id */
export const taskProgress = writable<Record<string, TaskProgress>>({});

let ws: WebSocket | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
let reconnectAttempts = 0;
let isConnecting = false;

const INITIAL_RECONNECT_DELAY = 2000;
const MAX_RECONNECT_DELAY = 30000;
const MAX_RECONNECT_ATTEMPTS = 10;

export function connectWebSocket(): () => void {
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const wsUrl = `${protocol}//${window.location.hostname}:3000/api/ws`;

	function connect() {
		if (isConnecting || (ws && ws.readyState === WebSocket.OPEN)) {
			return;
		}

		isConnecting = true;

		try {
			ws = new WebSocket(wsUrl);

			ws.onopen = () => {
				console.log('[WS] Connected to server');
				isConnecting = false;
				reconnectAttempts = 0;
				if (reconnectTimeout) {
					clearTimeout(reconnectTimeout);
					reconnectTimeout = null;
				}
			};

			ws.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					handleMessage(data);
				} catch (e) {
					console.error('[WS] Failed to parse message:', e);
				}
			};

			ws.onclose = () => {
				isConnecting = false;
				scheduleReconnect();
			};

			ws.onerror = () => {
				isConnecting = false;
				// Error is logged by onclose, no need to duplicate
			};
		} catch (error) {
			console.error('[WS] Failed to connect:', error);
			isConnecting = false;
			scheduleReconnect();
		}
	}

	function scheduleReconnect() {
		if (reconnectTimeout) return;

		reconnectAttempts++;

		if (reconnectAttempts > MAX_RECONNECT_ATTEMPTS) {
			console.warn('[WS] Max reconnection attempts reached, giving up');
			return;
		}

		// Exponential backoff with jitter
		const delay = Math.min(
			INITIAL_RECONNECT_DELAY * Math.pow(2, reconnectAttempts - 1) + Math.random() * 1000,
			MAX_RECONNECT_DELAY
		);

		if (reconnectAttempts <= 3) {
			console.log(`[WS] Reconnecting in ${Math.round(delay / 1000)}s (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
		}

		reconnectTimeout = setTimeout(() => {
			reconnectTimeout = null;
			connect();
		}, delay);
	}

	function handleMessage(data: any) {
		// Handle both "system.stats" (backend format) and "system_stats" (legacy)
		switch (data.type) {
			case 'system.stats':
				// Backend sends: { type: "system.stats", data: {...} }
				if (data.data) {
					systemStats.set({
						cpuUsage: data.data.cpu_usage,
						memoryUsage: data.data.memory_usage,
						memoryUsed: data.data.memory_used,
						memoryTotal: data.data.memory_total
					});
				}
				break;
			case 'system_stats':
				// Legacy format
				systemStats.set({
					cpuUsage: data.cpu_usage,
					memoryUsage: data.memory_usage,
					memoryUsed: data.memory_used,
					memoryTotal: data.memory_total
				});
				break;
			case 'task.progress':
				if (data.data) {
					const progress: TaskProgress = data.data;
					console.log('[WS] Task progress:', progress.task_id, progress.progress_percent + '%', progress.status, progress.current_step);
					taskProgress.update((tasks) => {
						if (progress.status === 'completed' || progress.status === 'failed') {
							tasks[progress.task_id] = progress;
							setTimeout(() => {
								taskProgress.update((t) => {
									delete t[progress.task_id];
									return { ...t };
								});
							}, 3000);
						} else {
							tasks[progress.task_id] = progress;
						}
						return { ...tasks };
					});
				}
				break;
			case 'file.task':
				if (data.data) {
					const ft = data.data as {
						task_id: string;
						task_type: string;
						file_name: string;
						status: string;
						progress: number;
						error_message: string | null;
					};
					// Upsert: create task if it doesn't exist, otherwise update
					fileTasks.upsertTask(ft.task_id, ft.task_type as FileTaskType, ft.file_name, {
						status: ft.status as FileTaskStatus,
						progress: ft.progress,
						error: ft.error_message || undefined
					});
				}
				break;
			case 'notification':
				console.log('[WS] Notification:', data.message || data.data?.message);
				break;
			default:
				// Silently ignore unknown message types
				break;
		}
	}

	// Start connection
	connect();

	// Return disconnect function
	return () => {
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}
		if (ws) {
			ws.close();
			ws = null;
		}
		reconnectAttempts = 0;
		isConnecting = false;
	};
}

export function sendMessage(message: object) {
	if (ws && ws.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify(message));
	} else {
		console.warn('[WS] Cannot send message, WebSocket not connected');
	}
}
