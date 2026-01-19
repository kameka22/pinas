<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';

	// Types
	interface DockerStats {
		running: boolean;
		version: string | null;
		containers_total: number;
		containers_running: number;
		containers_paused: number;
		containers_stopped: number;
		images: number;
		data_usage: number | null;
	}

	interface Container {
		id: string;
		name: string;
		image: string;
		status: string;
		state: string;
		created: number;
		ports: { host: number | null; container: number; protocol: string }[];
	}

	interface Image {
		id: string;
		repo_tags: string[];
		size: number;
		created: number;
	}

	// State
	let activeView = 'overview';
	let dockerStats: DockerStats | null = null;
	let containers: Container[] = [];
	let images: Image[] = [];
	let loading = true;
	let error: string | null = null;

	// System stats for widgets
	let cpuUsage = 0;
	let memoryUsage = 0;
	let memoryTotal = 0;
	let memoryAvailable = 0;

	const views = [
		{ id: 'overview', label: $t.docker.views.overview, icon: 'mdi:view-dashboard' },
		{ id: 'project', label: $t.docker.views.project, icon: 'mdi:folder-multiple' },
		{ id: 'container', label: $t.docker.views.container, icon: 'mdi:docker' },
		{ id: 'image', label: $t.docker.views.image, icon: 'mdi:circle-outline' },
		{ id: 'network', label: $t.docker.views.network, icon: 'mdi:web' },
		{ id: 'log', label: $t.docker.views.log, icon: 'mdi:text-box-outline' },
		{ id: 'management', label: $t.docker.views.management, icon: 'mdi:cog' }
	];

	// Fetch data
	async function fetchDockerStatus() {
		try {
			const response = await fetch('/api/docker/status');
			if (response.ok) {
				dockerStats = await response.json();
			}
		} catch (e) {
			console.error('Failed to fetch Docker status:', e);
		}
	}

	async function fetchContainers() {
		try {
			const response = await fetch('/api/docker/containers?all=true');
			if (response.ok) {
				containers = await response.json();
			}
		} catch (e) {
			console.error('Failed to fetch containers:', e);
		}
	}

	async function fetchImages() {
		try {
			const response = await fetch('/api/docker/images');
			if (response.ok) {
				images = await response.json();
			}
		} catch (e) {
			console.error('Failed to fetch images:', e);
		}
	}

	async function fetchSystemStats() {
		try {
			const response = await fetch('/api/system/info');
			if (response.ok) {
				const data = await response.json();
				cpuUsage = Math.round(data.cpu.usage);
				memoryUsage = Math.round(data.memory.usage_percent);
				memoryTotal = data.memory.total;
				memoryAvailable = data.memory.available;
			}
		} catch (e) {
			console.error('Failed to fetch system stats:', e);
		}
	}

	async function loadData() {
		loading = true;
		error = null;
		await Promise.all([
			fetchDockerStatus(),
			fetchContainers(),
			fetchImages(),
			fetchSystemStats()
		]);
		loading = false;
	}

	// Container actions
	async function startContainer(id: string) {
		await fetch(`/api/docker/containers/${id}/start`, { method: 'POST' });
		await fetchContainers();
	}

	async function stopContainer(id: string) {
		await fetch(`/api/docker/containers/${id}/stop`, { method: 'POST' });
		await fetchContainers();
	}

	async function restartContainer(id: string) {
		await fetch(`/api/docker/containers/${id}/restart`, { method: 'POST' });
		await fetchContainers();
	}

	async function removeContainer(id: string) {
		await fetch(`/api/docker/containers/${id}?force=true`, { method: 'DELETE' });
		await fetchContainers();
	}

	// Formatting
	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	// Lifecycle
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		loadData();
		refreshInterval = setInterval(loadData, 10000); // Refresh every 10s
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="docker-app">
	<!-- Sidebar -->
	<aside class="sidebar">
		<nav class="sidebar-nav">
			{#each views as view}
				<button
					class="nav-item"
					class:active={activeView === view.id}
					on:click={() => (activeView = view.id)}
				>
					<Icon icon={view.icon} class="w-5 h-5" />
					<span>{view.label}</span>
				</button>
			{/each}
		</nav>
	</aside>

	<!-- Main Content -->
	<main class="main-content">
		{#if loading && !dockerStats}
			<div class="loading">
				<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
			</div>
		{:else if activeView === 'overview'}
			<!-- Overview -->
			<div class="overview">
				<!-- Status Header -->
				<div class="status-card">
					<div class="status-info">
						<h1>Docker</h1>
						<p class="status-line">
							{$t.docker.serviceStatus}
							{#if dockerStats?.running}
								<span class="status-badge running">{$t.docker.status.normal}</span>
							{:else}
								<span class="status-badge stopped">{$t.docker.status.stopped}</span>
							{/if}
						</p>

						<div class="stats-row">
							<div class="stat">
								<span class="stat-label">{$t.docker.stats.projects}</span>
								<span class="stat-value">0/0</span>
							</div>
							<div class="stat">
								<span class="stat-label">{$t.docker.stats.containers}</span>
								<span class="stat-value">{dockerStats?.containers_running || 0}/{dockerStats?.containers_total || 0}</span>
							</div>
							<div class="stat">
								<span class="stat-label">{$t.docker.stats.local}</span>
								<span class="stat-value">{dockerStats?.images || 0}</span>
							</div>
							<div class="stat">
								<span class="stat-label">{$t.docker.stats.data}</span>
								<span class="stat-value">{dockerStats?.data_usage ? formatBytes(dockerStats.data_usage) : '-'}</span>
							</div>
						</div>
					</div>
					<div class="status-illustration">
						<Icon icon="mdi:docker" class="w-24 h-24 text-blue-400" />
					</div>
				</div>

				<!-- System Gauges -->
				<div class="gauges-row">
					<div class="gauge-card">
						<h3>{$t.docker.cpuUsage}</h3>
						<div class="gauge">
							<svg viewBox="0 0 100 100" class="gauge-svg">
								<circle cx="50" cy="50" r="40" fill="none" stroke="#e5e7eb" stroke-width="8" />
								<circle
									cx="50"
									cy="50"
									r="40"
									fill="none"
									stroke="#3b82f6"
									stroke-width="8"
									stroke-linecap="round"
									stroke-dasharray="{cpuUsage * 2.51} 251"
									transform="rotate(-90 50 50)"
								/>
							</svg>
							<span class="gauge-value">{cpuUsage}%</span>
						</div>
						<p class="gauge-label">Total <strong>{cpuUsage}%</strong></p>
					</div>

					<div class="gauge-card">
						<h3>{$t.docker.memoryCapacity}</h3>
						<div class="gauge">
							<svg viewBox="0 0 100 100" class="gauge-svg">
								<circle cx="50" cy="50" r="40" fill="none" stroke="#e5e7eb" stroke-width="8" />
								<circle
									cx="50"
									cy="50"
									r="40"
									fill="none"
									stroke="#a855f7"
									stroke-width="8"
									stroke-linecap="round"
									stroke-dasharray="{memoryUsage * 2.51} 251"
									transform="rotate(-90 50 50)"
								/>
							</svg>
							<span class="gauge-value">{memoryUsage}%</span>
						</div>
						<p class="gauge-label">
							{$t.docker.available} <strong>{formatBytes(memoryAvailable)}</strong><br />
							Total <strong>{formatBytes(memoryTotal)}</strong>
						</p>
					</div>
				</div>
			</div>

		{:else if activeView === 'container'}
			<!-- Containers List -->
			<div class="list-view">
				<div class="list-header">
					<h2>{$t.docker.views.container}</h2>
					<button class="btn-refresh" on:click={fetchContainers}>
						<Icon icon="mdi:refresh" class="w-4 h-4" />
					</button>
				</div>

				{#if containers.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:docker" class="w-16 h-16 text-slate-300" />
						<p>{$t.docker.noContainers}</p>
					</div>
				{:else}
					<table class="data-table">
						<thead>
							<tr>
								<th>{$t.docker.table.name}</th>
								<th>{$t.docker.table.image}</th>
								<th>{$t.docker.table.status}</th>
								<th>{$t.docker.table.ports}</th>
								<th>{$t.docker.table.actions}</th>
							</tr>
						</thead>
						<tbody>
							{#each containers as container}
								<tr>
									<td class="name-cell">
										<Icon icon="mdi:docker" class="w-5 h-5 text-blue-500" />
										<span>{container.name}</span>
									</td>
									<td>{container.image}</td>
									<td>
										<span class="status-badge" class:running={container.state === 'running'} class:stopped={container.state !== 'running'}>
											{container.status}
										</span>
									</td>
									<td>
										{#each container.ports as port}
											{#if port.host}
												<span class="port-badge">{port.host}:{port.container}</span>
											{/if}
										{/each}
									</td>
									<td class="actions-cell">
										{#if container.state === 'running'}
											<button class="action-btn" title="Stop" on:click={() => stopContainer(container.id)}>
												<Icon icon="mdi:stop" class="w-4 h-4" />
											</button>
											<button class="action-btn" title="Restart" on:click={() => restartContainer(container.id)}>
												<Icon icon="mdi:restart" class="w-4 h-4" />
											</button>
										{:else}
											<button class="action-btn" title="Start" on:click={() => startContainer(container.id)}>
												<Icon icon="mdi:play" class="w-4 h-4" />
											</button>
										{/if}
										<button class="action-btn danger" title="Remove" on:click={() => removeContainer(container.id)}>
											<Icon icon="mdi:delete" class="w-4 h-4" />
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/if}
			</div>

		{:else if activeView === 'image'}
			<!-- Images List -->
			<div class="list-view">
				<div class="list-header">
					<h2>{$t.docker.views.image}</h2>
					<button class="btn-refresh" on:click={fetchImages}>
						<Icon icon="mdi:refresh" class="w-4 h-4" />
					</button>
				</div>

				{#if images.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:image-multiple" class="w-16 h-16 text-slate-300" />
						<p>{$t.docker.noImages}</p>
					</div>
				{:else}
					<table class="data-table">
						<thead>
							<tr>
								<th>{$t.docker.table.repository}</th>
								<th>{$t.docker.table.tag}</th>
								<th>{$t.docker.table.imageId}</th>
								<th>{$t.docker.table.size}</th>
								<th>{$t.docker.table.created}</th>
							</tr>
						</thead>
						<tbody>
							{#each images as image}
								<tr>
									<td>{image.repo_tags[0]?.split(':')[0] || '<none>'}</td>
									<td>{image.repo_tags[0]?.split(':')[1] || '<none>'}</td>
									<td class="mono">{image.id}</td>
									<td>{formatBytes(image.size)}</td>
									<td>{formatDate(image.created)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/if}
			</div>

		{:else}
			<!-- Placeholder for other views -->
			<div class="placeholder-view">
				<Icon icon="mdi:construction" class="w-16 h-16 text-slate-300" />
				<p>{$t.docker.underDevelopment}</p>
			</div>
		{/if}
	</main>
</div>

<style>
	.docker-app {
		display: flex;
		height: 100%;
		background: #f8fafc;
	}

	/* Sidebar */
	.sidebar {
		width: 180px;
		background: white;
		border-right: 1px solid #e2e8f0;
	}

	.sidebar-nav {
		padding: 12px 8px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 12px;
		border: none;
		background: transparent;
		border-radius: 8px;
		color: #64748b;
		font-size: 14px;
		cursor: pointer;
		transition: all 0.15s ease;
		text-align: left;
	}

	.nav-item:hover {
		background: #f1f5f9;
		color: #334155;
	}

	.nav-item.active {
		background: #eff6ff;
		color: #2563eb;
	}

	/* Main Content */
	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	/* Overview */
	.overview {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.status-card {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		background: white;
		border-radius: 16px;
		padding: 24px;
	}

	.status-info h1 {
		font-size: 24px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 8px;
	}

	.status-line {
		font-size: 16px;
		color: #64748b;
		margin-bottom: 24px;
	}

	.status-badge {
		display: inline-block;
		padding: 2px 12px;
		border-radius: 12px;
		font-size: 14px;
		font-weight: 500;
		margin-left: 8px;
	}

	.status-badge.running {
		background: #dcfce7;
		color: #16a34a;
	}

	.status-badge.stopped {
		background: #fee2e2;
		color: #dc2626;
	}

	.stats-row {
		display: flex;
		gap: 32px;
	}

	.stat {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.stat-label {
		font-size: 13px;
		color: #94a3b8;
	}

	.stat-value {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
	}

	.status-illustration {
		opacity: 0.5;
	}

	/* Gauges */
	.gauges-row {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 24px;
	}

	.gauge-card {
		background: white;
		border-radius: 16px;
		padding: 24px;
	}

	.gauge-card h3 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 16px;
	}

	.gauge {
		position: relative;
		width: 120px;
		height: 120px;
		margin: 0 auto 16px;
	}

	.gauge-svg {
		width: 100%;
		height: 100%;
	}

	.gauge-value {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		font-size: 24px;
		font-weight: 600;
		color: #1e293b;
	}

	.gauge-label {
		text-align: center;
		font-size: 13px;
		color: #64748b;
	}

	.gauge-label strong {
		color: #1e293b;
	}

	/* List Views */
	.list-view {
		background: white;
		border-radius: 16px;
		padding: 24px;
	}

	.list-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 20px;
	}

	.list-header h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.btn-refresh {
		padding: 8px;
		border: none;
		background: #f1f5f9;
		border-radius: 8px;
		cursor: pointer;
		color: #64748b;
	}

	.btn-refresh:hover {
		background: #e2e8f0;
		color: #334155;
	}

	/* Table */
	.data-table {
		width: 100%;
		border-collapse: collapse;
	}

	.data-table th,
	.data-table td {
		padding: 12px 16px;
		text-align: left;
		border-bottom: 1px solid #f1f5f9;
	}

	.data-table th {
		font-size: 12px;
		font-weight: 600;
		color: #64748b;
		text-transform: uppercase;
	}

	.data-table td {
		font-size: 14px;
		color: #334155;
	}

	.name-cell {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.mono {
		font-family: monospace;
		font-size: 13px;
	}

	.port-badge {
		display: inline-block;
		padding: 2px 8px;
		background: #f1f5f9;
		border-radius: 4px;
		font-size: 12px;
		font-family: monospace;
		margin-right: 4px;
	}

	.actions-cell {
		display: flex;
		gap: 4px;
	}

	.action-btn {
		padding: 6px;
		border: none;
		background: #f1f5f9;
		border-radius: 6px;
		cursor: pointer;
		color: #64748b;
	}

	.action-btn:hover {
		background: #e2e8f0;
		color: #334155;
	}

	.action-btn.danger:hover {
		background: #fee2e2;
		color: #dc2626;
	}

	/* Empty State */
	.empty-state,
	.placeholder-view {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px;
		gap: 16px;
	}

	.empty-state p,
	.placeholder-view p {
		font-size: 14px;
		color: #94a3b8;
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
