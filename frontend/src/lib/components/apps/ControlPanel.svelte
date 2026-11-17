<script lang="ts">
	import Icon from '@iconify/svelte';
	import UserManager from './UserManager.svelte';
	import TimeLanguage from './TimeLanguage.svelte';
	import FileService from './FileService.svelte';
	import TerminalSettings from './TerminalSettings.svelte';
	import PrinterSettings from './PrinterSettings.svelte';
	import NetworkSettings from './NetworkSettings.svelte';
	import UpdateSettings from './UpdateSettings.svelte';
	import HardwarePower from './HardwarePower.svelte';
	import SecuritySettings from './SecuritySettings.svelte';
	import DeviceConnection from './DeviceConnection.svelte';
	import Personalization from './Personalization.svelte';
	import ConfirmModal from '#lib/components/ui/ConfirmModal.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsTabs from '#lib/components/ui/settings/SettingsTabs.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { errorMessage } from '#lib/stores/toasts.js';
	import { hasUnsavedChanges } from '#lib/stores/unsavedChanges.js';
	import { onMount } from 'svelte';
	import { t, locale } from '#lib/i18n/index.js';
	import { systemInfo as systemInfoStore, formatBytes, formatUptime } from '#lib/stores/system.js';

	let aboutPools: StoragePool[] = [];
	let aboutVolumes: VolumeInfo[] = [];
	let aboutServices: ServiceStatus[] = [];
	let aboutLoadedFor: string | null = null;
	function statusTone(status: string): string {
		if (status === 'normal' || status === 'mounted') return 'success';
		if (status === 'error') return 'danger';
		if (status === 'unmounted') return '';
		return 'warning';
	}
	let aboutLoading = false;
	let aboutError = '';
	async function loadAboutTab(tab: string) {
		if (aboutLoadedFor === tab) return;
		aboutLoadedFor = tab;
		aboutLoading = true;
		aboutError = '';
		try {
			if (tab === 'storage') [aboutPools, aboutVolumes] = await Promise.all([api.getPools(), api.getVolumes()]);
			if (tab === 'service') aboutServices = await api.get<ServiceStatus[]>('/services');
		} catch (e) {
			aboutLoadedFor = null;
			aboutError = errorMessage(e, $t.common.errors.loadFailed);
		} finally {
			aboutLoading = false;
		}
	}
	$: if (selectedItem === 'about' && activeTab !== 'general') loadAboutTab(activeTab);
	import { api, auth, type NetworkInterface, type StoragePool, type VolumeInfo, type ServiceStatus } from '#lib/stores/api.js';
	import { gradientStyle } from '#lib/utils/gradient.js';

	export let config: { section?: string } | undefined = undefined;

	type ViewMode = 'grid' | 'detail';
	let viewMode: ViewMode = config?.section ? 'detail' : 'grid';

	let searchQuery = '';

	interface SidebarItem {
		id: string;
		icon: string;
		labelKey: string;
		iconColor: string;
		component?: string;
	}

	interface SidebarCategory {
		titleKey: string;
		items: SidebarItem[];
	}

	const sidebarCategories: SidebarCategory[] = [
		{
			titleKey: 'controlPanel.categories.connectionAccess',
			items: [
				{ id: 'users', icon: 'mdi:account-multiple', labelKey: 'controlPanel.items.userManagement', iconColor: 'text-blue-500', component: 'UserManager' },
				{ id: 'files', icon: 'mdi:folder', labelKey: 'controlPanel.items.fileService', iconColor: 'text-orange-500', component: 'FileService' },
				{ id: 'device', icon: 'mdi:monitor-screenshot', labelKey: 'controlPanel.items.deviceConnection', iconColor: 'text-slate-500', component: 'DeviceConnection' },
				{ id: 'terminal', icon: 'mdi:console-line', labelKey: 'controlPanel.items.terminal', iconColor: 'text-purple-500', component: 'TerminalSettings' },
				{ id: 'printer', icon: 'mdi:printer', labelKey: 'controlPanel.items.printer', iconColor: 'text-indigo-500', component: 'PrinterSettings' }
			]
		},
		{
			titleKey: 'controlPanel.categories.general',
			items: [
				{ id: 'hardware', icon: 'mdi:chip', labelKey: 'controlPanel.items.hardwarePower', iconColor: 'text-emerald-500', component: 'HardwarePower' },
				{ id: 'time', icon: 'mdi:earth', labelKey: 'controlPanel.items.timeLanguage', iconColor: 'text-cyan-500', component: 'TimeLanguage' },
				{ id: 'network', icon: 'mdi:wifi', labelKey: 'controlPanel.items.network', iconColor: 'text-blue-500', component: 'NetworkSettings' },
				{ id: 'security', icon: 'mdi:shield-check', labelKey: 'controlPanel.items.security', iconColor: 'text-green-500', component: 'SecuritySettings' },
				{ id: 'personalization', icon: 'mdi:palette', labelKey: 'controlPanel.items.personalization', iconColor: 'text-pink-500', component: 'Personalization' }
			]
		},
		{
			titleKey: 'controlPanel.categories.service',
			items: [
				{ id: 'update', icon: 'mdi:update', labelKey: 'controlPanel.items.systemUpdate', iconColor: 'text-blue-600', component: 'UpdateSettings' },
				{ id: 'about', icon: 'mdi:information', labelKey: 'controlPanel.items.about', iconColor: 'text-teal-500' }
			]
		}
	];

	// Items for grid view with gradients
	interface GridItem {
		id: string;
		icon: string;
		labelKey: string;
		gradient: string;
		badge?: number;
	}

	interface GridCategory {
		titleKey: string;
		items: GridItem[];
	}

	const gridCategories: GridCategory[] = [
		{
			titleKey: 'controlPanel.categories.connectionAccess',
			items: [
				{ id: 'users', icon: 'mdi:account-multiple', labelKey: 'controlPanel.items.userManagement', gradient: 'from-blue-400 to-blue-500' },
				{ id: 'files', icon: 'mdi:folder', labelKey: 'controlPanel.items.fileService', gradient: 'from-orange-400 to-orange-500' },
				{ id: 'device', icon: 'mdi:monitor-screenshot', labelKey: 'controlPanel.items.deviceConnection', gradient: 'from-slate-400 to-slate-500' },
				{ id: 'terminal', icon: 'mdi:console-line', labelKey: 'controlPanel.items.terminal', gradient: 'from-purple-400 to-purple-500' },
				{ id: 'printer', icon: 'mdi:printer', labelKey: 'controlPanel.items.printer', gradient: 'from-indigo-400 to-indigo-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.general',
			items: [
				{ id: 'hardware', icon: 'mdi:chip', labelKey: 'controlPanel.items.hardwarePower', gradient: 'from-emerald-400 to-emerald-500' },
				{ id: 'time', icon: 'mdi:earth', labelKey: 'controlPanel.items.timeLanguage', gradient: 'from-cyan-400 to-cyan-500' },
				{ id: 'network', icon: 'mdi:wifi', labelKey: 'controlPanel.items.network', gradient: 'from-blue-400 to-blue-500' },
				{ id: 'security', icon: 'mdi:shield-check', labelKey: 'controlPanel.items.security', gradient: 'from-green-400 to-green-500' },
				{ id: 'personalization', icon: 'mdi:palette', labelKey: 'controlPanel.items.personalization', gradient: 'from-pink-400 to-pink-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.service',
			items: [
				{ id: 'update', icon: 'mdi:update', labelKey: 'controlPanel.items.systemUpdate', gradient: 'from-blue-500 to-blue-600' },
				{ id: 'about', icon: 'mdi:information', labelKey: 'controlPanel.items.about', gradient: 'from-teal-400 to-teal-500' }
			]
		}
	];

	let selectedItem: string | null = config?.section || null;
	let activeTab = 'general';

	// Primary network interface for the "About" card
	let networkHostname = '';
	let primaryInterface: NetworkInterface | null = null;
	async function refreshNetworkIdentity() {
		try {
			const status = await api.getNetworkStatus();
			networkHostname = status.hostname;
			primaryInterface =
				status.interfaces.find((i) => i.ip_address) ?? status.interfaces[0] ?? null;
		} catch {
			// Network card simply stays empty
		}
	}
	onMount(refreshNetworkIdentity);
	$: if (selectedItem === 'about') refreshNetworkIdentity();

	const NA = '—';
	$: systemInfo = {
		deviceName: networkHostname || $systemInfoStore?.hostname || NA,
		systemVersion: $systemInfoStore?.version ?? NA,
		deviceOwner: $auth.user?.username ?? NA,
		model: $systemInfoStore?.model ?? NA,
		serialNumber: $systemInfoStore?.serial ?? NA,
		lastStartup: $systemInfoStore ? new Date($systemInfoStore.bootTime * 1000).toLocaleString() : NA,
		powerTime: $systemInfoStore ? formatUptime($systemInfoStore.uptime, $locale) : NA,
		cpu: {
			model: $systemInfoStore?.cpu.model || NA,
			freq: $systemInfoStore?.cpu.frequencyMhz ? `${$systemInfoStore.cpu.frequencyMhz} MHz` : null,
			cores: $systemInfoStore?.cpu.cores ?? NA,
			temp: $systemInfoStore?.cpu.temperature != null ? `${Math.round($systemInfoStore.cpu.temperature)}°C` : null
		},
		memory: {
			size: $systemInfoStore ? formatBytes($systemInfoStore.memory.total) : NA
		},
		network: {
			interface: primaryInterface?.display_name || primaryInterface?.name || NA,
			ip: primaryInterface?.ip_address || NA,
			speed: primaryInterface?.speed || null,
			mac: primaryInterface?.mac_address || NA,
			subnet: primaryInterface?.subnet_mask || NA
		}
	};

	// Navigation away from a section with pending edits asks first.
	let pendingNavigation: (() => void) | null = null;
	function navigate(action: () => void) {
		if ($hasUnsavedChanges) pendingNavigation = action;
		else action();
	}
	function confirmNavigation() {
		const action = pendingNavigation;
		pendingNavigation = null;
		action?.();
	}

	function selectItem(id: string) {
		if (id === selectedItem && viewMode === 'detail') return;
		navigate(() => {
			selectedItem = id;
			activeTab = 'general';
			viewMode = 'detail';
		});
	}

	function goToGrid() {
		navigate(() => {
			viewMode = 'grid';
			selectedItem = null;
		});
	}

	// Helper to get translation by key path
	function getTranslation(obj: any, path: string): string {
		const keys = path.split('.');
		let result = obj;
		for (const key of keys) {
			if (result && typeof result === 'object' && key in result) {
				result = result[key];
			} else {
				return path;
			}
		}
		return typeof result === 'string' ? result : path;
	}

	$: filteredGridCategories = searchQuery
		? gridCategories
				.map((cat) => ({
					...cat,
					items: cat.items.filter((item) =>
						getTranslation($t, item.labelKey).toLowerCase().includes(searchQuery.toLowerCase())
					)
				}))
				.filter((cat) => cat.items.length > 0)
		: gridCategories;

	$: filteredSidebarCategories = searchQuery
		? sidebarCategories
				.map((cat) => ({
					...cat,
					items: cat.items.filter((item) =>
						getTranslation($t, item.labelKey).toLowerCase().includes(searchQuery.toLowerCase())
					)
				}))
				.filter((cat) => cat.items.length > 0)
		: sidebarCategories;

	$: tabs = [
		{ value: 'general', label: $t.controlPanel.tabs.general },
		{ value: 'storage', label: $t.controlPanel.tabs.storage },
		{ value: 'service', label: $t.controlPanel.tabs.service }
	];
</script>

<div class="control-panel">
	{#if viewMode === 'detail'}
		<!-- Detail View with Sidebar -->
		<aside class="sidebar">
			<!-- Grid toggle button -->
			<button class="grid-toggle" on:click={goToGrid} title={$t.controlPanel.backToGrid}>
				<Icon icon="mdi:apps" class="w-5 h-5" />
			</button>

			<!-- Search -->
			<div class="sidebar-search">
				<Icon icon="mdi:magnify" class="w-4 h-4 text-slate-400" />
				<input type="text" placeholder={$t.common.search} bind:value={searchQuery} class="search-input" />
			</div>

			<!-- Sidebar navigation -->
			<nav class="sidebar-nav">
				{#each filteredSidebarCategories as category}
					<div class="sidebar-category">
						<span class="sidebar-category-title">{getTranslation($t, category.titleKey)}</span>
						{#each category.items as item}
							<button
								class="sidebar-item"
								class:active={selectedItem === item.id}
								on:click={() => selectItem(item.id)}
							>
								<Icon icon={item.icon} class="w-5 h-5 {item.iconColor}" />
								<span>{getTranslation($t, item.labelKey)}</span>
							</button>
						{/each}
					</div>
				{/each}
			</nav>
		</aside>

		<!-- Main Content -->
		<main class="content">
			{#if selectedItem === 'users'}
				<!-- User Management -->
				<UserManager />
			{:else if selectedItem === 'files'}
				<!-- File Services -->
				<FileService />
			{:else if selectedItem === 'terminal'}
				<!-- Terminal / SSH Settings -->
				<TerminalSettings />
			{:else if selectedItem === 'printer'}
				<!-- Printer Sharing (CUPS) -->
				<PrinterSettings />
			{:else if selectedItem === 'network'}
				<!-- Network Settings -->
				<NetworkSettings />
			{:else if selectedItem === 'update'}
				<!-- System Update -->
				<UpdateSettings />
			{:else if selectedItem === 'hardware'}
				<HardwarePower />
			{:else if selectedItem === 'security'}
				<SecuritySettings />
			{:else if selectedItem === 'device'}
				<DeviceConnection />
			{:else if selectedItem === 'personalization'}
				<Personalization />
			{:else if selectedItem === 'time'}
				<!-- Time & Language -->
				<TimeLanguage />
			{:else if selectedItem === 'about'}
				<div class="settings-page">
					<SettingsHeader title={$t.controlPanel.items.about} description={$t.settingsUi.descriptions.about} icon="mdi:information-outline" />
					<SettingsTabs label={$t.controlPanel.items.about} items={tabs} value={activeTab} onchange={(value) => (activeTab = value)} />

					{#if activeTab === 'general'}
						<SettingsSection title={$t.controlPanel.about.device} icon="mdi:server">
							<dl class="settings-details">
								<dt>{$t.controlPanel.about.deviceName}</dt><dd>{systemInfo.deviceName}</dd>
								<dt>{$t.controlPanel.about.systemVersion}</dt><dd>{systemInfo.systemVersion}</dd>
								<dt>{$t.controlPanel.about.deviceOwner}</dt><dd>{systemInfo.deviceOwner}</dd>
								<dt>{$t.controlPanel.about.model}</dt><dd>{systemInfo.model}</dd>
								<dt>{$t.controlPanel.about.serialNumber}</dt><dd class="settings-mono">{systemInfo.serialNumber}</dd>
								<dt>{$t.controlPanel.about.lastStartup}</dt><dd>{systemInfo.lastStartup}</dd>
								<dt>{$t.controlPanel.about.powerTime}</dt><dd>{systemInfo.powerTime}</dd>
							</dl>
						</SettingsSection>
						<SettingsSection title={$t.controlPanel.about.hardware} icon="mdi:chip">
							<dl class="settings-details">
								<dt>{$t.controlPanel.about.cpu}</dt>
								<dd>{[systemInfo.cpu.model, systemInfo.cpu.freq, `${systemInfo.cpu.cores} ${$t.controlPanel.about.cores}`, systemInfo.cpu.temp].filter(Boolean).join(' · ')}</dd>
								<dt>{$t.controlPanel.about.memory}</dt><dd>{systemInfo.memory.size}</dd>
							</dl>
						</SettingsSection>
						<SettingsSection title={$t.controlPanel.about.network} icon="mdi:web">
							<dl class="settings-details">
								<dt>{systemInfo.network.interface}</dt>
								<dd class="settings-mono">{[systemInfo.network.ip, systemInfo.network.speed].filter(Boolean).join(' · ')}</dd>
								<dt>{$t.controlPanel.about.macAddress}</dt>
								<dd class="settings-mono">{systemInfo.network.mac} · {systemInfo.network.subnet}</dd>
							</dl>
						</SettingsSection>
					{:else if aboutLoading}
						<SettingsState kind="loading" message={$t.common.loading} />
					{:else if aboutError}
						<SettingsState kind="error" message={aboutError} onretry={() => loadAboutTab(activeTab)} />
					{:else if activeTab === 'storage'}
						<SettingsSection title={$t.controlPanel.about.pools} icon="mdi:harddisk">
							{#if aboutPools.length === 0}
								<SettingsState kind="empty" message={$t.dashboard.noPools} />
							{:else}
								<div class="settings-table-wrap">
									<table class="settings-table">
										<tbody>
											{#each aboutPools as pool (pool.id)}
												<tr>
													<td><strong>{pool.name}</strong> · {pool.raid_type}</td>
													<td>{formatBytes(pool.used_size)} / {formatBytes(pool.total_size)}</td>
													<td><span class="settings-badge {statusTone(pool.status)}">{$t.storageManager.status[pool.status] ?? pool.status}</span></td>
												</tr>
											{/each}
										</tbody>
									</table>
								</div>
							{/if}
						</SettingsSection>
						<SettingsSection title={$t.controlPanel.about.volumes} icon="mdi:database">
							{#if aboutVolumes.length === 0}
								<SettingsState kind="empty" message={$t.controlPanel.about.noVolumes} />
							{:else}
								<div class="settings-table-wrap">
									<table class="settings-table">
										<tbody>
											{#each aboutVolumes as vol (vol.id)}
												<tr>
													<td><strong>{vol.name}</strong> · {vol.fs_type}</td>
													<td class="settings-mono">{vol.mount_point}</td>
													<td><span class="settings-badge {statusTone(vol.status)}">{$t.storageManager.status[vol.status] ?? vol.status}</span></td>
												</tr>
											{/each}
										</tbody>
									</table>
								</div>
							{/if}
						</SettingsSection>
					{:else if activeTab === 'service'}
						<SettingsSection title={$t.dashboard.services} icon="mdi:cog-outline">
							{#if aboutServices.length === 0}
								<SettingsState kind="empty" message={$t.common.noData} />
							{:else}
								<div class="settings-table-wrap">
									<table class="settings-table">
										<tbody>
											{#each aboutServices as svc (svc.name)}
												<tr>
													<td class="settings-mono">{svc.name}</td>
													<td>
														<span class="settings-badge" class:success={svc.running}>{svc.running ? $t.dashboard.running : $t.dashboard.stopped}</span>
														{#if !svc.enabled}<span class="settings-badge">{$t.common.disabled}</span>{/if}
													</td>
												</tr>
											{/each}
										</tbody>
									</table>
								</div>
							{/if}
						</SettingsSection>
					{/if}
				</div>
			{/if}
		</main>
	{:else}
		<!-- Grid View -->
		<div class="grid-view">
			<!-- Search bar for grid -->
			<div class="grid-search-bar">
				<button class="grid-toggle-btn" on:click={goToGrid}>
					<Icon icon="mdi:apps" class="w-5 h-5" />
				</button>
				<div class="grid-search">
					<Icon icon="mdi:magnify" class="w-4 h-4 text-slate-400" />
					<input type="text" placeholder={$t.common.search} bind:value={searchQuery} class="search-input" />
				</div>
			</div>

			<!-- Grid content -->
			<div class="grid-content">
				{#each filteredGridCategories as category}
					<section class="grid-category">
						<h3 class="grid-category-title">{getTranslation($t, category.titleKey)}</h3>
						<div class="icon-grid">
							{#each category.items as item}
								<button class="icon-item" on:click={() => selectItem(item.id)}>
									<div class="icon-wrapper" style={gradientStyle(item.gradient)}>
										<Icon icon={item.icon} class="w-7 h-7 text-white" />
										{#if item.badge}
											<span class="icon-badge">{item.badge}</span>
										{/if}
									</div>
									<span class="icon-label">{getTranslation($t, item.labelKey)}</span>
								</button>
							{/each}
						</div>
					</section>
				{/each}
			</div>
		</div>
	{/if}
</div>

<ConfirmModal
	visible={pendingNavigation !== null}
	title={$t.settingsUi.leaveTitle}
	message={$t.settingsUi.leaveMessage}
	confirmLabel={$t.settingsUi.leaveConfirm}
	cancelLabel={$t.settingsUi.stay}
	danger
	on:confirm={confirmNavigation}
	on:cancel={() => (pendingNavigation = null)}
/>

<style>
	.control-panel {
		display: flex;
		height: 100%;
		background: white;
	}

	/* Sidebar */
	.sidebar {
		width: 200px;
		border-right: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
		background: #fafafa;
		flex-shrink: 0;
	}

	.grid-toggle {
		padding: 12px 16px;
		display: flex;
		align-items: center;
		color: #6b7280;
		border-bottom: 1px solid #e5e7eb;
		transition: all 0.15s ease;
	}

	.grid-toggle:hover {
		background: #f3f4f6;
		color: #3b82f6;
	}

	.sidebar-search {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 12px;
		padding: 8px 12px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
	}

	.search-input {
		flex: 1;
		border: none;
		background: transparent;
		font-size: 13px;
		color: #1f2937;
		outline: none;
	}

	.search-input::placeholder {
		color: #9ca3af;
	}

	.sidebar-nav {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

	.sidebar-category {
		margin-bottom: 8px;
	}

	.sidebar-category-title {
		display: block;
		padding: 8px 16px 4px;
		font-size: 11px;
		font-weight: 500;
		color: #9ca3af;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.sidebar-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 16px;
		font-size: 13px;
		color: #374151;
		transition: all 0.15s ease;
		text-align: left;
		border: none;
		background: transparent;
	}

	.sidebar-item:hover {
		background: #f3f4f6;
	}

	.sidebar-item.active {
		background: #eff6ff;
		color: #2563eb;
	}

	/* Main Content */
	.content {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.tab {
		padding: 16px 0;
		font-size: 14px;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		transition: all 0.15s ease;
		background: transparent;
		border-top: none;
		border-left: none;
		border-right: none;
	}

	.tab:hover {
		color: #374151;
	}

	.tab.active {
		color: #2563eb;
		border-bottom-color: #2563eb;
	}

	/* Grid View */
	.grid-view {
		flex: 1;
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.grid-search-bar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		border-bottom: 1px solid #e5e7eb;
		background: #fafafa;
	}

	.grid-toggle-btn {
		padding: 8px;
		color: #6b7280;
		border-radius: 8px;
	}

	.grid-toggle-btn:hover {
		background: #f3f4f6;
	}

	.grid-search {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		width: 200px;
	}

	.grid-content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.grid-category {
		margin-bottom: 32px;
	}

	.grid-category-title {
		font-size: 13px;
		font-weight: 500;
		color: #6b7280;
		margin-bottom: 16px;
	}

	.icon-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 16px;
	}

	.icon-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		width: 90px;
		padding: 12px 8px;
		border-radius: 12px;
		transition: all 0.15s ease;
		border: none;
		background: transparent;
		cursor: pointer;
	}

	.icon-item:hover {
		background: #f3f4f6;
	}

	.icon-wrapper {
		position: relative;
		width: 56px;
		height: 56px;
		border-radius: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.icon-badge {
		position: absolute;
		top: -4px;
		right: -4px;
		width: 18px;
		height: 18px;
		background: #ef4444;
		border-radius: 50%;
		font-size: 10px;
		font-weight: 600;
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.icon-label {
		font-size: 12px;
		color: #4b5563;
		text-align: center;
		line-height: 1.3;
	}


</style>
