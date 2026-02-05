<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { api } from '$lib/stores/api';
	import FolderPicker from '$lib/components/ui/FolderPicker.svelte';

	// Types
	interface KodiStatus {
		connected: boolean;
		version: string;
		name: string;
		playing: boolean;
		paused: boolean;
		current_title: string | null;
		current_type: string | null;
		volume: number;
		muted: boolean;
	}

	interface KodiInfo {
		version: string;
		build: string;
		name: string;
		uptime: string;
		cpu_temp: number | null;
		free_space: string;
	}

	interface MediaSource {
		id: string;
		name: string;
		path: string;
		source_type: string;
		protocol: string | null;
	}

	interface KodiSetting {
		id: string;
		label: string;
		category: string;
		value: any;
		setting_type: string;
		options: { label: string; value: any }[] | null;
		min: number | null;
		max: number | null;
	}

	interface KodiAddon {
		id: string;
		name: string;
		version: string;
		addon_type: string;
		enabled: boolean;
		installed: boolean;
		description: string | null;
	}

	// State
	let status: KodiStatus | null = null;
	let info: KodiInfo | null = null;
	let sources: MediaSource[] = [];
	let settings: KodiSetting[] = [];
	let addons: KodiAddon[] = [];
	let loading = true;
	let error: string | null = null;
	let activeTab = 'remote';
	let settingsCategory = 'video';
	let sourceFilter = 'all';

	// Modals
	let showAddSourceModal = false;
	let newSource = { name: '', path: '', source_type: 'video' };

	// Polling interval
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	onMount(async () => {
		await loadAll();
		// Poll status every 3 seconds
		pollInterval = setInterval(loadStatus, 3000);
	});

	onDestroy(() => {
		if (pollInterval) {
			clearInterval(pollInterval);
		}
	});

	async function loadAll() {
		loading = true;
		error = null;
		try {
			await Promise.all([loadStatus(), loadInfo(), loadSources(), loadSettings(), loadAddons()]);
		} catch (e) {
			error = e instanceof Error ? e.message : $t.kodi.errors.loadFailed;
		} finally {
			loading = false;
		}
	}

	async function loadStatus() {
		try {
			status = await api.get<KodiStatus>('/kodi/status');
		} catch (e) {
			console.warn('Failed to get Kodi status:', e);
		}
	}

	async function loadInfo() {
		try {
			info = await api.get<KodiInfo>('/kodi/info');
		} catch (e) {
			console.warn('Failed to get Kodi info:', e);
		}
	}

	async function loadSources() {
		try {
			sources = await api.get<MediaSource[]>('/kodi/sources');
		} catch (e) {
			console.warn('Failed to get sources:', e);
		}
	}

	async function loadSettings() {
		try {
			settings = await api.get<KodiSetting[]>('/kodi/settings');
		} catch (e) {
			console.warn('Failed to get settings:', e);
		}
	}

	async function loadAddons() {
		try {
			addons = await api.get<KodiAddon[]>('/kodi/addons');
		} catch (e) {
			console.warn('Failed to get addons:', e);
		}
	}

	// Playback controls
	async function playPause() {
		await api.post('/kodi/playback/play-pause');
		await loadStatus();
	}

	async function stop() {
		await api.post('/kodi/playback/stop');
		await loadStatus();
	}

	async function setVolume(vol: number) {
		await api.post('/kodi/playback/volume', { volume: vol });
		if (status) status.volume = vol;
	}

	async function inputAction(action: string) {
		await api.post(`/kodi/input/${action}`);
	}

	// Sources management
	async function addSource() {
		if (!newSource.name || !newSource.path) return;
		try {
			await api.post('/kodi/sources', newSource);
			showAddSourceModal = false;
			newSource = { name: '', path: '', source_type: 'video' };
			await loadSources();
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.addSourceFailed);
		}
	}

	async function removeSource(sourceId: string) {
		if (!confirm($t.kodi.sources.deleteConfirm)) return;
		try {
			await api.delete(`/kodi/sources/${sourceId}`);
			await loadSources();
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.removeSourceFailed);
		}
	}

	// Settings
	async function updateSetting(settingId: string, value: any) {
		try {
			await api.put(`/kodi/settings/${settingId}`, { value });
			// Update local state
			const setting = settings.find((s) => s.id === settingId);
			if (setting) setting.value = value;
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.updateSettingFailed);
		}
	}

	// Addons
	async function toggleAddon(addon: KodiAddon) {
		const action = addon.enabled ? 'disable' : 'enable';
		try {
			await api.post(`/kodi/addons/${addon.id}/${action}`);
			addon.enabled = !addon.enabled;
			addons = [...addons];
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.toggleAddonFailed);
		}
	}

	// Library
	async function scanLibrary(type: string) {
		try {
			await api.post(`/kodi/library/${type}/scan`);
			alert($t.kodi.library.scanStarted);
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.scanFailed);
		}
	}

	// Computed
	$: filteredSources =
		sourceFilter === 'all' ? sources : sources.filter((s) => s.source_type === sourceFilter);
	$: filteredSettings = settings.filter((s) => s.category === settingsCategory);
	$: settingCategories = [...new Set(settings.map((s) => s.category))];
