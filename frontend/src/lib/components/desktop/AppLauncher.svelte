<script lang="ts">
	import Icon from '@iconify/svelte';
	import { openWindow } from '$stores/windows';
	import { allApps, addToDesktop, removeFromDesktop, pinnedAppIds, type DesktopApp } from '$stores/desktop';
	import { createEventDispatcher } from 'svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';
	import { t } from '$lib/i18n';

	export let visible = false;

	const dispatch = createEventDispatcher();

	interface AppCategory {
		titleKey: string;
		appIds: string[];
	}

	// Category definitions with app IDs
	const categoryDefs: AppCategory[] = [
		{
			titleKey: 'system',
			appIds: ['control-panel']
		},
		{
			titleKey: 'storageFiles',
			appIds: ['storage', 'file-manager', 'shares']
		},
		{
			titleKey: 'services',
			appIds: ['app-center', 'docker', 'terminal']
		}
	];

	// Reactive categories based on allApps store
	$: categories = categoryDefs.map((cat) => ({
		titleKey: cat.titleKey,
		apps: $allApps.filter((app) => cat.appIds.includes(app.id) || (app.isInstalled && cat.titleKey === 'services'))
	})).filter((cat) => cat.apps.length > 0);

	let searchQuery = '';

	let contextMenu = {
		visible: false,
		x: 0,
		y: 0,
		targetApp: null as DesktopApp | null
	};

	$: filteredCategories = searchQuery
		? categories
				.map((cat) => ({
					...cat,
					apps: cat.apps.filter((app) => app.label.toLowerCase().includes(searchQuery.toLowerCase()))
				}))
				.filter((cat) => cat.apps.length > 0)
		: categories;

	function launchApp(app: DesktopApp) {
		openWindow({
			id: app.id,
			title: app.label,
			icon: app.icon,
			component: app.component,
			x: 150 + Math.random() * 100,
			y: 80 + Math.random() * 50,
			width: app.window?.width ?? 900,
			height: app.window?.height ?? 600
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
		contextMenu = {
			visible: true,
			x: e.clientX,
			y: e.clientY,
			targetApp: app
		};
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
				action: () => {
					if (app) launchApp(app);
				}
			},
			isOnDesktop
				? {
						label: translations.common.removeFromDesktop,
						icon: 'mdi:close-circle-outline',
						action: () => {
							if (app) removeFromDesktop(app.id);
						},
						danger: true
					}
				: {
						label: translations.common.addToDesktop,
						icon: 'mdi:monitor-screenshot',
						action: () => {
							if (app) addToDesktop(app.id);
						}
					}
		];
	}

	$: contextMenuItems = getContextMenuItems(contextMenu.targetApp, $pinnedAppIds, $t);
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="overlay" on:click={() => dispatch('close')}></div>

	<div class="launcher">
		<!-- Search -->
		<div class="search-bar">
			<Icon icon="mdi:magnify" class="w-5 h-5 text-slate-400" />
			<input type="text" placeholder={$t.common.searchApplications} bind:value={searchQuery} autofocus />
		</div>

		<!-- App Grid -->
		<div class="app-list">
			{#each filteredCategories as category}
				<div class="category">
					<h3 class="category-title">{$t.desktop.appLauncher.categories[category.titleKey]}</h3>
					<div class="app-grid">
						{#each category.apps as app}
							<button
								class="app-item"
								class:on-desktop={$pinnedAppIds.includes(app.id)}
								on:click={() => launchApp(app)}
								on:contextmenu={(e) => handleContextMenu(e, app)}
							>
								<div class="app-icon bg-gradient-to-br {app.gradient}">
									<Icon icon={app.icon} class="w-7 h-7 text-white" />
								</div>
								<span class="app-label">{app.label}</span>
								{#if $pinnedAppIds.includes(app.id)}
									<div class="desktop-indicator" title="On desktop">
										<Icon icon="mdi:monitor" class="w-3 h-3" />
									</div>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{/each}

			{#if filteredCategories.length === 0}
				<div class="empty-state">
					<Icon icon="mdi:magnify" class="w-12 h-12 text-slate-300" />
					<p>{$t.common.noApplicationsFound}</p>
				</div>
			{/if}
		</div>
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
		width: 480px;
		max-height: calc(100vh - 120px);
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		z-index: 201;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.search-bar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px 20px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
	}

	.search-bar input {
		flex: 1;
		border: none;
		background: transparent;
		font-size: 15px;
		color: #1e293b;
		outline: none;
	}

	.search-bar input::placeholder {
		color: #94a3b8;
	}

	.app-list {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
	}

	.category {
		margin-bottom: 20px;
	}

	.category:last-child {
		margin-bottom: 0;
	}

	.category-title {
		font-size: 12px;
		font-weight: 600;
		color: #64748b;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 12px;
		padding-left: 4px;
	}

	.app-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
	}

	.app-item {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px 8px;
		border-radius: 12px;
		transition: all 0.15s ease;
		border: none;
		background: transparent;
		cursor: pointer;
	}

	.app-item:hover {
		background: rgba(0, 0, 0, 0.05);
	}

	.app-item:active {
		transform: scale(0.95);
	}

	.app-icon {
		width: 52px;
		height: 52px;
		border-radius: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		transition: all 0.15s ease;
	}

	.app-item:hover .app-icon {
		transform: scale(1.05);
		box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15);
	}

	.app-label {
		font-size: 11px;
		color: #475569;
		text-align: center;
		line-height: 1.3;
		max-width: 80px;
	}

	.desktop-indicator {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: rgba(59, 130, 246, 0.15);
		color: #3b82f6;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px 0;
		gap: 12px;
	}

	.empty-state p {
		font-size: 14px;
		color: #94a3b8;
	}
</style>
