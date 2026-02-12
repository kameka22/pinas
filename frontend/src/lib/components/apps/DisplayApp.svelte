<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { onMount, onDestroy } from 'svelte';
	import { api } from '$lib/stores/api';
	import FolderPicker from '$lib/components/ui/FolderPicker.svelte';

	// Display types
	interface DisplayService {
		id: string;
		name: string;
		description: string;
		running: boolean;
		enabled: boolean;
	}

	interface DisplayStatus {
		active_service: string | null;
		available_services: DisplayService[];
	}

	// Kodi types
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

	// Navigation: 'main' = service cards, 'configure' = service config sub-view
	let view: 'main' | 'configure' = 'main';
	let configServiceId: string | null = null;

	// Display state
	let displayStatus: DisplayStatus | null = null;
	let switching = false;

	// Kodi state
	let kodiStatus: KodiStatus | null = null;
	let kodiInfo: KodiInfo | null = null;
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

	// Polling
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	// Service icons & colors
	const serviceConfig: Record<string, { icon: string; gradient: string }> = {
		kodi: { icon: 'simple-icons:kodi', gradient: 'linear-gradient(135deg, #17b0de 0%, #1a8cba 100%)' },
		retroarch: { icon: 'simple-icons:retroarch', gradient: 'linear-gradient(135deg, #e44332 0%, #b5251a 100%)' },
		default: { icon: 'mdi:monitor', gradient: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }
	};

	function getServiceConfig(id: string) {
		return serviceConfig[id] || serviceConfig.default;
	}

	onMount(async () => {
		await loadAll();
		pollInterval = setInterval(pollStatus, 3000);
	});

	onDestroy(() => {
		if (pollInterval) clearInterval(pollInterval);
	});

	async function loadAll() {
		loading = true;
		error = null;
		try {
			await loadDisplayStatus();
			if (displayStatus?.active_service === 'kodi') {
				await Promise.all([loadKodiStatus(), loadKodiInfo(), loadSources(), loadSettings(), loadAddons()]);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : $t.kodi.errors.loadFailed;
		} finally {
			loading = false;
		}
	}

	async function pollStatus() {
		await loadDisplayStatus();
		if (displayStatus?.active_service === 'kodi') {
			await loadKodiStatus();
		}
	}

	// Display API
	async function loadDisplayStatus() {
		try {
			displayStatus = await api.get<DisplayStatus>('/display/status');
		} catch (e) {
			console.warn('Failed to get display status:', e);
		}
	}

	async function toggleService(serviceId: string) {
		if (switching) return;
		switching = true;
		try {
			const isActive = displayStatus?.active_service === serviceId;
			await api.post('/display/switch', { service: isActive ? null : serviceId });
			await loadDisplayStatus();
			if (!isActive && serviceId === 'kodi') {
				await Promise.all([loadKodiStatus(), loadKodiInfo(), loadSources(), loadSettings(), loadAddons()]);
			}
		} catch (e) {
			console.error('Failed to switch display service:', e);
		} finally {
			switching = false;
		}
	}

	function openConfigure(serviceId: string) {
		configServiceId = serviceId;
		view = 'configure';
		activeTab = 'remote';
	}

	function backToMain() {
		view = 'main';
		configServiceId = null;
	}

	// Kodi API
	async function loadKodiStatus() {
		try { kodiStatus = await api.get<KodiStatus>('/kodi/status'); } catch (e) { console.warn('Failed to get Kodi status:', e); }
	}

	async function loadKodiInfo() {
		try { kodiInfo = await api.get<KodiInfo>('/kodi/info'); } catch (e) { console.warn('Failed to get Kodi info:', e); }
	}

	async function loadSources() {
		try { sources = await api.get<MediaSource[]>('/kodi/sources'); } catch (e) { console.warn('Failed to get sources:', e); }
	}

	async function loadSettings() {
		try { settings = await api.get<KodiSetting[]>('/kodi/settings'); } catch (e) { console.warn('Failed to get settings:', e); }
	}

	async function loadAddons() {
		try { addons = await api.get<KodiAddon[]>('/kodi/addons'); } catch (e) { console.warn('Failed to get addons:', e); }
	}

	// Playback controls
	async function playPause() { await api.post('/kodi/playback/play-pause'); await loadKodiStatus(); }
	async function stopPlayback() { await api.post('/kodi/playback/stop'); await loadKodiStatus(); }
	async function setVolume(vol: number) { await api.post('/kodi/playback/volume', { volume: vol }); if (kodiStatus) kodiStatus.volume = vol; }
	async function gotoPrevious() { await api.post('/kodi/playback/previous'); }
	async function gotoNext() { await api.post('/kodi/playback/next'); }
	async function inputAction(action: string) { await api.post(`/kodi/input/${action}`); }

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
		try { await api.delete(`/kodi/sources/${sourceId}`); await loadSources(); } catch (e) { alert(e instanceof Error ? e.message : $t.kodi.errors.removeSourceFailed); }
	}

	// Settings
	async function updateSetting(settingId: string, value: any) {
		try {
			await api.put(`/kodi/settings/${settingId}`, { value });
			const setting = settings.find((s) => s.id === settingId);
			if (setting) setting.value = value;
		} catch (e) {
			alert(e instanceof Error ? e.message : $t.kodi.errors.updateSettingFailed);
		}
	}

	// Addons
	async function toggleAddon(addon: KodiAddon) {
		const action = addon.enabled ? 'disable' : 'enable';
		try { await api.post(`/kodi/addons/${addon.id}/${action}`); addon.enabled = !addon.enabled; addons = [...addons]; } catch (e) { alert(e instanceof Error ? e.message : $t.kodi.errors.toggleAddonFailed); }
	}

	// Library
	async function scanLibrary(type: string) {
		try { await api.post(`/kodi/library/${type}/scan`); alert($t.kodi.library.scanStarted); } catch (e) { alert(e instanceof Error ? e.message : $t.kodi.errors.scanFailed); }
	}

	// Computed
	$: filteredSources = sourceFilter === 'all' ? sources : sources.filter((s) => s.source_type === sourceFilter);
	$: filteredSettings = settings.filter((s) => s.category === settingsCategory);
	$: settingCategories = [...new Set(settings.map((s) => s.category))];
	$: kodiActive = displayStatus?.active_service === 'kodi';
	$: activeServiceObj = displayStatus?.available_services?.find((s) => s.id === displayStatus?.active_service);