</script>

<div class="kodi-app">
	<!-- Header -->
	<header class="app-header">
		<div class="header-left">
			<Icon icon="simple-icons:kodi" class="w-6 h-6 kodi-icon" />
			<h1>{$t.kodi.title}</h1>
		</div>
		<div class="header-right">
			<div class="status-indicator" class:connected={status?.connected}>
				<span class="status-dot"></span>
				<span>{status?.connected ? $t.kodi.status.connected : $t.kodi.status.disconnected}</span>
			</div>
			{#if info}
				<span class="version-badge">v{info.version}</span>
			{/if}
		</div>
	</header>

	<!-- Navigation -->
	<nav class="tabs">
		<button class:active={activeTab === 'remote'} on:click={() => (activeTab = 'remote')}>
			<Icon icon="mdi:remote" />
			{$t.kodi.tabs.remote}
		</button>
		<button class:active={activeTab === 'sources'} on:click={() => (activeTab = 'sources')}>
			<Icon icon="mdi:folder-network" />
			{$t.kodi.tabs.sources}
		</button>
		<button class:active={activeTab === 'settings'} on:click={() => (activeTab = 'settings')}>
			<Icon icon="mdi:cog" />
			{$t.kodi.tabs.settings}
		</button>
		<button class:active={activeTab === 'addons'} on:click={() => (activeTab = 'addons')}>
			<Icon icon="mdi:puzzle" />
			{$t.kodi.tabs.addons}
		</button>
		<button class:active={activeTab === 'library'} on:click={() => (activeTab = 'library')}>
			<Icon icon="mdi:filmstrip" />
			{$t.kodi.tabs.library}
		</button>
	</nav>

	<!-- Content -->
	<main class="content">
		{#if loading}
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-8 h-8 animate-spin" />
				<span>{$t.common.loading}</span>
			</div>
		{:else if activeTab === 'remote'}
			<!-- Remote Control Tab -->
			<div class="remote-panel">
				<!-- Now Playing -->
				{#if status?.playing}
					<div class="now-playing">
						<Icon icon="mdi:play-circle" class="w-5 h-5" />
						<span>{status.current_title || $t.kodi.remote.nowPlaying}</span>
					</div>
				{/if}

				<!-- Playback Controls -->
				<div class="playback-section">
					<h3>{$t.kodi.remote.playback}</h3>
					<div class="playback-controls">
						<button class="control-btn" on:click={() => inputAction('back')} title={$t.kodi.remote.previous}>
							<Icon icon="mdi:skip-previous" />
						</button>
						<button class="control-btn play-btn" on:click={playPause}>
							<Icon icon={status?.playing ? 'mdi:pause' : 'mdi:play'} />
						</button>
						<button class="control-btn" on:click={stop} title={$t.kodi.remote.stop}>
							<Icon icon="mdi:stop" />
						</button>
						<button class="control-btn" on:click={() => inputAction('info')} title={$t.kodi.remote.next}>
							<Icon icon="mdi:skip-next" />
						</button>
					</div>
				</div>

				<!-- Volume -->
				<div class="volume-section">
					<h3>{$t.kodi.remote.volume}</h3>
					<div class="volume-control">
						<button on:click={() => setVolume(Math.max(0, (status?.volume || 50) - 10))}>
							<Icon icon="mdi:volume-minus" />
						</button>
						<input
							type="range"
							min="0"
							max="100"
							value={status?.volume || 50}
							on:change={(e) => setVolume(+e.currentTarget.value)}
						/>
						<button on:click={() => setVolume(Math.min(100, (status?.volume || 50) + 10))}>
							<Icon icon="mdi:volume-plus" />
						</button>
						<span class="volume-value">{status?.volume || 0}%</span>
					</div>
				</div>

				<!-- Navigation Pad -->
				<div class="nav-section">
					<h3>{$t.kodi.remote.navigation}</h3>
					<div class="nav-pad">
						<button class="nav-btn up" on:click={() => inputAction('up')}>
							<Icon icon="mdi:chevron-up" />
						</button>
						<div class="nav-row">
							<button class="nav-btn" on:click={() => inputAction('left')}>
								<Icon icon="mdi:chevron-left" />
							</button>
							<button class="nav-btn ok-btn" on:click={() => inputAction('select')}>OK</button>
							<button class="nav-btn" on:click={() => inputAction('right')}>
								<Icon icon="mdi:chevron-right" />
							</button>
						</div>
						<button class="nav-btn down" on:click={() => inputAction('down')}>
							<Icon icon="mdi:chevron-down" />
						</button>
					</div>
				</div>

				<!-- Quick Actions -->
				<div class="quick-actions">
					<button on:click={() => inputAction('back')}>
						<Icon icon="mdi:arrow-left" /> {$t.kodi.remote.back}
					</button>
					<button on:click={() => inputAction('home')}>
						<Icon icon="mdi:home" /> {$t.kodi.remote.home}
					</button>
					<button on:click={() => inputAction('context')}>
						<Icon icon="mdi:menu" /> {$t.kodi.remote.menu}
					</button>
					<button on:click={() => inputAction('info')}>
						<Icon icon="mdi:information" /> {$t.kodi.remote.info}
					</button>
				</div>
			</div>
		{:else if activeTab === 'sources'}
			<!-- Sources Tab -->
			<div class="sources-panel">
				<div class="panel-header">
					<h2>{$t.kodi.sources.title}</h2>
					<div class="header-actions">
						<select bind:value={sourceFilter}>
							<option value="all">{$t.kodi.sources.allTypes}</option>
							<option value="video">{$t.kodi.sources.mediaTypes.video}</option>
							<option value="music">{$t.kodi.sources.mediaTypes.music}</option>
							<option value="pictures">{$t.kodi.sources.mediaTypes.pictures}</option>
						</select>
						<button class="btn-primary" on:click={() => (showAddSourceModal = true)}>
							<Icon icon="mdi:plus" /> {$t.kodi.sources.addSource}
						</button>
					</div>
				</div>

				<div class="sources-list">
					{#each filteredSources as source}
						<div class="source-item">
							<div class="source-icon">
								{#if source.protocol === 'smb'}
									<Icon icon="mdi:microsoft-windows" />
								{:else if source.protocol === 'nfs'}
									<Icon icon="mdi:nas" />
								{:else}
									<Icon icon="mdi:folder" />
								{/if}
							</div>
							<div class="source-info">
								<span class="source-name">{source.name}</span>
								<span class="source-path">{source.path}</span>
								<div class="source-meta">
									<span class="badge type-badge">{source.source_type}</span>
									{#if source.protocol}
										<span class="badge protocol-badge">{source.protocol.toUpperCase()}</span>
									{/if}
								</div>
							</div>
							<button class="delete-btn" on:click={() => removeSource(source.id)} title={$t.common.delete}>
								<Icon icon="mdi:delete" />
							</button>
						</div>
					{/each}
					{#if filteredSources.length === 0}
						<div class="empty-state">
							<Icon icon="mdi:folder-off" class="w-12 h-12" />
							<span>{$t.kodi.sources.noSources}</span>
						</div>
					{/if}
				</div>
			</div>
		{:else if activeTab === 'settings'}
			<!-- Settings Tab -->
			<div class="settings-panel">
				<div class="settings-sidebar">
					{#each settingCategories as cat}
						<button
							class="category-btn"
							class:active={settingsCategory === cat}
							on:click={() => (settingsCategory = cat)}
						>
							{#if cat === 'video'}
								<Icon icon="mdi:video" />
							{:else if cat === 'audio'}
								<Icon icon="mdi:volume-high" />
							{:else if cat === 'interface'}
								<Icon icon="mdi:palette" />
							{:else if cat === 'network'}
								<Icon icon="mdi:network" />
							{:else if cat === 'cache'}
								<Icon icon="mdi:cached" />
							{:else}
								<Icon icon="mdi:cog" />
							{/if}
							<span>{cat.charAt(0).toUpperCase() + cat.slice(1)}</span>
						</button>
					{/each}
				</div>

				<div class="settings-content">
					<h2>{settingsCategory.charAt(0).toUpperCase() + settingsCategory.slice(1)}</h2>
					<div class="settings-list">
						{#each filteredSettings as setting}
							<div class="setting-item">
								<label for={setting.id}>{setting.label}</label>
								{#if setting.setting_type === 'boolean'}
									<label class="toggle">
										<input
											type="checkbox"
											id={setting.id}
											checked={setting.value}
											on:change={(e) => updateSetting(setting.id, e.currentTarget.checked)}
										/>
										<span class="slider"></span>
									</label>
								{:else if setting.setting_type === 'list' && setting.options}
									<select
										id={setting.id}
										value={setting.value}
										on:change={(e) => updateSetting(setting.id, e.currentTarget.value)}
									>
										{#each setting.options as opt}
											<option value={opt.value}>{opt.label}</option>
										{/each}
									</select>
								{:else if setting.setting_type === 'integer'}
									<input
										type="number"
										id={setting.id}
										value={setting.value}
										min={setting.min || undefined}
										max={setting.max || undefined}
										on:change={(e) => updateSetting(setting.id, +e.currentTarget.value)}
									/>
								{:else}
									<input
										type="text"
										id={setting.id}
										value={setting.value}
										on:change={(e) => updateSetting(setting.id, e.currentTarget.value)}
									/>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			</div>
		{:else if activeTab === 'addons'}
			<!-- Addons Tab -->
			<div class="addons-panel">
				<h2>{$t.kodi.addons.title}</h2>
				<div class="addons-list">
					{#each addons as addon}
						<div class="addon-item" class:disabled={!addon.enabled}>
							<div class="addon-icon">
								<Icon
									icon={addon.addon_type === 'skin'
										? 'mdi:palette'
										: addon.addon_type === 'video'
											? 'mdi:video'
											: addon.addon_type === 'service'
												? 'mdi:cog'
												: 'mdi:puzzle'}
								/>
							</div>
							<div class="addon-info">
								<span class="addon-name">{addon.name}</span>
								<span class="addon-version">v{addon.version}</span>
								{#if addon.description}
									<span class="addon-desc">{addon.description}</span>
								{/if}
							</div>
							<label class="toggle">
								<input
									type="checkbox"
									checked={addon.enabled}
									on:change={() => toggleAddon(addon)}
								/>
								<span class="slider"></span>
							</label>
						</div>
					{/each}
				</div>
			</div>
		{:else if activeTab === 'library'}
			<!-- Library Tab -->
			<div class="library-panel">
				<h2>{$t.kodi.library.title}</h2>

				<div class="library-actions">
					<div class="library-card">
						<Icon icon="mdi:filmstrip" class="w-12 h-12" />
						<h3>{$t.kodi.library.videoLibrary}</h3>
						<div class="card-actions">
							<button class="btn-primary" on:click={() => scanLibrary('video')}>
								<Icon icon="mdi:magnify" /> {$t.kodi.library.scan}
							</button>
							<button on:click={() => api.post('/kodi/library/video/clean')}>
								<Icon icon="mdi:broom" /> {$t.kodi.library.clean}
							</button>
						</div>
					</div>

					<div class="library-card">
						<Icon icon="mdi:music" class="w-12 h-12" />
						<h3>{$t.kodi.library.musicLibrary}</h3>
						<div class="card-actions">
							<button class="btn-primary" on:click={() => scanLibrary('music')}>
								<Icon icon="mdi:magnify" /> {$t.kodi.library.scan}
							</button>
							<button on:click={() => api.post('/kodi/library/music/clean')}>
								<Icon icon="mdi:broom" /> {$t.kodi.library.clean}
							</button>
						</div>
					</div>
				</div>
			</div>
		{/if}
	</main>
</div>

<!-- Add Source Modal -->
{#if showAddSourceModal}
	<div class="modal-overlay" on:click={() => (showAddSourceModal = false)}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h3>{$t.kodi.sources.addSource}</h3>
				<button class="close-btn" on:click={() => (showAddSourceModal = false)}>
					<Icon icon="mdi:close" />
				</button>
			</div>
			<div class="modal-body">
				<div class="form-group">
					<label for="source-name">{$t.kodi.sources.fields.sourceName}</label>
					<input
						type="text"
						id="source-name"
						bind:value={newSource.name}
						placeholder="Films NAS"
					/>
				</div>
				<div class="form-group">
					<FolderPicker
						bind:value={newSource.path}
						label={$t.kodi.sources.fields.serverPath}
						placeholder="smb://192.168.1.100/movies"
						hint={$t.kodi.sources.pathFormats}
					/>
				</div>
				<div class="form-group">
					<label for="source-type">{$t.kodi.sources.fields.mediaType}</label>
					<select id="source-type" bind:value={newSource.source_type}>
						<option value="video">{$t.kodi.sources.mediaTypes.video}</option>
						<option value="music">{$t.kodi.sources.mediaTypes.music}</option>
						<option value="pictures">{$t.kodi.sources.mediaTypes.pictures}</option>
						<option value="files">{$t.kodi.sources.mediaTypes.files}</option>
					</select>
				</div>
			</div>
			<div class="modal-footer">
				<button on:click={() => (showAddSourceModal = false)}>{$t.common.cancel}</button>
				<button class="btn-primary" on:click={addSource}>{$t.common.add}</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.kodi-app {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #f8f9fc;
	}

	/* Header */
	.app-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		background: linear-gradient(135deg, #17b0de 0%, #1a8cba 100%);
		color: white;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.header-left h1 {
		font-size: 20px;
		font-weight: 600;
		margin: 0;
	}

	.kodi-icon {
		color: white;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		opacity: 0.9;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #f87171;
	}

	.status-indicator.connected .status-dot {
		background: #4ade80;
	}

	.version-badge {
		padding: 4px 10px;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 12px;
		font-size: 12px;
	}

	/* Tabs */
	.tabs {
		display: flex;
		background: white;
		border-bottom: 1px solid #e5e7eb;
		padding: 0 16px;
	}

	.tabs button {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px 20px;
		font-size: 14px;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		transition: all 0.2s;
	}

	.tabs button:hover {
		color: #374151;
		background: #f3f4f6;
	}

	.tabs button.active {
		color: #17b0de;
		border-bottom-color: #17b0de;
	}

	/* Content */
	.content {
		flex: 1;
		overflow: auto;
		padding: 20px;
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 200px;
		gap: 12px;
		color: #6b7280;
	}

	/* Remote Panel */
	.remote-panel {
		max-width: 400px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.now-playing {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		padding: 12px 20px;
		background: linear-gradient(135deg, #17b0de 0%, #1a8cba 100%);
		color: white;
		border-radius: 12px;
		font-weight: 500;
	}

	.playback-section h3,
	.volume-section h3,
	.nav-section h3 {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		color: #6b7280;
		margin-bottom: 12px;
		text-align: center;
	}

	.playback-controls {
		display: flex;
		justify-content: center;
		gap: 12px;
	}

	.control-btn {
		width: 56px;
		height: 56px;
		border-radius: 50%;
		background: white;
		border: 1px solid #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		color: #374151;
		transition: all 0.2s;
	}

	.control-btn:hover {
		background: #f3f4f6;
		border-color: #17b0de;
		color: #17b0de;
	}

	.control-btn.play-btn {
		width: 72px;
		height: 72px;
		background: #17b0de;
		color: white;
		border: none;
		font-size: 32px;
	}

	.control-btn.play-btn:hover {
		background: #1a8cba;
	}

	/* Volume */
	.volume-control {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		background: white;
		border-radius: 12px;
		border: 1px solid #e5e7eb;
	}

	.volume-control button {
		width: 36px;
		height: 36px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
	}

	.volume-control button:hover {
		background: #f3f4f6;
		color: #17b0de;
	}

	.volume-control input[type='range'] {
		flex: 1;
		height: 6px;
		border-radius: 3px;
		background: #e5e7eb;
		accent-color: #17b0de;
	}

	.volume-value {
		min-width: 40px;
		text-align: right;
		font-size: 14px;
		font-weight: 500;
		color: #374151;
	}

	/* Navigation Pad */
	.nav-pad {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
	}

	.nav-row {
		display: flex;
		gap: 8px;
	}

	.nav-btn {
		width: 64px;
		height: 64px;
		border-radius: 12px;
		background: white;
		border: 1px solid #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		color: #374151;
		transition: all 0.2s;
	}

	.nav-btn:hover {
		background: #f3f4f6;
		border-color: #17b0de;
	}

	.nav-btn.ok-btn {
		background: #17b0de;
		color: white;
		border: none;
		font-weight: 600;
	}

	.nav-btn.ok-btn:hover {
		background: #1a8cba;
	}

	/* Quick Actions */
	.quick-actions {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
	}

	.quick-actions button {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 12px 8px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		font-size: 12px;
		color: #6b7280;
		transition: all 0.2s;
	}

	.quick-actions button:hover {
		background: #f3f4f6;
		color: #17b0de;
		border-color: #17b0de;
	}

	/* Sources Panel */
	.sources-panel,
	.addons-panel,
	.library-panel {
		background: white;
		border-radius: 12px;
		border: 1px solid #e5e7eb;
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
	}

	.panel-header h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
	}

	.header-actions {
		display: flex;
		gap: 12px;
	}

	.header-actions select {
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #17b0de;
		color: white;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
	}

	.btn-primary:hover {
		background: #1a8cba;
	}

	.sources-list {
		padding: 12px;
	}

	.source-item {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px;
		border-radius: 10px;
		transition: background 0.2s;
	}

	.source-item:hover {
		background: #f9fafb;
	}

	.source-icon {
		width: 48px;
		height: 48px;
		border-radius: 10px;
		background: #f3f4f6;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		color: #6b7280;
	}

	.source-info {
		flex: 1;
		min-width: 0;
	}

	.source-name {
		display: block;
		font-weight: 500;
		color: #1f2937;
		margin-bottom: 4px;
	}

	.source-path {
		display: block;
		font-size: 13px;
		color: #6b7280;
		word-break: break-all;
		margin-bottom: 6px;
	}

	.source-meta {
		display: flex;
		gap: 6px;
	}

	.badge {
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 500;
	}

	.type-badge {
		background: #e0f2fe;
		color: #0369a1;
	}

	.protocol-badge {
		background: #f3e8ff;
		color: #7c3aed;
	}

	.delete-btn {
		width: 36px;
		height: 36px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #9ca3af;
		opacity: 0;
		transition: all 0.2s;
	}

	.source-item:hover .delete-btn {
		opacity: 1;
	}

	.delete-btn:hover {
		background: #fef2f2;
		color: #ef4444;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px;
		color: #9ca3af;
		gap: 12px;
	}

	/* Settings Panel */
	.settings-panel {
		display: flex;
		background: white;
		border-radius: 12px;
		border: 1px solid #e5e7eb;
		overflow: hidden;
		min-height: 400px;
	}

	.settings-sidebar {
		width: 200px;
		background: #f9fafb;
		border-right: 1px solid #e5e7eb;
		padding: 12px;
	}

	.category-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 12px 16px;
		border-radius: 8px;
		font-size: 14px;
		color: #6b7280;
		text-align: left;
		transition: all 0.2s;
	}

	.category-btn:hover {
		background: #e5e7eb;
		color: #374151;
	}

	.category-btn.active {
		background: #17b0de;
		color: white;
	}

	.settings-content {
		flex: 1;
		padding: 20px;
	}

	.settings-content h2 {
		font-size: 18px;
		font-weight: 600;
		margin: 0 0 20px 0;
		text-transform: capitalize;
	}

	.settings-list {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.setting-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background: #f9fafb;
		border-radius: 10px;
	}

	.setting-item label {
		font-size: 14px;
		color: #374151;
	}

	.setting-item select,
	.setting-item input[type='number'],
	.setting-item input[type='text'] {
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		font-size: 14px;
		min-width: 200px;
	}

	/* Toggle Switch */
	.toggle {
		position: relative;
		display: inline-block;
		width: 48px;
		height: 26px;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.slider {
		position: absolute;
		cursor: pointer;
		inset: 0;
		background: #e5e7eb;
		border-radius: 26px;
		transition: 0.3s;
	}

	.slider::before {
		position: absolute;
		content: '';
		height: 20px;
		width: 20px;
		left: 3px;
		bottom: 3px;
		background: white;
		border-radius: 50%;
		transition: 0.3s;
	}

	.toggle input:checked + .slider {
		background: #17b0de;
	}

	.toggle input:checked + .slider::before {
		transform: translateX(22px);
	}

	/* Addons Panel */
	.addons-panel h2 {
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
		font-size: 16px;
		font-weight: 600;
		margin: 0;
	}

	.addons-list {
		padding: 12px;
	}

	.addon-item {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px;
		border-radius: 10px;
		transition: all 0.2s;
	}

	.addon-item:hover {
		background: #f9fafb;
	}

	.addon-item.disabled {
		opacity: 0.6;
	}

	.addon-icon {
		width: 48px;
		height: 48px;
		border-radius: 10px;
		background: #f3f4f6;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		color: #17b0de;
	}

	.addon-info {
		flex: 1;
	}

	.addon-name {
		display: block;
		font-weight: 500;
		color: #1f2937;
	}

	.addon-version {
		font-size: 12px;
		color: #9ca3af;
		margin-left: 8px;
	}

	.addon-desc {
		display: block;
		font-size: 13px;
		color: #6b7280;
		margin-top: 4px;
	}

	/* Library Panel */
	.library-panel h2 {
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
		font-size: 16px;
		font-weight: 600;
		margin: 0;
	}

	.library-actions {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 20px;
		padding: 20px;
	}

	.library-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 32px 24px;
		background: #f9fafb;
		border-radius: 12px;
		text-align: center;
	}

	.library-card h3 {
		margin: 16px 0;
		font-size: 16px;
		font-weight: 600;
	}

	.card-actions {
		display: flex;
		gap: 12px;
	}

	.card-actions button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 8px;
		font-size: 14px;
		border: 1px solid #e5e7eb;
		background: white;
	}

	.card-actions button:hover {
		background: #f3f4f6;
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 16px;
		width: 100%;
		max-width: 480px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 20px 24px;
		border-bottom: 1px solid #e5e7eb;
	}

	.modal-header h3 {
		font-size: 18px;
		font-weight: 600;
		margin: 0;
	}

	.close-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
	}

	.close-btn:hover {
		background: #f3f4f6;
	}

	.modal-body {
		padding: 24px;
	}

	.form-group {
		margin-bottom: 20px;
	}

	.form-group label {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 8px;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 10px 14px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: #17b0de;
		box-shadow: 0 0 0 3px rgba(23, 176, 222, 0.1);
	}

	.help-text {
		font-size: 12px;
		color: #6b7280;
		margin-top: 6px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 24px;
		border-top: 1px solid #e5e7eb;
	}

	.modal-footer button {
		padding: 10px 20px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
	}

	.modal-footer button:first-child {
		background: #f3f4f6;
		color: #374151;
	}

	.modal-footer button:first-child:hover {
		background: #e5e7eb;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
