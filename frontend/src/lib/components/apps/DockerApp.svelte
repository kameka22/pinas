<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { t } from '$lib/i18n';
	import ConfirmModal from '$lib/components/ui/ConfirmModal.svelte';
	import LogViewer from '$lib/components/ui/LogViewer.svelte';

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

	interface Volume {
		name: string;
		driver: string;
		mount_point: string;
		created: string;
	}

	interface Network {
		id: string;
		name: string;
		driver: string;
		scope: string;
		containers: string[];
	}

	// State
	let activeView = 'overview';
	let dockerStats: DockerStats | null = null;
	let containers: Container[] = [];
	let images: Image[] = [];
	let volumes: Volume[] = [];
	let networks: Network[] = [];
	let loading = true;

	// Search
	let searchContainer = '';
	let searchImage = '';
	let searchVolume = '';
	let searchNetwork = '';

	// Filtered lists
	$: filteredContainers = containers.filter(c => {
		if (!searchContainer) return true;
		const q = searchContainer.toLowerCase();
		return c.name.toLowerCase().includes(q) || c.image.toLowerCase().includes(q);
	});

	$: filteredImages = images.filter(img => {
		if (!searchImage) return true;
		const q = searchImage.toLowerCase();
		return (img.repo_tags[0] || '').toLowerCase().includes(q) || img.id.toLowerCase().includes(q);
	});

	$: filteredVolumes = volumes.filter(v => {
		if (!searchVolume) return true;
		const q = searchVolume.toLowerCase();
		return v.name.toLowerCase().includes(q) || v.driver.toLowerCase().includes(q);
	});

	$: filteredNetworks = networks.filter(n => {
		if (!searchNetwork) return true;
		const q = searchNetwork.toLowerCase();
		return n.name.toLowerCase().includes(q) || n.driver.toLowerCase().includes(q);
	});

	// System stats
	let cpuUsage = 0;
	let memoryUsage = 0;
	let memoryTotal = 0;
	let memoryAvailable = 0;

	// Confirm modal state
	let confirmVisible = false;
	let confirmTitle = '';
	let confirmMessage = '';
	let confirmLabel = '';
	let confirmDanger = false;
	let confirmLoading = false;
	let confirmAction: (() => Promise<void>) | null = null;

	// Log viewer state
	let logVisible = false;
	let logTitle = '';
	let logLines: string[] = [];
	let logLoading = false;
	let logContainerId = '';

	// Pull image state
	let pullInput = '';
	let pulling = false;

	const BUILTIN_NETWORKS = ['bridge', 'host', 'none'];

	// Safe i18n accessors (avoids SSR undefined errors)
	$: d = $t.docker || {} as any;
	$: dc = d.confirm || {} as any;
	$: dl = d.logs || {} as any;
	$: dv = d.volumes || {} as any;
	$: dn = d.networks || {} as any;
	$: dp = d.pull || {} as any;
	$: dt = d.table || {} as any;
	$: ds = d.stats || {} as any;
	$: dw = d.views || {} as any;
	$: dst = d.status || {} as any;

	// Reactive views (updates when locale changes)
	$: views = [
		{ id: 'overview', label: dw.overview || 'Overview', icon: 'mdi:view-dashboard' },
		{ id: 'container', label: dw.container || 'Containers', icon: 'mdi:docker' },
		{ id: 'image', label: dw.image || 'Images', icon: 'mdi:layers' },
		{ id: 'volume', label: dw.volume || 'Volumes', icon: 'mdi:database' },
		{ id: 'network', label: dw.network || 'Networks', icon: 'mdi:web' }
	];

	// Fetch data
	async function fetchDockerStatus() {
		try {
			const response = await fetch('/api/docker/status');
			if (response.ok) dockerStats = await response.json();
		} catch (e) {
			console.error('Failed to fetch Docker status:', e);
		}
	}

	async function fetchContainers() {
		try {
			const response = await fetch('/api/docker/containers?all=true');
			if (response.ok) containers = await response.json();
		} catch (e) {
			console.error('Failed to fetch containers:', e);
		}
	}

	async function fetchImages() {
		try {
			const response = await fetch('/api/docker/images');
			if (response.ok) images = await response.json();
		} catch (e) {
			console.error('Failed to fetch images:', e);
		}
	}

	async function fetchVolumes() {
		try {
			const response = await fetch('/api/docker/volumes');
			if (response.ok) volumes = await response.json();
		} catch (e) {
			console.error('Failed to fetch volumes:', e);
		}
	}

	async function fetchNetworks() {
		try {
			const response = await fetch('/api/docker/networks');
			if (response.ok) networks = await response.json();
		} catch (e) {
			console.error('Failed to fetch networks:', e);
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
		await Promise.all([
			fetchDockerStatus(),
			fetchContainers(),
			fetchImages(),
			fetchVolumes(),
			fetchNetworks(),
			fetchSystemStats()
		]);
		loading = false;
	}

	// Confirmation helpers
	function showConfirm(title: string, message: string, label: string, danger: boolean, action: () => Promise<void>) {
		confirmTitle = title;
		confirmMessage = message;
		confirmLabel = label;
		confirmDanger = danger;
		confirmAction = action;
		confirmVisible = true;
	}

	async function handleConfirm() {
		if (!confirmAction) return;
		confirmLoading = true;
		try {
			await confirmAction();
		} finally {
			confirmLoading = false;
			confirmVisible = false;
			confirmAction = null;
		}
	}

	function handleConfirmCancel() {
		confirmVisible = false;
		confirmAction = null;
	}

	// Container actions
	function confirmStopContainer(c: Container) {
		showConfirm(
			dc.stopTitle || 'Stop Container',
			(dc.stopMessage || 'Are you sure you want to stop "{name}"?').replace('{name}', c.name),
			dc.stopBtn || 'Stop', false, async () => {
				await fetch(`/api/docker/containers/${c.id}/stop`, { method: 'POST' });
				await fetchContainers();
			}
		);
	}

	function confirmStartContainer(c: Container) {
		showConfirm(
			dc.startTitle || 'Start Container',
			(dc.startMessage || 'Are you sure you want to start "{name}"?').replace('{name}', c.name),
			dc.startBtn || 'Start', false, async () => {
				await fetch(`/api/docker/containers/${c.id}/start`, { method: 'POST' });
				await fetchContainers();
			}
		);
	}

	function confirmRestartContainer(c: Container) {
		showConfirm(
			dc.restartTitle || 'Restart Container',
			(dc.restartMessage || 'Are you sure you want to restart "{name}"?').replace('{name}', c.name),
			dc.restartBtn || 'Restart', false, async () => {
				await fetch(`/api/docker/containers/${c.id}/restart`, { method: 'POST' });
				await fetchContainers();
			}
		);
	}

	function confirmRemoveContainer(c: Container) {
		showConfirm(
			dc.removeContainerTitle || 'Remove Container',
			(dc.removeContainerMessage || 'Remove container "{name}"? This cannot be undone.').replace('{name}', c.name),
			dc.removeBtn || 'Remove', true, async () => {
				await fetch(`/api/docker/containers/${c.id}?force=true`, { method: 'DELETE' });
				await fetchContainers();
			}
		);
	}

	function confirmRemoveImage(img: Image) {
		const name = img.repo_tags[0] || img.id;
		showConfirm(
			dc.removeImageTitle || 'Remove Image',
			(dc.removeImageMessage || 'Remove image "{name}"? This cannot be undone.').replace('{name}', name),
			dc.removeBtn || 'Remove', true, async () => {
				await fetch(`/api/docker/images/${encodeURIComponent(img.id)}?force=true`, { method: 'DELETE' });
				await fetchImages();
			}
		);
	}

	function confirmRemoveVolume(vol: Volume) {
		showConfirm(
			dc.removeVolumeTitle || 'Remove Volume',
			(dc.removeVolumeMessage || 'Remove volume "{name}"? All data will be lost.').replace('{name}', vol.name),
			dc.removeBtn || 'Remove', true, async () => {
				await fetch(`/api/docker/volumes/${encodeURIComponent(vol.name)}?force=true`, { method: 'DELETE' });
				await fetchVolumes();
			}
		);
	}

	function confirmRemoveNetwork(net: Network) {
		showConfirm(
			dc.removeNetworkTitle || 'Remove Network',
			(dc.removeNetworkMessage || 'Remove network "{name}"?').replace('{name}', net.name),
			dc.removeBtn || 'Remove', true, async () => {
				await fetch(`/api/docker/networks/${encodeURIComponent(net.id)}`, { method: 'DELETE' });
				await fetchNetworks();
			}
		);
	}

	// Prune
	function confirmPruneImages() {
		showConfirm(
			dc.pruneImagesTitle || 'Prune Images',
			dc.pruneImagesMessage || 'This will remove all unused images. This cannot be undone.',
			dc.pruneImagesBtn || 'Prune', true, async () => {
				await fetch('/api/docker/images/prune', { method: 'POST' });
				await fetchImages();
			}
		);
	}

	function confirmPruneVolumes() {
		showConfirm(
			dc.pruneVolumesTitle || 'Prune Volumes',
			dc.pruneVolumesMessage || 'This will remove all unused volumes. All data will be lost.',
			dc.pruneVolumesBtn || 'Prune', true, async () => {
				await fetch('/api/docker/volumes/prune', { method: 'POST' });
				await fetchVolumes();
			}
		);
	}

	// Logs
	async function openLogs(container: Container) {
		logContainerId = container.id;
		logTitle = `${dl.title || 'Logs'} - ${container.name}`;
		logLines = [];
		logVisible = true;
		await fetchLogs(100);
	}

	async function fetchLogs(tail: number) {
		logLoading = true;
		try {
			const response = await fetch(`/api/docker/containers/${logContainerId}/logs?tail=${tail}`);
			if (response.ok) logLines = await response.json();
		} catch (e) {
			console.error('Failed to fetch logs:', e);
		} finally {
			logLoading = false;
		}
	}

	function handleLogRefresh(event: CustomEvent<{ tail: number }>) {
		fetchLogs(event.detail.tail);
	}

	function closeLogs() {
		logVisible = false;
		logLines = [];
		logContainerId = '';
	}

	// Pull image
	async function pullImage() {
		if (!pullInput.trim()) return;
		pulling = true;
		try {
			await fetch('/api/docker/images/pull', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ image: pullInput.trim() })
			});
			pullInput = '';
			await fetchImages();
		} catch (e) {
			console.error('Failed to pull image:', e);
		} finally {
			pulling = false;
		}
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

	function formatDateString(dateStr: string): string {
		if (!dateStr) return '-';
		try {
			return new Date(dateStr).toLocaleDateString();
		} catch {
			return dateStr;
		}
	}

	// Lifecycle
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		loadData();
		refreshInterval = setInterval(loadData, 10000);
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
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
			</div>

		{:else if activeView === 'overview'}
			<!-- ==================== OVERVIEW ==================== -->
			<div class="overview">
				<div class="status-card">
					<div class="status-info">
						<h1>Docker</h1>
						<p class="status-line">
							{d.serviceStatus || 'Service'}
							{#if dockerStats?.running}
								<span class="badge badge-green">{dst.normal || 'Normal'}</span>
							{:else}
								<span class="badge badge-red">{dst.stopped || 'Stopped'}</span>
							{/if}
						</p>
						<div class="stats-row">
							<div class="stat">
								<span class="stat-label">{ds.containers || 'Containers'}</span>
								<span class="stat-value">{dockerStats?.containers_running || 0}/{dockerStats?.containers_total || 0}</span>
							</div>
							<div class="stat">
								<span class="stat-label">{ds.images || 'Images'}</span>
								<span class="stat-value">{dockerStats?.images || 0}</span>
							</div>
							<div class="stat">
								<span class="stat-label">{ds.volumes || 'Volumes'}</span>
								<span class="stat-value">{volumes.length}</span>
							</div>
							<div class="stat">
								<span class="stat-label">{ds.networks || 'Networks'}</span>
								<span class="stat-value">{networks.length}</span>
							</div>
						</div>
					</div>
					<div class="status-illustration">
						<Icon icon="mdi:docker" class="w-24 h-24 text-blue-400" />
					</div>
				</div>

				<div class="gauges-row">
					<div class="gauge-card">
						<h3>{d.cpuUsage || 'CPU'}</h3>
						<div class="gauge">
							<svg viewBox="0 0 100 100" class="gauge-svg">
								<circle cx="50" cy="50" r="40" fill="none" stroke="#e5e7eb" stroke-width="8" />
								<circle cx="50" cy="50" r="40" fill="none" stroke="#3b82f6" stroke-width="8"
									stroke-linecap="round" stroke-dasharray="{cpuUsage * 2.51} 251" transform="rotate(-90 50 50)" />
							</svg>
							<span class="gauge-value">{cpuUsage}%</span>
						</div>
						<p class="gauge-label">Total <strong>{cpuUsage}%</strong></p>
					</div>
					<div class="gauge-card">
						<h3>{d.memoryCapacity || 'Memory'}</h3>
						<div class="gauge">
							<svg viewBox="0 0 100 100" class="gauge-svg">
								<circle cx="50" cy="50" r="40" fill="none" stroke="#e5e7eb" stroke-width="8" />
								<circle cx="50" cy="50" r="40" fill="none" stroke="#a855f7" stroke-width="8"
									stroke-linecap="round" stroke-dasharray="{memoryUsage * 2.51} 251" transform="rotate(-90 50 50)" />
							</svg>
							<span class="gauge-value">{memoryUsage}%</span>
						</div>
						<p class="gauge-label">
							{d.available || 'Available'} <strong>{formatBytes(memoryAvailable)}</strong><br />
							Total <strong>{formatBytes(memoryTotal)}</strong>
						</p>
					</div>
				</div>
			</div>

		{:else if activeView === 'container'}
			<!-- ==================== CONTAINERS ==================== -->
			<div class="panel">
				<div class="panel-header">
					<h2>{dw.container || 'Containers'}</h2>
					<div class="header-right">
						<div class="search-box">
							<Icon icon="mdi:magnify" class="w-4 h-4" />
							<input type="text" bind:value={searchContainer} placeholder={d.search || 'Search...'} />
						</div>
						<button class="btn-icon" on:click={fetchContainers} title="Refresh">
							<Icon icon="mdi:refresh" class="w-4 h-4" />
						</button>
					</div>
				</div>

				{#if filteredContainers.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:docker" class="w-12 h-12 text-slate-300" />
						<p>{searchContainer ? (d.noResults || 'No results') : (d.noContainers || 'No containers')}</p>
					</div>
				{:else}
					<div class="item-list">
						{#each filteredContainers as c}
							<div class="item-row grid-container">
								<div class="item-info">
									<div class="status-dot" class:dot-running={c.state === 'running'} class:dot-stopped={c.state !== 'running'}></div>
									<div class="item-details">
										<div class="item-name-line">
											<span class="item-name">{c.name}</span>
											<span class="badge" class:badge-green={c.state === 'running'} class:badge-red={c.state !== 'running'}>
												{c.status}
											</span>
										</div>
										<span class="item-sub">{c.image}</span>
									</div>
								</div>
								<div class="item-tags">
									{#each c.ports as port}
										{#if port.host}
											<span class="port-tag">{port.host}:{port.container}</span>
										{/if}
									{/each}
								</div>
								<div class="item-actions">
									{#if c.state === 'running'}
										<button class="btn-action" title={dc.stopBtn || 'Stop'} on:click={() => confirmStopContainer(c)}>
											<Icon icon="mdi:stop" class="w-4 h-4" />
										</button>
										<button class="btn-action" title={dc.restartBtn || 'Restart'} on:click={() => confirmRestartContainer(c)}>
											<Icon icon="mdi:restart" class="w-4 h-4" />
										</button>
									{:else}
										<button class="btn-action btn-action-success" title={dc.startBtn || 'Start'} on:click={() => confirmStartContainer(c)}>
											<Icon icon="mdi:play" class="w-4 h-4" />
										</button>
										<button class="btn-action btn-action-danger" title={dc.removeBtn || 'Remove'} on:click={() => confirmRemoveContainer(c)}>
											<Icon icon="mdi:delete" class="w-4 h-4" />
										</button>
									{/if}
									<button class="btn-action" title={dl.title || 'Logs'} on:click={() => openLogs(c)}>
										<Icon icon="mdi:text-box-outline" class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		{:else if activeView === 'image'}
			<!-- ==================== IMAGES ==================== -->
			<div class="panel">
				<div class="panel-header">
					<h2>{dw.image || 'Images'}</h2>
					<div class="header-right">
						<div class="search-box">
							<Icon icon="mdi:magnify" class="w-4 h-4" />
							<input type="text" bind:value={searchImage} placeholder={d.search || 'Search...'} />
						</div>
						<button class="btn-prune" on:click={confirmPruneImages} title={dc.pruneImagesTitle || 'Prune'}>
							<Icon icon="mdi:broom" class="w-4 h-4" />
							<span>{dc.pruneImagesBtn || 'Prune'}</span>
						</button>
						<button class="btn-icon" on:click={fetchImages} title="Refresh">
							<Icon icon="mdi:refresh" class="w-4 h-4" />
						</button>
					</div>
				</div>

				<div class="pull-bar">
					<input
						type="text"
						bind:value={pullInput}
						placeholder={dp.placeholder || 'nginx:latest'}
						disabled={pulling}
						on:keydown={(e) => e.key === 'Enter' && pullImage()}
					/>
					<button class="btn-pull" on:click={pullImage} disabled={pulling || !pullInput.trim()}>
						{#if pulling}
							<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
							<span>{dp.pulling || 'Pulling...'}</span>
						{:else}
							<Icon icon="mdi:download" class="w-4 h-4" />
							<span>{dp.button || 'Pull'}</span>
						{/if}
					</button>
				</div>

				{#if filteredImages.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:layers" class="w-12 h-12 text-slate-300" />
						<p>{searchImage ? (d.noResults || 'No results') : (d.noImages || 'No images')}</p>
					</div>
				{:else}
					<div class="item-list">
						{#each filteredImages as img}
							<div class="item-row grid-image">
								<div class="item-info">
									<div class="item-details">
										<div class="item-name-line">
											<Icon icon="mdi:layers" class="w-4 h-4 text-orange-500" />
											<span class="item-name">{img.repo_tags[0]?.split(':')[0] || '<none>'}</span>
											<span class="badge badge-outline">{img.repo_tags[0]?.split(':')[1] || 'latest'}</span>
										</div>
										<span class="item-sub">{img.id.substring(0, 20)}</span>
									</div>
								</div>
								<div class="item-meta">
									<span class="meta-value">{formatBytes(img.size)}</span>
								</div>
								<div class="item-meta">
									<span class="meta-value">{formatDate(img.created)}</span>
								</div>
								<div class="item-actions">
									<button class="btn-action btn-action-danger" title={dc.removeBtn || 'Remove'} on:click={() => confirmRemoveImage(img)}>
										<Icon icon="mdi:delete" class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		{:else if activeView === 'volume'}
			<!-- ==================== VOLUMES ==================== -->
			<div class="panel">
				<div class="panel-header">
					<h2>{dv.title || 'Volumes'}</h2>
					<div class="header-right">
						<div class="search-box">
							<Icon icon="mdi:magnify" class="w-4 h-4" />
							<input type="text" bind:value={searchVolume} placeholder={d.search || 'Search...'} />
						</div>
						<button class="btn-prune" on:click={confirmPruneVolumes} title={dc.pruneVolumesTitle || 'Prune'}>
							<Icon icon="mdi:broom" class="w-4 h-4" />
							<span>{dc.pruneVolumesBtn || 'Prune'}</span>
						</button>
						<button class="btn-icon" on:click={fetchVolumes} title="Refresh">
							<Icon icon="mdi:refresh" class="w-4 h-4" />
						</button>
					</div>
				</div>

				{#if filteredVolumes.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:database" class="w-12 h-12 text-slate-300" />
						<p>{searchVolume ? (d.noResults || 'No results') : (dv.noVolumes || 'No volumes')}</p>
					</div>
				{:else}
					<div class="item-list">
						{#each filteredVolumes as vol}
							<div class="item-row grid-volume">
								<div class="item-info">
									<div class="item-details">
										<div class="item-name-line">
											<Icon icon="mdi:database" class="w-4 h-4 text-purple-500" />
											<span class="item-name" title={vol.name}>{vol.name.length > 40 ? vol.name.substring(0, 40) + '...' : vol.name}</span>
										</div>
										<span class="item-sub">{vol.mount_point}</span>
									</div>
								</div>
								<div class="item-meta">
									<span class="meta-label">{dt.driver || 'Driver'}</span>
									<span class="meta-value">{vol.driver}</span>
								</div>
								<div class="item-meta">
									<span class="meta-value">{formatDateString(vol.created)}</span>
								</div>
								<div class="item-actions">
									<button class="btn-action btn-action-danger" title={dc.removeBtn || 'Remove'} on:click={() => confirmRemoveVolume(vol)}>
										<Icon icon="mdi:delete" class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		{:else if activeView === 'network'}
			<!-- ==================== NETWORKS ==================== -->
			<div class="panel">
				<div class="panel-header">
					<h2>{dn.title || 'Networks'}</h2>
					<div class="header-right">
						<div class="search-box">
							<Icon icon="mdi:magnify" class="w-4 h-4" />
							<input type="text" bind:value={searchNetwork} placeholder={d.search || 'Search...'} />
						</div>
						<button class="btn-icon" on:click={fetchNetworks} title="Refresh">
							<Icon icon="mdi:refresh" class="w-4 h-4" />
						</button>
					</div>
				</div>

				{#if filteredNetworks.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:web" class="w-12 h-12 text-slate-300" />
						<p>{searchNetwork ? (d.noResults || 'No results') : (dn.noNetworks || 'No networks')}</p>
					</div>
				{:else}
					<div class="item-list">
						{#each filteredNetworks as net}
							<div class="item-row grid-network">
								<div class="item-info">
									<div class="item-details">
										<div class="item-name-line">
											<Icon icon="mdi:web" class="w-4 h-4 text-teal-500" />
											<span class="item-name">{net.name}</span>
											{#if BUILTIN_NETWORKS.includes(net.name)}
												<span class="badge badge-blue">{dn.builtIn || 'built-in'}</span>
											{/if}
										</div>
										<span class="item-sub">{net.id.substring(0, 16)}</span>
									</div>
								</div>
								<div class="item-meta">
									<span class="meta-label">{dt.driver || 'Driver'}</span>
									<span class="meta-value">{net.driver}</span>
								</div>
								<div class="item-meta">
									<span class="meta-label">{dt.scope || 'Scope'}</span>
									<span class="meta-value">{net.scope}</span>
								</div>
								<div class="item-actions">
									<button
										class="btn-action btn-action-danger"
										title={dc.removeBtn || 'Remove'}
										disabled={BUILTIN_NETWORKS.includes(net.name)}
										on:click={() => confirmRemoveNetwork(net)}
									>
										<Icon icon="mdi:delete" class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</main>
</div>

<ConfirmModal
	visible={confirmVisible}
	title={confirmTitle}
	message={confirmMessage}
	confirmLabel={confirmLabel}
	danger={confirmDanger}
	loading={confirmLoading}
	on:confirm={handleConfirm}
	on:cancel={handleConfirmCancel}
/>

<LogViewer
	visible={logVisible}
	title={logTitle}
	logs={logLines}
	loading={logLoading}
	on:refresh={handleLogRefresh}
	on:close={closeLogs}
/>

<style>
	/* ===== LAYOUT ===== */
	.docker-app {
		display: flex;
		height: 100%;
		background: #f8fafc;
	}

	.sidebar {
		width: 180px;
		background: white;
		border-right: 1px solid #e2e8f0;
		flex-shrink: 0;
	}

	.sidebar-nav {
		padding: 12px 8px;
		display: flex;
		flex-direction: column;
		gap: 2px;
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

	.nav-item:hover { background: #f1f5f9; color: #334155; }
	.nav-item.active { background: #eff6ff; color: #2563eb; font-weight: 500; }

	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
		min-width: 0;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	/* ===== OVERVIEW ===== */
	.overview { display: flex; flex-direction: column; gap: 24px; }

	.status-card {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		background: white;
		border-radius: 12px;
		padding: 24px;
		box-shadow: 0 1px 3px rgba(0,0,0,0.04);
	}

	.status-info h1 { font-size: 24px; font-weight: 600; color: #1e293b; margin-bottom: 8px; }
	.status-line { font-size: 16px; color: #64748b; margin-bottom: 24px; }
	.status-illustration { opacity: 0.4; }

	.stats-row { display: flex; gap: 32px; }
	.stat { display: flex; flex-direction: column; gap: 4px; }
	.stat-label { font-size: 13px; color: #94a3b8; }
	.stat-value { font-size: 20px; font-weight: 600; color: #1e293b; }

	.gauges-row { display: grid; grid-template-columns: repeat(2, 1fr); gap: 24px; }

	.gauge-card {
		background: white;
		border-radius: 12px;
		padding: 24px;
		box-shadow: 0 1px 3px rgba(0,0,0,0.04);
	}

	.gauge-card h3 { font-size: 16px; font-weight: 600; color: #1e293b; margin-bottom: 16px; }
	.gauge { position: relative; width: 120px; height: 120px; margin: 0 auto 16px; }
	.gauge-svg { width: 100%; height: 100%; }

	.gauge-value {
		position: absolute;
		top: 50%; left: 50%;
		transform: translate(-50%, -50%);
		font-size: 24px; font-weight: 600; color: #1e293b;
	}

	.gauge-label { text-align: center; font-size: 13px; color: #64748b; }
	.gauge-label strong { color: #1e293b; }

	/* ===== PANELS ===== */
	.panel {
		background: white;
		border-radius: 12px;
		box-shadow: 0 1px 3px rgba(0,0,0,0.04);
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 20px 24px;
		border-bottom: 1px solid #f1f5f9;
	}

	.panel-header h2 { font-size: 17px; font-weight: 600; color: #1e293b; }

	.header-right {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	/* ===== SEARCH BOX ===== */
	.search-box {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 12px;
		height: 34px;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		background: #f8fafc;
		color: #94a3b8;
		transition: all 0.15s;
	}

	.search-box:focus-within {
		border-color: #3b82f6;
		background: white;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
		color: #64748b;
	}

	.search-box input {
		border: none;
		background: transparent;
		font-size: 13px;
		color: #334155;
		width: 140px;
		outline: none;
	}

	.search-box input::placeholder { color: #94a3b8; }

	.btn-icon {
		width: 34px; height: 34px;
		display: flex; align-items: center; justify-content: center;
		border: 1px solid #e2e8f0;
		background: white;
		border-radius: 8px;
		cursor: pointer;
		color: #64748b;
		transition: all 0.15s;
	}

	.btn-icon:hover { background: #f8fafc; color: #334155; border-color: #cbd5e1; }

	.btn-prune {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 12px;
		height: 34px;
		border: 1px solid #fecaca;
		background: white;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		color: #dc2626;
		cursor: pointer;
		transition: all 0.15s;
		white-space: nowrap;
	}

	.btn-prune:hover {
		background: #fef2f2;
		border-color: #f87171;
	}

	/* ===== ITEM LIST (shared card-style rows) ===== */
	.item-list {
		padding: 4px 0;
	}

	.item-row {
		display: grid;
		align-items: center;
		gap: 16px;
		padding: 14px 24px;
		border-bottom: 1px solid #f1f5f9;
		transition: background 0.1s ease;
	}

	.item-row:last-child { border-bottom: none; }
	.item-row:hover { background: #f8fafc; }

	/* Grid variants */
	.grid-container { grid-template-columns: 1fr auto auto; }
	.grid-image { grid-template-columns: 1fr 80px 90px auto; }
	.grid-volume { grid-template-columns: 1fr 80px 90px auto; }
	.grid-network { grid-template-columns: 1fr 80px 80px auto; }

	.item-info {
		display: flex;
		align-items: center;
		gap: 14px;
		min-width: 0;
	}

	.status-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.dot-running {
		background: #22c55e;
		box-shadow: 0 0 6px rgba(34, 197, 94, 0.4);
	}

	.dot-stopped {
		background: #94a3b8;
	}

	.item-details {
		min-width: 0;
	}

	.item-name-line {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 2px;
	}

	.item-name {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.item-sub {
		display: block;
		font-size: 12px;
		font-family: 'SF Mono', 'JetBrains Mono', monospace;
		color: #64748b;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.item-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		justify-content: flex-end;
		min-width: 100px;
	}

	.item-meta {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 1px;
	}

	.meta-label {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: #94a3b8;
		line-height: 1;
	}

	.meta-value {
		font-size: 13px;
		color: #334155;
		white-space: nowrap;
	}

	.item-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}

	/* ===== PULL BAR ===== */
	.pull-bar {
		display: flex;
		gap: 8px;
		padding: 16px 24px;
		border-bottom: 1px solid #f1f5f9;
	}

	.pull-bar input {
		flex: 1;
		padding: 9px 14px;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 14px;
		background: #f8fafc;
		transition: all 0.15s;
	}

	.pull-bar input:focus {
		outline: none;
		border-color: #3b82f6;
		background: white;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.pull-bar input:disabled { opacity: 0.6; }

	.btn-pull {
		display: flex; align-items: center; gap: 6px;
		padding: 9px 16px;
		border: none;
		background: #2563eb;
		color: white;
		border-radius: 8px;
		font-size: 13px; font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s;
	}

	.btn-pull:hover:not(:disabled) { background: #1d4ed8; }
	.btn-pull:disabled { opacity: 0.5; cursor: not-allowed; }

	/* ===== PORT TAGS ===== */
	.port-tag {
		display: inline-block;
		padding: 2px 8px;
		background: #f1f5f9;
		border: 1px solid #e2e8f0;
		border-radius: 4px;
		font-size: 11px;
		font-family: 'SF Mono', 'JetBrains Mono', monospace;
		color: #475569;
	}

	/* ===== ACTION BUTTONS ===== */
	.btn-action {
		width: 30px; height: 30px;
		display: flex; align-items: center; justify-content: center;
		border: 1px solid #e2e8f0;
		background: white;
		border-radius: 6px;
		cursor: pointer;
		color: #64748b;
		transition: all 0.15s;
	}

	.btn-action:hover:not(:disabled) {
		background: #f1f5f9;
		color: #334155;
		border-color: #cbd5e1;
	}

	.btn-action-danger:hover:not(:disabled) {
		background: #fef2f2;
		color: #dc2626;
		border-color: #fecaca;
	}

	.btn-action-success:hover:not(:disabled) {
		background: #f0fdf4;
		color: #16a34a;
		border-color: #bbf7d0;
	}

	.btn-action:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	/* ===== BADGES ===== */
	.badge {
		display: inline-flex;
		align-items: center;
		padding: 3px 10px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		white-space: nowrap;
	}

	.badge-green { background: #dcfce7; color: #15803d; }
	.badge-red { background: #fee2e2; color: #dc2626; }
	.badge-blue { background: #dbeafe; color: #1d4ed8; font-size: 11px; }

	.badge-outline {
		background: transparent;
		border: 1px solid #e2e8f0;
		color: #64748b;
	}

	/* ===== EMPTY STATE ===== */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px 24px;
		gap: 12px;
	}

	.empty-state p { font-size: 14px; color: #94a3b8; }

	/* ===== UTILITIES ===== */
	.spinning { animation: spin 1s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
	:global(.animate-spin) { animation: spin 1s linear infinite; }
</style>
