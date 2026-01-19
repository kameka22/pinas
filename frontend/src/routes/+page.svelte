<script lang="ts">
	import { onMount } from 'svelte';
	import { openWindow } from '$stores/windows';
	import { desktopApps, removeFromDesktop, initDesktop, loadInstalledApps, installedApps, type DesktopApp } from '$stores/desktop';
	import { loadAllAppTranslations, t } from '$lib/i18n';
	import Icon from '@iconify/svelte';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';

	let contextMenu = {
		visible: false,
		x: 0,
		y: 0,
		targetApp: null as DesktopApp | null
	};

	onMount(async () => {
		initDesktop();
		// Load installed apps from the backend registry
		await loadInstalledApps();
	});

	// Load translations when installed apps change
	$: if ($installedApps.length > 0) {
		loadAllAppTranslations($installedApps.map((app) => app.id));
	}

	function handleIconDblClick(app: DesktopApp) {
		openWindow({
			id: app.id,
			title: app.label,
			icon: app.icon,
			component: app.component,
			x: 180 + Math.random() * 150,
			y: 80 + Math.random() * 80,
			width: app.window?.width ?? 900,
			height: app.window?.height ?? 600
		});
	}

	function handleContextMenu(e: MouseEvent, app: DesktopApp) {
		e.preventDefault();
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

	$: contextMenuItems = contextMenu.targetApp
		? [
				{
					label: $t.common.open,
					icon: 'mdi:open-in-app',
					action: () => {
						if (contextMenu.targetApp) {
							handleIconDblClick(contextMenu.targetApp);
						}
					}
				},
				{
					label: $t.common.removeFromDesktop,
					icon: 'mdi:close-circle-outline',
					action: () => {
						if (contextMenu.targetApp) {
							removeFromDesktop(contextMenu.targetApp.id);
						}
					},
					danger: true
				}
			]
		: [];
</script>

<!-- Desktop Icons Grid -->
<div class="desktop-icons">
	{#each $desktopApps as app}
		<button
			class="desktop-icon"
			on:dblclick={() => handleIconDblClick(app)}
			on:contextmenu={(e) => handleContextMenu(e, app)}
		>
			<div class="icon-wrapper bg-gradient-to-br {app.gradient}">
				<Icon icon={app.icon} class="w-8 h-8 text-white" />
			</div>
			<span class="icon-label">{app.label}</span>
		</button>
	{/each}
</div>

<ContextMenu
	visible={contextMenu.visible}
	x={contextMenu.x}
	y={contextMenu.y}
	items={contextMenuItems}
	on:close={closeContextMenu}
/>

<style>
	.desktop-icons {
		position: absolute;
		top: 24px;
		left: 24px;
		display: grid;
		grid-template-columns: repeat(auto-fill, 80px);
		grid-auto-rows: 90px;
		gap: 8px;
		padding: 0;
		align-content: start;
	}

	.desktop-icon {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: flex-start;
		gap: 6px;
		padding: 8px 4px;
		border-radius: 10px;
		cursor: pointer;
		transition: all 0.15s ease;
		background: transparent;
		border: none;
		width: 80px;
		height: 88px;
	}

	.desktop-icon:hover {
		background: rgba(255, 255, 255, 0.15);
	}

	.desktop-icon:active {
		transform: scale(0.95);
	}

	.icon-wrapper {
		width: 52px;
		height: 52px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		transition: all 0.15s ease;
		flex-shrink: 0;
	}

	.desktop-icon:hover .icon-wrapper {
		transform: scale(1.05);
		box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
	}

	.icon-label {
		font-size: 11px;
		font-weight: 500;
		color: white;
		text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
		text-align: center;
		max-width: 76px;
		line-height: 1.2;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
	}
</style>
