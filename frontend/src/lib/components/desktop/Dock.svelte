<script lang="ts">
	import Icon from '@iconify/svelte';
	import { windows, openWindow, focusWindow, restoreWindow } from '$stores/windows';
	import { allApps, dockPinnedIds, addToDock, removeFromDock } from '$stores/desktop';
	import { t } from '$lib/i18n';
	import { gradientStyle } from '$lib/utils/gradient';
	import ContextMenu from '$lib/components/ui/ContextMenu.svelte';

	// Resolve pinned dock apps from IDs
	$: pinnedDockApps = $dockPinnedIds
		.map((id) => $allApps.find((a) => a.id === id))
		.filter(Boolean) as (typeof $allApps)[number][];

	// Set of open window IDs (reactive)
	$: openWindowIds = new Set($windows.map((w) => w.id));

	// Open windows that are NOT pinned in dock
	$: unpinnedOpenWindows = $windows.filter((w) => !$dockPinnedIds.includes(w.id));

	// Context menu state
	let contextMenu = {
		visible: false,
		x: 0,
		y: 0,
		targetId: null as string | null,
		isPinned: false
	};

	function getAppLabel(app: { labelKey?: string; label: string }): string {
		if (app.labelKey && $t.apps[app.labelKey as keyof typeof $t.apps]) {
			return $t.apps[app.labelKey as keyof typeof $t.apps];
		}
		return app.label;
	}

	function handleClick(app: (typeof pinnedDockApps)[0]) {
		const existingWindow = $windows.find((w) => w.id === app.id);
		if (existingWindow) {
			if (existingWindow.minimized) {
				restoreWindow(app.id);
			}
			focusWindow(app.id);
		} else {
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
		}
	}


	function handleWindowClick(win: { id: string; minimized: boolean }) {
		if (win.minimized) {
			restoreWindow(win.id);
		}
		focusWindow(win.id);
	}

	function handleContextMenu(e: MouseEvent, id: string, isPinned: boolean) {
		e.preventDefault();
		e.stopPropagation();
		contextMenu = { visible: true, x: e.clientX, y: e.clientY, targetId: id, isPinned };
	}

	function closeContextMenu() {
		contextMenu = { ...contextMenu, visible: false, targetId: null };
	}

	$: contextMenuItems = (() => {
		if (!contextMenu.targetId) return [];
		const id = contextMenu.targetId;
		const isPinned = contextMenu.isPinned;

		if (isPinned) {
			const app = pinnedDockApps.find((a) => a.id === id);
			return [
				{
					label: $t.common.open,
					icon: 'mdi:open-in-app',
					action: () => { if (app) handleClick(app); }
				},
				{
					label: $t.common.removeFromDock,
					icon: 'mdi:dock-bottom',
					action: () => removeFromDock(id),
					danger: true
				}
			];
		} else {
			return [
				{
					label: $t.common.pinToDock,
					icon: 'mdi:pin',
					action: () => addToDock(id)
				}
			];
		}
	})();
</script>

<nav class="dock-container">
		{#each pinnedDockApps as app}
			<button
				class="dock-item"
				on:click={() => handleClick(app)}
				on:contextmenu={(e) => handleContextMenu(e, app.id, true)}
				title={getAppLabel(app)}
			>
				<div class="dock-icon" style={gradientStyle(app.gradient)}>
					<Icon icon={app.icon} class="w-6 h-6 text-white" />
				</div>
				{#if openWindowIds.has(app.id)}
					<span class="dock-indicator"></span>
				{/if}
			</button>
		{/each}

		<!-- Open windows (excluding pinned apps) -->
		{#if unpinnedOpenWindows.length > 0}
			<div class="dock-separator"></div>

			{#each unpinnedOpenWindows as win}
				<button
					class="dock-item"
					class:minimized={win.minimized}
					on:click={() => handleWindowClick(win)}
					on:contextmenu={(e) => handleContextMenu(e, win.id, false)}
					title={win.title}
				>
					<div class="dock-icon" style={gradientStyle(win.gradient || 'from-slate-600 to-slate-700')}>
						<Icon icon={win.icon} class="w-6 h-6 text-white" />
					</div>
					{#if !win.minimized}
						<span class="dock-indicator"></span>
					{/if}
				</button>
			{/each}
		{/if}
</nav>

<ContextMenu
	visible={contextMenu.visible}
	x={contextMenu.x}
	y={contextMenu.y}
	items={contextMenuItems}
	on:close={closeContextMenu}
/>

<style>
	.dock-container {
		position: fixed;
		bottom: 8px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 90;
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		background: rgba(255, 255, 255, 0.2);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		border-radius: 18px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
	}

	.dock-item {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 4px;
		border: none;
		background: transparent;
		border-radius: 12px;
		transition: all 0.2s ease;
		cursor: pointer;
	}

	.dock-item:hover {
		transform: translateY(-8px) scale(1.1);
	}

	.dock-item:hover .dock-icon {
		box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3);
	}

	.dock-icon {
		width: 48px;
		height: 48px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		transition: all 0.2s ease;
	}

	.dock-indicator {
		position: absolute;
		bottom: -2px;
		width: 4px;
		height: 4px;
		background: white;
		border-radius: 50%;
		box-shadow: 0 0 4px rgba(255, 255, 255, 0.5);
	}

	.dock-separator {
		width: 1px;
		height: 40px;
		background: rgba(255, 255, 255, 0.3);
		margin: 0 6px;
	}

	.dock-item.minimized .dock-icon {
		opacity: 0.7;
	}
</style>
