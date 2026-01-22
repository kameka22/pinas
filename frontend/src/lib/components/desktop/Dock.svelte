<script lang="ts">
	import Icon from '@iconify/svelte';
	import { windows, openWindow, focusWindow, restoreWindow } from '$stores/windows';
	import { t } from '$lib/i18n';

	// Dock items configuration with translation keys
	const dockItemsConfig = [
		{
			id: 'file-manager',
			icon: 'mdi:folder',
			labelKey: 'files',
			component: 'FileManager',
			gradient: 'from-amber-400 to-amber-500'
		},
		{
			id: 'app-center',
			icon: 'mdi:store',
			labelKey: 'appCenter',
			component: 'AppCenter',
			gradient: 'from-purple-500 to-pink-500'
		},
		{
			id: 'control-panel',
			icon: 'mdi:tune-variant',
			labelKey: 'controlPanel',
			component: 'ControlPanel',
			gradient: 'from-slate-500 to-slate-600'
		}
	];

	// Reactive dock items with translated labels
	$: dockItems = dockItemsConfig.map(item => ({
		...item,
		label: $t.apps[item.labelKey as keyof typeof $t.apps] || item.labelKey
	}));

	function handleClick(item: (typeof dockItems)[0]) {
		const existingWindow = $windows.find((w) => w.id === item.id);
		if (existingWindow) {
			if (existingWindow.minimized) {
				restoreWindow(item.id);
			}
			focusWindow(item.id);
		} else {
			openWindow({
				id: item.id,
				title: item.label,
				icon: item.icon,
				component: item.component,
				x: 150 + Math.random() * 100,
				y: 80 + Math.random() * 50,
				width: 900,
				height: 600
			});
		}
	}

	function isOpen(id: string): boolean {
		return $windows.some((w) => w.id === id);
	}

	function handleWindowClick(win: { id: string; minimized: boolean }) {
		if (win.minimized) {
			restoreWindow(win.id);
		}
		focusWindow(win.id);
	}
</script>

<nav class="dock">
	<div class="dock-container">
		{#each dockItems as item}
			<button
				class="dock-item"
				on:click={() => handleClick(item)}
				title={item.label}
			>
				<div class="dock-icon bg-gradient-to-br {item.gradient}">
					<Icon icon={item.icon} class="w-6 h-6 text-white" />
				</div>
				{#if isOpen(item.id)}
					<span class="dock-indicator"></span>
				{/if}
			</button>
		{/each}

		<!-- Open windows (excluding pinned apps) -->
		{#if $windows.filter(w => !dockItems.some(d => d.id === w.id)).length > 0}
			<div class="dock-separator"></div>

			{#each $windows.filter(w => !dockItems.some(d => d.id === w.id)) as win}
				<button
					class="dock-item"
					class:minimized={win.minimized}
					on:click={() => handleWindowClick(win)}
					title={win.title}
				>
					<div class="dock-icon bg-gradient-to-br from-slate-600 to-slate-700">
						<Icon icon={win.icon} class="w-6 h-6 text-white" />
					</div>
					{#if !win.minimized}
						<span class="dock-indicator"></span>
					{/if}
				</button>
			{/each}
		{/if}
	</div>
</nav>

<style>
	.dock {
		position: fixed;
		bottom: 8px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 90;
	}

	.dock-container {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		background: rgba(255, 255, 255, 0.2);
		backdrop-filter: blur(20px);
		border-radius: 18px;
		border: 1px solid rgba(255, 255, 255, 0.2);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
	}

	.dock-item {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 4px;
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
