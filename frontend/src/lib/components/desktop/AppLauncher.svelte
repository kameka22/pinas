<script lang="ts">
	import Icon from '@iconify/svelte';
	import { openWindow } from '$stores/windows';
	import { allApps, addToDesktop, removeFromDesktop, pinnedAppIds, type DesktopApp } from '$stores/desktop';
	import { createEventDispatcher, tick } from 'svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import { t } from '$lib/i18n';
	import { gradientStyle } from '$lib/utils/gradient';

	export let visible = false;

	const dispatch = createEventDispatcher();

	interface CategoryDef {
		key: string;
		icon: string;
		appIds: string[];
	}

	const categoryDefs: CategoryDef[] = [
		{ key: 'system', icon: 'mdi:cog', appIds: ['control-panel', 'process-manager'] },
		{ key: 'storageFiles', icon: 'mdi:folder-multiple', appIds: ['storage', 'file-manager', 'shares'] },
		{ key: 'services', icon: 'mdi:apps', appIds: ['app-center', 'docker', 'terminal', 'kodi'] }
	];

	const allCategoryKeys = ['all', ...categoryDefs.map(c => c.key)];

	let selectedCategory = 'all';
	let searchQuery = '';
	let slideDirection: 'down' | 'up' = 'down';
	let slideKey = 0;

	let contextMenu = {
		visible: false,
		x: 0,
		y: 0,
		targetApp: null as DesktopApp | null
	};

	// Build categories with their apps
	$: categories = categoryDefs.map((cat) => ({
		key: cat.key,
		icon: cat.icon,
		apps: $allApps.filter((app) => cat.appIds.includes(app.id) || (app.isInstalled && cat.key === 'services'))
	})).filter((cat) => cat.apps.length > 0);

	// Apps for the selected category
	$: displayedApps = (() => {
		let apps: DesktopApp[];
		if (selectedCategory === 'all') {
			apps = categories.flatMap((cat) => cat.apps);
		} else {
			apps = categories.find((c) => c.key === selectedCategory)?.apps || [];
		}
		if (searchQuery) {
			const q = searchQuery.toLowerCase();
			apps = apps.filter((app) => getAppLabel(app).toLowerCase().includes(q));
		}
		return apps;
	})();

	// Count per category
	$: categoryAppCount = (key: string) => {
		if (key === 'all') return $allApps.length;
		return categories.find((c) => c.key === key)?.apps.length || 0;
	};

	function selectCategory(key: string) {
		if (key === selectedCategory) return;
		const oldIdx = allCategoryKeys.indexOf(selectedCategory);
		const newIdx = allCategoryKeys.indexOf(key);
		slideDirection = newIdx > oldIdx ? 'down' : 'up';
		slideKey++;
		selectedCategory = key;
	}

	function getAppLabel(app: DesktopApp): string {
		if (app.labelKey && $t.apps[app.labelKey as keyof typeof $t.apps]) {
			return $t.apps[app.labelKey as keyof typeof $t.apps];
		}
		return app.label;
	}

	function launchApp(app: DesktopApp) {
		openWindow({
			id: app.id,
			title: getAppLabel(app),
			icon: app.icon,
			component: app.component,
			x: 150 + Math.random() * 100,
			y: 80 + Math.random() * 50,
			width: app.window?.width ?? 900,
			height: app.window?.height ?? 600,
			appConfig: app.appConfig,
			gradient: app.gradient
		});
		dispatch('close');
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			dispatch('close');
		}
	}

	function handleContextMenu(e: MouseEvent, app: DesktopApp) {
		e.preventDefault();
		e.stopPropagation();
		contextMenu = { visible: true, x: e.clientX, y: e.clientY, targetApp: app };
	}

	function closeContextMenu() {
		contextMenu = { ...contextMenu, visible: false, targetApp: null };
	}

	function getContextMenuItems(app: DesktopApp | null, pinnedIds: string[], translations: typeof $t) {
		if (!app) return [];
		const isOnDesktop = pinnedIds.includes(app.id);
		return [
			{
				label: translations.common.open,
				icon: 'mdi:open-in-app',
				action: () => { if (app) launchApp(app); }
			},
			isOnDesktop
				? {
						label: translations.common.removeFromDesktop,
						icon: 'mdi:close-circle-outline',
						action: () => { if (app) removeFromDesktop(app.id); },
						danger: true
					}
				: {
						label: translations.common.addToDesktop,
						icon: 'mdi:monitor-screenshot',
						action: () => { if (app) addToDesktop(app.id); }
					}
		];
	}

	$: contextMenuItems = getContextMenuItems(contextMenu.targetApp, $pinnedAppIds, $t);

	// Reset state when opening
	$: if (visible) {
		selectedCategory = 'all';
		searchQuery = '';
		slideKey = 0;
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="overlay" on:click={() => dispatch('close')}></div>

	<div class="launcher">
		<!-- Left: Categories -->
		<nav class="sidebar">
			<div class="category-list">
				<button
					class="category-item"
					class:active={selectedCategory === 'all'}
					on:click={() => selectCategory('all')}
				>
					<Icon icon="mdi:view-grid" class="w-[18px] h-[18px]" />
					<span>{$t.desktop.appLauncher.categories.all}</span>
					<span class="category-count">{categoryAppCount('all')}</span>
				</button>

				<div class="category-divider"></div>

				{#each categories as cat}
					<button
						class="category-item"
						class:active={selectedCategory === cat.key}
						on:click={() => selectCategory(cat.key)}
					>
						<Icon icon={cat.icon} class="w-[18px] h-[18px]" />
						<span>{$t.desktop.appLauncher.categories[cat.key]}</span>
						<span class="category-count">{cat.apps.length}</span>
					</button>
				{/each}
			</div>
		</nav>

		<!-- Right: App grid -->
		<main class="content">
			<!-- Search -->
			<div class="search-bar">
				<Icon icon="mdi:magnify" class="w-4 h-4 text-slate-400" />
				<input
					type="text"
					placeholder={$t.common.searchApplications}
					bind:value={searchQuery}
					autofocus
				/>
				{#if searchQuery}
					<button class="search-clear" on:click={() => searchQuery = ''}>
						<Icon icon="mdi:close" class="w-3.5 h-3.5" />
					</button>
				{/if}
			</div>

			<!-- Apps with slide transition -->
			<div class="app-scroll">
				{#key slideKey}
					<div class="slide-panel" class:slide-from-bottom={slideDirection === 'down'} class:slide-from-top={slideDirection === 'up'}>
						{#if displayedApps.length > 0}
							<div class="app-grid">
								{#each displayedApps as app}
									<button
										class="app-item"
										on:click={() => launchApp(app)}
										on:contextmenu={(e) => handleContextMenu(e, app)}
									>
										<div class="app-icon" style={gradientStyle(app.gradient)}>
											<Icon icon={app.icon} class="w-7 h-7 text-white" />
										</div>
										<span class="app-label">{getAppLabel(app)}</span>
										{#if $pinnedAppIds.includes(app.id)}
											<div class="desktop-dot"></div>
										{/if}
									</button>
								{/each}
							</div>
						{:else}
							<div class="empty-state">
								<Icon icon="mdi:magnify" class="w-10 h-10 text-slate-300" />
								<p>{$t.common.noApplicationsFound}</p>
							</div>
						{/if}
					</div>
				{/key}
			</div>
		</main>
	</div>
{/if}

<ContextMenu
	visible={contextMenu.visible}
	x={contextMenu.x}
	y={contextMenu.y}
	items={contextMenuItems}
	on:close={closeContextMenu}
/>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.3);
		z-index: 200;
	}

	.launcher {
		position: fixed;
		top: 48px;
		left: 8px;
		width: 540px;
		height: 420px;
		background: rgba(255, 255, 255, 0.96);
		backdrop-filter: blur(24px);
		border-radius: 16px;
		box-shadow:
			0 25px 50px -12px rgba(0, 0, 0, 0.2),
			0 0 0 1px rgba(0, 0, 0, 0.05);
		z-index: 201;
		display: flex;
		overflow: hidden;
	}

	/* ---- Sidebar ---- */
	.sidebar {
		width: 170px;
		background: rgba(241, 245, 249, 0.7);
		border-right: 1px solid rgba(0, 0, 0, 0.06);
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
		padding-top: 4px;
	}

	.category-list {
		padding: 8px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.category-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 9px 12px;
		border: none;
		background: transparent;
		border-radius: 8px;
		color: #64748b;
		font-size: 13px;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.category-item:hover {
		background: rgba(0, 0, 0, 0.05);
		color: #334155;
	}

	.category-item.active {
		background: white;
		color: #1e293b;
		font-weight: 500;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
	}

	.category-item span:first-of-type {
		flex: 1;
		text-align: left;
	}

	.category-count {
		font-size: 11px;
		font-weight: 500;
		color: #94a3b8;
		background: rgba(0, 0, 0, 0.04);
		padding: 1px 7px;
		border-radius: 10px;
		min-width: 22px;
		text-align: center;
	}

	.category-item.active .category-count {
		color: #3b82f6;
		background: rgba(59, 130, 246, 0.1);
	}

	.category-divider {
		height: 1px;
		background: rgba(0, 0, 0, 0.06);
		margin: 4px 12px;
	}

	/* ---- Content ---- */
	.content {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.search-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 16px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.06);
		flex-shrink: 0;
	}

	.search-bar input {
		flex: 1;
		border: none;
		background: transparent;
		font-size: 13px;
		color: #1e293b;
		outline: none;
	}

	.search-bar input::placeholder {
		color: #94a3b8;
	}

	.search-clear {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border: none;
		background: rgba(0, 0, 0, 0.06);
		border-radius: 50%;
		color: #94a3b8;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.search-clear:hover {
		background: rgba(0, 0, 0, 0.1);
		color: #64748b;
	}

	.app-scroll {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		position: relative;
	}

	/* ---- Slide transition ---- */
	.slide-panel {
		padding: 16px;
		animation-duration: 0.22s;
		animation-timing-function: cubic-bezier(0.16, 1, 0.3, 1);
		animation-fill-mode: both;
	}

	.slide-from-bottom {
		animation-name: slideInFromBottom;
	}

	.slide-from-top {
		animation-name: slideInFromTop;
	}

	@keyframes slideInFromBottom {
		from {
			opacity: 0;
			transform: translateY(30px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes slideInFromTop {
		from {
			opacity: 0;
			transform: translateY(-30px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.app-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 4px;
	}

	.app-item {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px 6px;
		border-radius: 12px;
		transition: all 0.12s ease;
		border: none;
		background: transparent;
		cursor: pointer;
	}

	.app-item:hover {
		background: rgba(0, 0, 0, 0.04);
	}

	.app-item:active {
		transform: scale(0.95);
	}

	.app-icon {
		width: 48px;
		height: 48px;
		border-radius: 13px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 3px 10px rgba(0, 0, 0, 0.1);
		transition: all 0.15s ease;
	}

	.app-item:hover .app-icon {
		transform: scale(1.06);
		box-shadow: 0 5px 14px rgba(0, 0, 0, 0.14);
	}

	.app-label {
		font-size: 11px;
		color: #475569;
		text-align: center;
		line-height: 1.3;
		max-width: 76px;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.desktop-dot {
		position: absolute;
		top: 10px;
		right: 10px;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #3b82f6;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		min-height: 200px;
		gap: 10px;
	}

	.empty-state p {
		font-size: 13px;
		color: #94a3b8;
	}
</style>