</script>

<div class="display-app">
	{#if view === 'main'}
		<!-- ===== MAIN VIEW: Service Cards ===== -->
		<header class="app-header">
			<div class="header-left">
				<Icon icon="mdi:monitor-screenshot" class="w-6 h-6" />
				<h1>{$t.display.title}</h1>
			</div>
		</header>

		<!-- HDMI Output Status Banner -->
		<div class="hdmi-banner">
			<div class="hdmi-icon">
				<Icon icon="mdi:hdmi-port" class="w-5 h-5" />
			</div>
			<div class="hdmi-info">
				<span class="hdmi-label">{$t.display.hdmiOutput}</span>
				{#if activeServiceObj}
					<span class="hdmi-value active">{activeServiceObj.name}</span>
				{:else}
					<span class="hdmi-value">PiNAS Splash</span>
				{/if}
			</div>
			{#if activeServiceObj}
				<span class="hdmi-badge active">
					<span class="dot"></span>
					{$t.kodi.status.connected}
				</span>
			{:else}
				<span class="hdmi-badge">
					<Icon icon="mdi:monitor-screenshot" class="w-3.5 h-3.5" />
					Splash
				</span>
			{/if}
		</div>

		<!-- Service Cards -->
		<div class="main-content">
			{#if loading}
				<div class="loading-state">
					<Icon icon="mdi:loading" class="w-8 h-8 animate-spin" />
					<span>{$t.common.loading}</span>
				</div>
			{:else if displayStatus}
				<h2 class="section-title">{$t.display.services}</h2>
				<div class="service-grid">
					{#each displayStatus.available_services as service}
						{@const isActive = displayStatus.active_service === service.id}
						{@const conf = getServiceConfig(service.id)}
						<div class="service-card" class:active={isActive}>
							<div class="card-header">
								<div class="card-icon" style="background: {isActive ? conf.gradient : ''}">
									<Icon icon={conf.icon} class="w-7 h-7" />
								</div>
								<label class="toggle">
									<input
										type="checkbox"
										checked={isActive}
										disabled={switching}
										on:change={() => toggleService(service.id)}
									/>
									<span class="slider"></span>
								</label>
							</div>
							<div class="card-body">
								<span class="card-name">{service.name}</span>
								<span class="card-desc">{service.description}</span>
							</div>
							<div class="card-footer">
								{#if isActive && service.running}
									<span class="status-pill active">
										<span class="dot"></span>
										{$t.kodi.status.connected}
									</span>
									<button class="configure-btn" on:click={() => openConfigure(service.id)}>
										<Icon icon="mdi:cog" class="w-4 h-4" />
										{$t.display.configure}
									</button>
								{:else if switching}
									<span class="status-pill switching">
										<Icon icon="mdi:loading" class="w-3.5 h-3.5 animate-spin" />
										{isActive ? $t.display.deactivating : $t.display.activating}
									</span>
								{:else}
									<span class="status-pill inactive">
										{$t.kodi.status.disconnected}
									</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>

				<!-- Splash info when no service active -->
				{#if !displayStatus.active_service}
					<div class="splash-banner">
						<Icon icon="mdi:information-outline" class="w-5 h-5" />
						<div class="splash-text">
							<span>{$t.display.splashActive}</span>
							<code>{window.location.origin}</code>
						</div>
					</div>
				{/if}
			{/if}
		</div>

	{:else}
		<!-- ===== CONFIGURE VIEW: Service-specific settings ===== -->
		<header class="config-header">
			<button class="back-btn" on:click={backToMain}>
				<Icon icon="mdi:arrow-left" class="w-5 h-5" />
				<span>{$t.display.back}</span>
			</button>
			<div class="config-title">
				{#if configServiceId}
					{@const conf = getServiceConfig(configServiceId)}
					<div class="config-icon" style="background: {conf.gradient}">
						<Icon icon={conf.icon} class="w-5 h-5" />
					</div>
				{/if}
				<h1>
					{#if configServiceId === 'kodi'}Kodi{:else}{configServiceId}{/if}
				</h1>
			</div>
			<div class="config-meta">
				{#if kodiActive && kodiInfo}
					<span class="version-badge">v{kodiInfo.version}</span>
				{/if}
				{#if kodiActive && kodiStatus?.playing}
					<span class="playing-badge">
						<Icon icon="mdi:play-circle" class="w-4 h-4" />
						{kodiStatus.current_title || $t.kodi.remote.nowPlaying}
					</span>
				{/if}
			</div>
		</header>

		{#if configServiceId === 'kodi'}
			<div class="config-layout">
				<!-- Sidebar tabs -->
				<nav class="config-sidebar">
					<button class:active={activeTab === 'remote'} on:click={() => (activeTab = 'remote')}>
						<Icon icon="mdi:remote" class="w-5 h-5" />
						<span>{$t.kodi.tabs.remote}</span>
					</button>
					<button class:active={activeTab === 'sources'} on:click={() => (activeTab = 'sources')}>
						<Icon icon="mdi:folder-network" class="w-5 h-5" />
						<span>{$t.kodi.tabs.sources}</span>
					</button>
					<button class:active={activeTab === 'settings'} on:click={() => (activeTab = 'settings')}>
						<Icon icon="mdi:cog" class="w-5 h-5" />
						<span>{$t.kodi.tabs.settings}</span>
					</button>
					<button class:active={activeTab === 'addons'} on:click={() => (activeTab = 'addons')}>
						<Icon icon="mdi:puzzle" class="w-5 h-5" />
						<span>{$t.kodi.tabs.addons}</span>
					</button>
					<button class:active={activeTab === 'library'} on:click={() => (activeTab = 'library')}>
						<Icon icon="mdi:filmstrip" class="w-5 h-5" />
						<span>{$t.kodi.tabs.library}</span>
					</button>
				</nav>

				<!-- Tab content -->
				<div class="config-content">
					{#if activeTab === 'remote'}
						<!-- Remote Control -->
						<div class="remote-panel">
							<div class="playback-section">
								<h3>{$t.kodi.remote.playback}</h3>
								<div class="playback-controls">
									<button class="control-btn" on:click={gotoPrevious} title={$t.kodi.remote.previous}>
										<Icon icon="mdi:skip-previous" />
									</button>
									<button class="control-btn play-btn" on:click={playPause}>
										<Icon icon={kodiStatus?.playing ? 'mdi:pause' : 'mdi:play'} />
									</button>
									<button class="control-btn" on:click={stopPlayback} title={$t.kodi.remote.stop}>
										<Icon icon="mdi:stop" />
									</button>
									<button class="control-btn" on:click={gotoNext} title={$t.kodi.remote.next}>
										<Icon icon="mdi:skip-next" />
									</button>
								</div>
							</div>

							<div class="volume-section">
								<h3>{$t.kodi.remote.volume}</h3>
								<div class="volume-control">
									<button on:click={() => setVolume(Math.max(0, (kodiStatus?.volume || 50) - 10))}>
										<Icon icon="mdi:volume-minus" />
									</button>
									<input
										type="range"
										min="0"
										max="100"
										value={kodiStatus?.volume || 50}
										on:change={(e) => setVolume(+e.currentTarget.value)}
									/>
									<button on:click={() => setVolume(Math.min(100, (kodiStatus?.volume || 50) + 10))}>
										<Icon icon="mdi:volume-plus" />
									</button>
									<span class="volume-value">{kodiStatus?.volume || 0}%</span>
								</div>
							</div>

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
						<!-- Sources -->
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

					{:else if activeTab === 'settings'}
						<!-- Settings -->
						<div class="settings-layout">
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
						<!-- Addons -->
						<div class="addons-header">
							<h2>{$t.kodi.addons.title}</h2>
						</div>
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

					{:else if activeTab === 'library'}
						<!-- Library -->
						<div class="addons-header">
							<h2>{$t.kodi.library.title}</h2>
						</div>
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
					{/if}
				</div>
			</div>
		{/if}
	{/if}
</div>

<!-- Add Source Modal -->
{#if showAddSourceModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={() => (showAddSourceModal = false)}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
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
					<input type="text" id="source-name" bind:value={newSource.name} placeholder="Films NAS" />
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
	.display-app {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #f8f9fc;
	}

	/* ========== HEADER (Main) ========== */
	.app-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 24px;
		background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
		color: white;
		flex-shrink: 0;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.header-left h1 {
		font-size: 18px;
		font-weight: 600;
		margin: 0;
	}

	/* ========== HDMI BANNER ========== */
	.hdmi-banner {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 24px;
		background: white;
		border-bottom: 1px solid #e5e7eb;
		flex-shrink: 0;
	}

	.hdmi-icon {
		width: 36px;
		height: 36px;
		border-radius: 8px;
		background: #f1f5f9;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
	}

	.hdmi-info {
		flex: 1;
		display: flex;
		flex-direction: column;
	}

	.hdmi-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: #94a3b8;
	}

	.hdmi-value {
		font-size: 14px;
		font-weight: 500;
		color: #64748b;
	}

	.hdmi-value.active {
		color: #1e293b;
	}

	.hdmi-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 12px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
		background: #f1f5f9;
		color: #64748b;
	}

	.hdmi-badge.active {
		background: #f0fdf4;
		color: #16a34a;
	}

	.hdmi-badge .dot,
	.status-pill .dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
	}

	/* ========== MAIN CONTENT ========== */
	.main-content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.section-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: #94a3b8;
		margin: 0 0 16px 0;
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

	/* ========== SERVICE CARDS ========== */
	.service-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 16px;
	}

	.service-card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 16px;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		transition: all 0.2s;
	}

	.service-card.active {
		border-color: #6366f1;
		box-shadow: 0 0 0 1px #6366f1, 0 4px 16px rgba(99, 102, 241, 0.1);
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.card-icon {
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: #f1f5f9;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #94a3b8;
		transition: all 0.2s;
	}

	.service-card.active .card-icon {
		color: white;
	}

	.card-body {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.card-name {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.card-desc {
		font-size: 13px;
		color: #94a3b8;
		line-height: 1.4;
	}

	.card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		min-height: 32px;
	}

	.status-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
	}

	.status-pill.active {
		background: #f0fdf4;
		color: #16a34a;
	}

	.status-pill.inactive {
		background: #f8fafc;
		color: #94a3b8;
	}

	.status-pill.switching {
		background: #fef9c3;
		color: #a16207;
	}

	.configure-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		background: #6366f1;
		color: white;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		transition: all 0.15s;
	}

	.configure-btn:hover {
		background: #4f46e5;
	}

	/* ========== SPLASH BANNER ========== */
	.splash-banner {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin-top: 20px;
		padding: 16px 20px;
		background: #f0f9ff;
		border: 1px solid #bae6fd;
		border-radius: 12px;
		color: #0369a1;
	}

	.splash-text {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.splash-text span {
		font-size: 14px;
	}

	.splash-text code {
		font-size: 14px;
		font-weight: 600;
		color: #6366f1;
	}

	/* ========== CONFIG HEADER ========== */
	.config-header {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 12px 20px;
		background: white;
		border-bottom: 1px solid #e5e7eb;
		flex-shrink: 0;
	}

	.back-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		color: #64748b;
		transition: all 0.15s;
	}

	.back-btn:hover {
		background: #f1f5f9;
		color: #1e293b;
	}

	.config-title {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.config-icon {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
	}

	.config-title h1 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
		margin: 0;
	}

	.config-meta {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.version-badge {
		padding: 3px 10px;
		background: #f1f5f9;
		border-radius: 12px;
		font-size: 12px;
		font-weight: 500;
		color: #64748b;
	}

	.playing-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 3px 10px;
		background: #f0fdf4;
		color: #16a34a;
		border-radius: 12px;
		font-size: 12px;
		font-weight: 500;
	}

	/* ========== CONFIG LAYOUT ========== */
	.config-layout {
		flex: 1;
		display: flex;
		overflow: hidden;
	}

	.config-sidebar {
		width: 180px;
		background: #f8f9fc;
		border-right: 1px solid #e5e7eb;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex-shrink: 0;
	}

	.config-sidebar button {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		border-radius: 8px;
		font-size: 13px;
		color: #64748b;
		text-align: left;
		transition: all 0.15s;
	}

	.config-sidebar button:hover {
		background: #e5e7eb;
		color: #374151;
	}

	.config-sidebar button.active {
		background: #6366f1;
		color: white;
	}

	.config-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	/* ========== REMOTE PANEL ========== */
	.remote-panel {
		max-width: 400px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.playback-section h3,
	.volume-section h3,
	.nav-section h3 {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		color: #94a3b8;
		margin-bottom: 12px;
		text-align: center;
	}

	.playback-controls {
		display: flex;
		justify-content: center;
		gap: 12px;
	}

	.control-btn {
		width: 52px;
		height: 52px;
		border-radius: 50%;
		background: white;
		border: 1px solid #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		color: #374151;
		transition: all 0.15s;
	}

	.control-btn:hover {
		background: #f3f4f6;
		border-color: #6366f1;
		color: #6366f1;
	}

	.control-btn.play-btn {
		width: 64px;
		height: 64px;
		background: #6366f1;
		color: white;
		border: none;
		font-size: 28px;
	}

	.control-btn.play-btn:hover {
		background: #4f46e5;
	}

	.volume-control {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		background: white;
		border-radius: 12px;
		border: 1px solid #e5e7eb;
	}

	.volume-control button {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
	}

	.volume-control button:hover {
		background: #f3f4f6;
		color: #6366f1;
	}

	.volume-control input[type='range'] {
		flex: 1;
		height: 6px;
		border-radius: 3px;
		background: #e5e7eb;
		accent-color: #6366f1;
	}

	.volume-value {
		min-width: 36px;
		text-align: right;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
	}

	.nav-pad {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.nav-row {
		display: flex;
		gap: 6px;
	}

	.nav-btn {
		width: 60px;
		height: 60px;
		border-radius: 12px;
		background: white;
		border: 1px solid #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		color: #374151;
		transition: all 0.15s;
	}

	.nav-btn:hover {
		background: #f3f4f6;
		border-color: #6366f1;
	}

	.nav-btn.ok-btn {
		background: #6366f1;
		color: white;
		border: none;
		font-weight: 600;
		font-size: 14px;
	}

	.nav-btn.ok-btn:hover {
		background: #4f46e5;
	}

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
		padding: 10px 6px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		font-size: 11px;
		color: #6b7280;
		transition: all 0.15s;
	}

	.quick-actions button:hover {
		background: #f3f4f6;
		color: #6366f1;
		border-color: #6366f1;
	}

	/* ========== SOURCES PANEL ========== */
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
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
		padding: 7px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 13px;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		background: #6366f1;
		color: white;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
	}

	.btn-primary:hover {
		background: #4f46e5;
	}

	.sources-list {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 12px;
		overflow: hidden;
	}

	.source-item {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		transition: background 0.15s;
	}

	.source-item:not(:last-child) {
		border-bottom: 1px solid #f1f5f9;
	}

	.source-item:hover {
		background: #f9fafb;
	}

	.source-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		background: #f3f4f6;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		color: #6b7280;
		flex-shrink: 0;
	}

	.source-info {
		flex: 1;
		min-width: 0;
	}

	.source-name {
		display: block;
		font-weight: 500;
		font-size: 14px;
		color: #1f2937;
		margin-bottom: 2px;
	}

	.source-path {
		display: block;
		font-size: 12px;
		color: #94a3b8;
		word-break: break-all;
		margin-bottom: 4px;
	}

	.source-meta {
		display: flex;
		gap: 6px;
	}

	.badge {
		padding: 1px 7px;
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
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #9ca3af;
		opacity: 0;
		transition: all 0.15s;
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

	/* ========== SETTINGS LAYOUT ========== */
	.settings-layout {
		display: flex;
		background: white;
		border-radius: 12px;
		border: 1px solid #e5e7eb;
		overflow: hidden;
		min-height: 360px;
	}

	.settings-sidebar {
		width: 180px;
		background: #f9fafb;
		border-right: 1px solid #e5e7eb;
		padding: 10px;
	}

	.category-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 10px 14px;
		border-radius: 8px;
		font-size: 13px;
		color: #6b7280;
		text-align: left;
		transition: all 0.15s;
	}

	.category-btn:hover {
		background: #e5e7eb;
		color: #374151;
	}

	.category-btn.active {
		background: #6366f1;
		color: white;
	}

	.settings-content {
		flex: 1;
		padding: 20px;
	}

	.settings-content h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0 0 16px 0;
		text-transform: capitalize;
	}

	.settings-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.setting-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		background: #f9fafb;
		border-radius: 10px;
	}

	.setting-item label {
		font-size: 13px;
		color: #374151;
	}

	.setting-item select,
	.setting-item input[type='number'],
	.setting-item input[type='text'] {
		padding: 7px 10px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		font-size: 13px;
		min-width: 180px;
	}

	/* ========== ADDONS ========== */
	.addons-header {
		margin-bottom: 16px;
	}

	.addons-header h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
	}

	.addons-list {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 12px;
		overflow: hidden;
	}

	.addon-item {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		transition: all 0.15s;
	}

	.addon-item:not(:last-child) {
		border-bottom: 1px solid #f1f5f9;
	}

	.addon-item:hover {
		background: #f9fafb;
	}

	.addon-item.disabled {
		opacity: 0.6;
	}

	.addon-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		background: #f3f4f6;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		color: #6366f1;
		flex-shrink: 0;
	}

	.addon-info {
		flex: 1;
	}

	.addon-name {
		display: inline;
		font-weight: 500;
		font-size: 14px;
		color: #1f2937;
	}

	.addon-version {
		font-size: 11px;
		color: #9ca3af;
		margin-left: 6px;
	}

	.addon-desc {
		display: block;
		font-size: 12px;
		color: #6b7280;
		margin-top: 2px;
	}

	/* ========== LIBRARY ========== */
	.library-actions {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 16px;
	}

	.library-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 28px 20px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 12px;
		text-align: center;
	}

	.library-card h3 {
		margin: 12px 0;
		font-size: 15px;
		font-weight: 600;
	}

	.card-actions {
		display: flex;
		gap: 10px;
	}

	.card-actions button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: 8px;
		font-size: 13px;
		border: 1px solid #e5e7eb;
		background: white;
	}

	.card-actions button:hover {
		background: #f3f4f6;
	}

	/* ========== TOGGLE SWITCH ========== */
	.toggle {
		position: relative;
		display: inline-block;
		width: 44px;
		height: 24px;
		flex-shrink: 0;
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
		border-radius: 24px;
		transition: 0.25s;
	}

	.slider::before {
		position: absolute;
		content: '';
		height: 18px;
		width: 18px;
		left: 3px;
		bottom: 3px;
		background: white;
		border-radius: 50%;
		transition: 0.25s;
	}

	.toggle input:checked + .slider {
		background: #6366f1;
	}

	.toggle input:checked + .slider::before {
		transform: translateX(20px);
	}

	.toggle input:disabled + .slider {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* ========== MODAL ========== */
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
		padding: 18px 24px;
		border-bottom: 1px solid #e5e7eb;
	}

	.modal-header h3 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
	}

	.close-btn {
		width: 30px;
		height: 30px;
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
		padding: 20px 24px;
	}

	.form-group {
		margin-bottom: 16px;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 6px;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 9px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: #6366f1;
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
		padding: 14px 24px;
		border-top: 1px solid #e5e7eb;
	}

	.modal-footer button {
		padding: 8px 18px;
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

	/* ========== UTILS ========== */
	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
