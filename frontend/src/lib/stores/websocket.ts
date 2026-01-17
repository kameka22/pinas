import { systemStats } from './system';

let ws: WebSocket | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
const RECONNECT_DELAY = 5000;

export function connectWebSocket(): () => void {
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const wsUrl = `${protocol}//${window.location.hostname}:3000/api/ws`;

	function connect() {
		try {
			ws = new WebSocket(wsUrl);

			ws.onopen = () => {
				console.log('[WS] Connected to server');
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
				console.log('[WS] Connection closed, reconnecting...');
				scheduleReconnect();
			};

			ws.onerror = (error) => {
				console.error('[WS] WebSocket error:', error);
			};
		} catch (error) {
			console.error('[WS] Failed to connect:', error);
			scheduleReconnect();
		}
	}

	function scheduleReconnect() {
		if (!reconnectTimeout) {
			reconnectTimeout = setTimeout(() => {
				reconnectTimeout = null;
				connect();
			}, RECONNECT_DELAY);
		}
	}

	function handleMessage(data: any) {
		switch (data.type) {
			case 'system_stats':
				systemStats.set({
					cpuUsage: data.cpu_usage,
					memoryUsage: data.memory_usage,
					memoryUsed: data.memory_used,
					memoryTotal: data.memory_total
				});
				break;
			case 'notification':
				// Handle notifications
				console.log('[WS] Notification:', data.message);
				break;
			default:
				console.log('[WS] Unknown message type:', data.type);
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
	};
}

export function sendMessage(message: object) {
	if (ws && ws.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify(message));
	} else {
		console.warn('[WS] Cannot send message, WebSocket not connected');
	}
}
