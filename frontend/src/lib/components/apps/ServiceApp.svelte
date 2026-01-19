<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	// Config passed from manifest
	export let config: {
		service_name: string;
		display_name?: string;
		description?: string;
		show_logs?: boolean;
		show_config?: boolean;
		restart_on_config?: boolean;
	} = { service_name: 'unknown' };

	// App metadata
	export let name: string = config.display_name || config.service_name;
	export let icon: string = 'mdi:cog';
	export let gradient: string = 'from-slate-500 to-slate-600';

	interface ServiceStatus {
		running: boolean;
		enabled: boolean;
		uptime: number | null;
		memory_usage: number | null;
		cpu_usage: number | null;
	}

	interface LogEntry {
		timestamp: string;
		level: 'info' | 'warn' | 'error';
		message: string;
	}

	let activeTab: 'status' | 'logs' | 'config' = 'status';
	let status: ServiceStatus | null = null;
	let logs: LogEntry[] = [];
	let loading = true;
	let actionInProgress = false;
	let error: string | null = null;
	let refreshInterval: ReturnType<typeof setInterval>;

	async function fetchStatus() {
		try {
			const response = await fetch(`/api/services/${config.service_name}/status`);
			if (response.ok) {
				status = await response.json();
			} else {
				// Mock status for development
				status = {
					running: Math.random() > 0.3,
					enabled: true,
					uptime: Math.floor(Math.random() * 86400),
					memory_usage: Math.floor(Math.random() * 100000000),
					cpu_usage: Math.random() * 5
				};
			}
		} catch (e) {
			// Mock for development
			status = {
				running: true,
				enabled: true,
				uptime: 3600,
				memory_usage: 50000000,
				cpu_usage: 2.5
			};
		}
		loading = false;
	}

	async function fetchLogs() {
		try {
			const response = await fetch(`/api/services/${config.service_name}/logs?lines=100`);
			if (response.ok) {
				logs = await response.json();
			}
		} catch (e) {
			// Mock logs
			logs = [
				{ timestamp: new Date().toISOString(), level: 'info', message: `${config.service_name} started successfully` },
				{ timestamp: new Date().toISOString(), level: 'info', message: 'Listening on all interfaces' },
			];
		}
	}

	async function performAction(action: 'start' | 'stop' | 'restart') {
		actionInProgress = true;
		error = null;

		try {
			const response = await fetch(`/api/services/${config.service_name}/${action}`, {
				method: 'POST'
			});

			if (!response.ok) {
				throw new Error(`Failed to ${action} service`);
			}

			await fetchStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : `Failed to ${action} service`;
			// Mock success for dev
			await fetchStatus();
		} finally {
			actionInProgress = false;
		}
	}

	function formatUptime(seconds: number | null): string {
		if (seconds === null) return '-';
		const days = Math.floor(seconds / 86400);
		const hours = Math.floor((seconds % 86400) / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);

		if (days > 0) return `${days}d ${hours}h ${minutes}m`;
		if (hours > 0) return `${hours}h ${minutes}m`;
		return `${minutes}m`;
	}

	function formatBytes(bytes: number | null): string {
		if (bytes === null) return '-';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}

	function formatTime(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString();
	}

	onMount(() => {
		fetchStatus();
		if (config.show_logs) fetchLogs();
		refreshInterval = setInterval(fetchStatus, 10000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="service-app">
	<!-- Header -->
	<header class="header">
		<div class="header-icon bg-gradient-to-br {gradient}">
			<Icon icon={icon} class="w-8 h-8 text-white" />
		</div>
		<div class="header-info">
			<h1>{name}</h1>
			{#if config.description}
				<p>{config.description}</p>
			{/if}
		</div>
		<div class="header-status">
			{#if loading}
				<span class="status-badge loading">
					<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
				</span>
			{:else if status?.running}
				<span class="status-badge running">
					<Icon icon="mdi:check-circle" class="w-4 h-4" />
					{$t.serviceApp?.running || 'Running'}
				</span>
			{:else}
				<span class="status-badge stopped">
					<Icon icon="mdi:stop-circle" class="w-4 h-4" />
					{$t.serviceApp?.stopped || 'Stopped'}
				</span>
			{/if}
		</div>
	</header>

	<!-- Tabs -->
	<nav class="tabs">
		<button
			class="tab"
			class:active={activeTab === 'status'}
			on:click={() => (activeTab = 'status')}
		>
			<Icon icon="mdi:information-outline" class="w-4 h-4" />
			{$t.serviceApp?.status || 'Status'}
		</button>
		{#if config.show_logs}
			<button
				class="tab"
				class:active={activeTab === 'logs'}
				on:click={() => { activeTab = 'logs'; fetchLogs(); }}
			>
				<Icon icon="mdi:text-box-outline" class="w-4 h-4" />
				{$t.serviceApp?.logs || 'Logs'}
			</button>
		{/if}
		{#if config.show_config}
			<button
				class="tab"
				class:active={activeTab === 'config'}
				on:click={() => (activeTab = 'config')}
			>
				<Icon icon="mdi:cog-outline" class="w-4 h-4" />
				{$t.serviceApp?.config || 'Configuration'}
			</button>
		{/if}
	</nav>

	<!-- Content -->
	<main class="content">
		{#if activeTab === 'status'}
			<div class="status-view">
				<!-- Actions -->
				<div class="actions-card">
					<h3>{$t.serviceApp?.actions || 'Actions'}</h3>
					<div class="action-buttons">
						{#if status?.running}
							<button
								class="action-btn"
								on:click={() => performAction('stop')}
								disabled={actionInProgress}
							>
								<Icon icon="mdi:stop" class="w-5 h-5" />
								{$t.serviceApp?.stop || 'Stop'}
							</button>
							<button
								class="action-btn primary"
								on:click={() => performAction('restart')}
								disabled={actionInProgress}
							>
								<Icon icon="mdi:restart" class="w-5 h-5" />
								{$t.serviceApp?.restart || 'Restart'}
							</button>
						{:else}
							<button
								class="action-btn primary"
								on:click={() => performAction('start')}
								disabled={actionInProgress}
							>
								<Icon icon="mdi:play" class="w-5 h-5" />
								{$t.serviceApp?.start || 'Start'}
							</button>
						{/if}
					</div>
					{#if error}
						<p class="error-message">{error}</p>
					{/if}
				</div>

				<!-- Stats -->
				<div class="stats-grid">
					<div class="stat-card">
						<Icon icon="mdi:clock-outline" class="w-6 h-6 text-blue-500" />
						<div class="stat-info">
							<span class="stat-label">{$t.serviceApp?.uptime || 'Uptime'}</span>
							<span class="stat-value">{formatUptime(status?.uptime ?? null)}</span>
						</div>
					</div>
					<div class="stat-card">
						<Icon icon="mdi:memory" class="w-6 h-6 text-purple-500" />
						<div class="stat-info">
							<span class="stat-label">{$t.serviceApp?.memory || 'Memory'}</span>
							<span class="stat-value">{formatBytes(status?.memory_usage ?? null)}</span>
						</div>
					</div>
					<div class="stat-card">
						<Icon icon="mdi:cpu-64-bit" class="w-6 h-6 text-green-500" />
						<div class="stat-info">
							<span class="stat-label">{$t.serviceApp?.cpu || 'CPU'}</span>
							<span class="stat-value">{status?.cpu_usage?.toFixed(1) ?? '-'}%</span>
						</div>
					</div>
					<div class="stat-card">
						<Icon icon="mdi:power" class="w-6 h-6 text-amber-500" />
						<div class="stat-info">
							<span class="stat-label">{$t.serviceApp?.autostart || 'Auto-start'}</span>
							<span class="stat-value">{status?.enabled ? ($t.common?.yes || 'Yes') : ($t.common?.no || 'No')}</span>
						</div>
					</div>
				</div>
			</div>

		{:else if activeTab === 'logs'}
			<div class="logs-view">
				<div class="logs-header">
					<h3>{$t.serviceApp?.recentLogs || 'Recent Logs'}</h3>
					<button class="refresh-btn" on:click={fetchLogs}>
						<Icon icon="mdi:refresh" class="w-4 h-4" />
					</button>
				</div>
				<div class="logs-container">
					{#if logs.length === 0}
						<div class="empty-logs">
							<Icon icon="mdi:text-box-outline" class="w-12 h-12 text-slate-300" />
							<p>{$t.serviceApp?.noLogs || 'No logs available'}</p>
						</div>
					{:else}
						{#each logs as log}
							<div class="log-entry {log.level}">
								<span class="log-time">{formatTime(log.timestamp)}</span>
								<span class="log-level">{log.level.toUpperCase()}</span>
								<span class="log-message">{log.message}</span>
							</div>
						{/each}
					{/if}
				</div>
			</div>

		{:else if activeTab === 'config'}
			<div class="config-view">
				<div class="placeholder">
					<Icon icon="mdi:cog" class="w-16 h-16 text-slate-300" />
					<p>{$t.serviceApp?.configPlaceholder || 'Configuration options coming soon'}</p>
				</div>
			</div>
		{/if}
	</main>
</div>

<style>
	.service-app {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #f8fafc;
	}

	/* Header */
	.header {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px 24px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.header-icon {
		width: 48px;
		height: 48px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.header-info {
		flex: 1;
	}

	.header-info h1 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
	}

	.header-info p {
		font-size: 14px;
		color: #64748b;
		margin-top: 2px;
	}

	.header-status {
		display: flex;
		align-items: center;
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border-radius: 20px;
		font-size: 13px;
		font-weight: 500;
	}

	.status-badge.loading {
		background: #f1f5f9;
		color: #64748b;
	}

	.status-badge.running {
		background: #dcfce7;
		color: #16a34a;
	}

	.status-badge.stopped {
		background: #fee2e2;
		color: #dc2626;
	}

	/* Tabs */
	.tabs {
		display: flex;
		gap: 4px;
		padding: 8px 24px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.tab {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: none;
		background: transparent;
		border-radius: 8px;
		font-size: 14px;
		color: #64748b;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.tab:hover {
		background: #f1f5f9;
		color: #334155;
	}

	.tab.active {
		background: #eff6ff;
		color: #2563eb;
	}

	/* Content */
	.content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	/* Status View */
	.status-view {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.actions-card {
		background: white;
		border-radius: 12px;
		padding: 20px;
	}

	.actions-card h3 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 16px;
	}

	.action-buttons {
		display: flex;
		gap: 12px;
	}

	.action-btn {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		border: 1px solid #e2e8f0;
		background: white;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: #334155;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover:not(:disabled) {
		background: #f8fafc;
		border-color: #cbd5e1;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.action-btn.primary {
		background: #3b82f6;
		border-color: #3b82f6;
		color: white;
	}

	.action-btn.primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.error-message {
		margin-top: 12px;
		color: #dc2626;
		font-size: 14px;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 16px;
	}

	.stat-card {
		display: flex;
		align-items: center;
		gap: 16px;
		background: white;
		border-radius: 12px;
		padding: 20px;
	}

	.stat-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.stat-label {
		font-size: 13px;
		color: #64748b;
	}

	.stat-value {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	/* Logs View */
	.logs-view {
		background: white;
		border-radius: 12px;
		overflow: hidden;
	}

	.logs-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid #e2e8f0;
	}

	.logs-header h3 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.refresh-btn {
		padding: 8px;
		border: none;
		background: #f1f5f9;
		border-radius: 6px;
		cursor: pointer;
		color: #64748b;
	}

	.refresh-btn:hover {
		background: #e2e8f0;
		color: #334155;
	}

	.logs-container {
		max-height: 400px;
		overflow-y: auto;
		font-family: monospace;
		font-size: 13px;
	}

	.log-entry {
		display: flex;
		gap: 12px;
		padding: 8px 20px;
		border-bottom: 1px solid #f8fafc;
	}

	.log-entry:hover {
		background: #f8fafc;
	}

	.log-time {
		color: #94a3b8;
		flex-shrink: 0;
	}

	.log-level {
		width: 50px;
		flex-shrink: 0;
		font-weight: 600;
	}

	.log-entry.info .log-level {
		color: #3b82f6;
	}

	.log-entry.warn .log-level {
		color: #f59e0b;
	}

	.log-entry.error .log-level {
		color: #ef4444;
	}

	.log-message {
		color: #334155;
		word-break: break-all;
	}

	.empty-logs {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px;
		gap: 12px;
	}

	.empty-logs p {
		color: #94a3b8;
		font-size: 14px;
	}

	/* Config View */
	.config-view {
		background: white;
		border-radius: 12px;
		padding: 24px;
	}

	.placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px;
		gap: 16px;
	}

	.placeholder p {
		color: #94a3b8;
		font-size: 14px;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	:global(.animate-spin) {
		animation: spin 1s linear infinite;
	}
</style>
