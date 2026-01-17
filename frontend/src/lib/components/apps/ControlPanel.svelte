<script lang="ts">
	import Icon from '@iconify/svelte';
	import UserManager from './UserManager.svelte';
	import TimeLanguage from './TimeLanguage.svelte';
	import { t } from '$lib/i18n';

	type ViewMode = 'grid' | 'detail';
	let viewMode: ViewMode = 'grid';

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
				{ id: 'files', icon: 'mdi:folder', labelKey: 'controlPanel.items.fileService', iconColor: 'text-orange-500' },
				{ id: 'device', icon: 'mdi:monitor-screenshot', labelKey: 'controlPanel.items.deviceConnection', iconColor: 'text-slate-500' },
				{ id: 'domain', icon: 'mdi:domain', labelKey: 'controlPanel.items.domainLdap', iconColor: 'text-blue-600' },
				{ id: 'terminal', icon: 'mdi:console-line', labelKey: 'controlPanel.items.terminal', iconColor: 'text-purple-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.general',
			items: [
				{ id: 'hardware', icon: 'mdi:chip', labelKey: 'controlPanel.items.hardwarePower', iconColor: 'text-emerald-500' },
				{ id: 'time', icon: 'mdi:earth', labelKey: 'controlPanel.items.timeLanguage', iconColor: 'text-cyan-500', component: 'TimeLanguage' },
				{ id: 'network', icon: 'mdi:wifi', labelKey: 'controlPanel.items.network', iconColor: 'text-blue-500' },
				{ id: 'security', icon: 'mdi:shield-check', labelKey: 'controlPanel.items.security', iconColor: 'text-green-500' },
				{ id: 'indexing', icon: 'mdi:database-search', labelKey: 'controlPanel.items.indexingService', iconColor: 'text-slate-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.service',
			items: [
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
				{ id: 'domain', icon: 'mdi:domain', labelKey: 'controlPanel.items.domainLdap', gradient: 'from-blue-500 to-blue-600' },
				{ id: 'terminal', icon: 'mdi:console-line', labelKey: 'controlPanel.items.terminal', gradient: 'from-purple-400 to-purple-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.general',
			items: [
				{ id: 'hardware', icon: 'mdi:chip', labelKey: 'controlPanel.items.hardwarePower', gradient: 'from-emerald-400 to-emerald-500' },
				{ id: 'time', icon: 'mdi:earth', labelKey: 'controlPanel.items.timeLanguage', gradient: 'from-cyan-400 to-cyan-500' },
				{ id: 'network', icon: 'mdi:wifi', labelKey: 'controlPanel.items.network', gradient: 'from-blue-400 to-blue-500' },
				{ id: 'security', icon: 'mdi:shield-check', labelKey: 'controlPanel.items.security', gradient: 'from-green-400 to-green-500' },
				{ id: 'indexing', icon: 'mdi:database-search', labelKey: 'controlPanel.items.indexingService', gradient: 'from-slate-400 to-slate-500' }
			]
		},
		{
			titleKey: 'controlPanel.categories.service',
			items: [
				{ id: 'about', icon: 'mdi:information', labelKey: 'controlPanel.items.about', gradient: 'from-teal-400 to-teal-500' }
			]
		}
	];

	let selectedItem: string | null = null;
	let activeTab = 'general';

	// Mock system data
	const systemInfo = {
		deviceName: 'PiNAS-001',
		systemVersion: '1.0.0',
		deviceOwner: 'admin',
		model: 'Raspberry Pi 5',
		serialNumber: 'RPI5-XXXX-XXXX',
		lastStartup: new Date().toLocaleString(),
		powerTime: '02 hours 15 minutes',
		cpu: {
			model: 'ARM Cortex-A76',
			freq: '2400',
			cores: '4',
			threads: '4',
			temp: '45'
		},
		memory: {
			model: 'LPDDR4X',
			size: '8 GB',
			speed: '4267 MHz'
		},
		network: {
			interface: 'eth0',
			ip: '192.168.1.100',
			speed: '1Gbps',
			mtu: '1500',
			mac: 'dc:a6:32:xx:xx:xx',
			subnet: '255.255.255.0'
		}
	};

	function selectItem(id: string) {
		selectedItem = id;
		viewMode = 'detail';
	}

	function goToGrid() {
		viewMode = 'grid';
		selectedItem = null;
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
		{ key: 'general', label: $t.controlPanel.tabs.general },
		{ key: 'storage', label: $t.controlPanel.tabs.storage },
		{ key: 'service', label: $t.controlPanel.tabs.service },
		{ key: 'deviceAnalysis', label: $t.controlPanel.tabs.deviceAnalysis }
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
			{:else if selectedItem === 'time'}
				<!-- Time & Language -->
				<TimeLanguage />
			{:else if selectedItem === 'about'}
				<!-- About / General Info -->
				<div class="content-header">
					<nav class="content-tabs">
						{#each tabs as tab}
							<button class="tab" class:active={activeTab === tab.key} on:click={() => (activeTab = tab.key)}>
								{tab.label}
							</button>
						{/each}
					</nav>
				</div>

				{#if activeTab === 'general'}
					<div class="info-grid">
						<!-- Device Info Card -->
						<div class="info-card device-info">
							<div class="info-row">
								<div class="info-block">
									<span class="info-label">{$t.controlPanel.about.deviceName}</span>
									<span class="info-value">
										{systemInfo.deviceName}
										<button class="edit-btn">
											<Icon icon="mdi:pencil" class="w-4 h-4" />
										</button>
									</span>
								</div>
								<div class="info-block">
									<span class="info-label">{$t.controlPanel.about.systemVersion}</span>
									<span class="info-value">{systemInfo.systemVersion}</span>
								</div>
							</div>
							<div class="info-row">
								<div class="info-block">
									<span class="info-label">{$t.controlPanel.about.deviceOwner}</span>
									<span class="info-value">{systemInfo.deviceOwner}</span>
								</div>
							</div>
							<div class="device-image">
								<Icon icon="mdi:server" class="w-24 h-24 text-slate-300" />
							</div>
						</div>

						<!-- Device Details Card -->
						<div class="info-card">
							<div class="card-header">
								<Icon icon="mdi:information-outline" class="w-5 h-5 text-slate-500" />
								<span>{$t.controlPanel.about.device}</span>
							</div>
							<div class="card-content">
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.model}</span>
									<span class="info-value">{systemInfo.model}</span>
								</div>
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.serialNumber}</span>
									<span class="info-value">{systemInfo.serialNumber}</span>
								</div>
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.lastStartup}</span>
									<span class="info-value">{systemInfo.lastStartup}</span>
								</div>
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.powerTime}</span>
									<span class="info-value">{systemInfo.powerTime}</span>
								</div>
							</div>
						</div>

						<!-- Hardware Card -->
						<div class="info-card">
							<div class="card-header">
								<Icon icon="mdi:chip" class="w-5 h-5 text-slate-500" />
								<span>{$t.controlPanel.about.hardware}</span>
							</div>
							<div class="card-content">
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.cpu}</span>
									<span class="info-value hardware-specs">
										{systemInfo.cpu.model}
										<span class="spec-divider">|</span>
										{systemInfo.cpu.freq}
										<span class="spec-divider">|</span>
										{systemInfo.cpu.cores} {$t.controlPanel.about.cores}
										<span class="spec-divider">|</span>
										{systemInfo.cpu.threads} {$t.controlPanel.about.threads}
										<span class="spec-divider">|</span>
										{systemInfo.cpu.temp}°C
									</span>
								</div>
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.memory}</span>
									<span class="info-value hardware-specs">
										{systemInfo.memory.model}
										<span class="spec-divider">|</span>
										{systemInfo.memory.size}
										<span class="spec-divider">|</span>
										{systemInfo.memory.speed}
									</span>
								</div>
							</div>
						</div>

						<!-- Network Card -->
						<div class="info-card">
							<div class="card-header">
								<Icon icon="mdi:web" class="w-5 h-5 text-slate-500" />
								<span>{$t.controlPanel.about.network}</span>
								<button class="expand-btn">
									<Icon icon="mdi:chevron-right" class="w-5 h-5" />
								</button>
							</div>
							<div class="card-content">
								<div class="info-line">
									<span class="info-label">{systemInfo.network.interface}</span>
									<span class="info-value hardware-specs">
										{systemInfo.network.ip}
										<span class="spec-divider">|</span>
										{systemInfo.network.speed}
										<span class="spec-divider">|</span>
										MTU{systemInfo.network.mtu}
									</span>
								</div>
								<div class="info-line">
									<span class="info-label">{$t.controlPanel.about.macAddress}</span>
									<span class="info-value hardware-specs">
										{systemInfo.network.mac}
										<span class="spec-divider">|</span>
										{systemInfo.network.subnet}
									</span>
								</div>
							</div>
						</div>
					</div>
				{:else}
					<div class="empty-tab">
						<Icon icon="mdi:cog" class="w-16 h-16 text-slate-200" />
						<p>{$t.controlPanel.contentFor.replace('{tab}', tabs.find(t => t.key === activeTab)?.label || activeTab)}</p>
					</div>
				{/if}
			{:else}
				<!-- Placeholder for other sections -->
				<div class="placeholder-content">
					<Icon icon="mdi:cog" class="w-16 h-16 text-slate-200" />
					<h3>
						{sidebarCategories
							.flatMap((c) => c.items)
							.find((i) => i.id === selectedItem)?.labelKey
							? getTranslation($t, sidebarCategories.flatMap((c) => c.items).find((i) => i.id === selectedItem)?.labelKey || '')
							: ''}
					</h3>
					<p>{$t.controlPanel.underDevelopment}</p>
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
									<div class="icon-wrapper bg-gradient-to-br {item.gradient}">
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

	.content-header {
		border-bottom: 1px solid #e5e7eb;
		padding: 0 24px;
		background: white;
	}

	.content-tabs {
		display: flex;
		gap: 32px;
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

	/* Info Grid (About/General) */
	.info-grid {
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.info-card {
		background: #f9fafb;
		border-radius: 12px;
		padding: 20px;
	}

	.info-card.device-info {
		position: relative;
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 16px;
	}

	.device-image {
		position: absolute;
		right: 24px;
		top: 50%;
		transform: translateY(-50%);
	}

	.info-row {
		display: flex;
		gap: 48px;
	}

	.info-block {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 16px;
		font-size: 14px;
		font-weight: 500;
		color: #1f2937;
	}

	.card-header .expand-btn {
		margin-left: auto;
		color: #9ca3af;
	}

	.card-content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.info-line {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.info-label {
		font-size: 12px;
		color: #9ca3af;
	}

	.info-value {
		font-size: 14px;
		color: #1f2937;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.info-value.hardware-specs {
		font-weight: 400;
	}

	.spec-divider {
		color: #d1d5db;
		margin: 0 4px;
	}

	.edit-btn {
		color: #9ca3af;
		padding: 4px;
		border-radius: 4px;
	}

	.edit-btn:hover {
		background: #e5e7eb;
		color: #6b7280;
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

	/* Placeholder */
	.placeholder-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		color: #9ca3af;
	}

	.placeholder-content h3 {
		font-size: 18px;
		font-weight: 500;
		color: #374151;
	}

	.placeholder-content p {
		font-size: 14px;
	}

	.empty-tab {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		color: #9ca3af;
	}

	.empty-tab p {
		font-size: 14px;
	}
</style>
