<script lang="ts">
	import Icon from '@iconify/svelte';
	import {
		type WindowState,
		closeWindow,
		minimizeWindow,
		maximizeWindow,
		focusWindow,
		updateWindowPosition
	} from '$stores/windows';
	import { t } from '$lib/i18n';

	export let window: WindowState;

	let isDragging = false;
	let dragOffset = { x: 0, y: 0 };

	function handleMouseDown(e: MouseEvent) {
		if ((e.target as HTMLElement).closest('.window-controls')) return;

		isDragging = true;
		dragOffset = {
			x: e.clientX - window.x,
			y: e.clientY - window.y
		};

		focusWindow(window.id);
	}

	function handleMouseMove(e: MouseEvent) {
		if (!isDragging) return;

		const newX = Math.max(0, e.clientX - dragOffset.x);
		const newY = Math.max(40, e.clientY - dragOffset.y);

		updateWindowPosition(window.id, newX, newY);
	}

	function handleMouseUp() {
		isDragging = false;
	}

	function handleWindowClick() {
		focusWindow(window.id);
	}

	$: windowStyle = window.maximized
		? 'top: 40px; left: 0; right: 0; bottom: 64px; width: auto; height: auto;'
		: `top: ${window.y}px; left: ${window.x}px; width: ${window.width}px; height: ${window.height}px;`;
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="window"
	style="{windowStyle} z-index: {window.zIndex};"
	on:mousedown={handleWindowClick}
>
	<!-- Window Header -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="window-header" on:mousedown={handleMouseDown}>
		<div class="flex items-center gap-2">
			<Icon icon={window.icon} class="w-4 h-4 text-slate-500" />
			<span class="window-title">{window.title}</span>
		</div>

		<div class="window-controls">
			<button
				class="window-btn window-btn-help"
				title={$t.window.help}
			>
				<Icon icon="mdi:help" class="w-3 h-3" />
			</button>
			<button
				class="window-btn window-btn-minimize"
				on:click|stopPropagation={() => minimizeWindow(window.id)}
				title={$t.window.minimize}
			>
				<Icon icon="mdi:minus" class="w-3 h-3" />
			</button>
			<button
				class="window-btn window-btn-maximize"
				on:click|stopPropagation={() => maximizeWindow(window.id)}
				title={$t.window.maximize}
			>
				<Icon icon="mdi:square-outline" class="w-3 h-3" />
			</button>
			<button
				class="window-btn window-btn-close"
				on:click|stopPropagation={() => closeWindow(window.id)}
				title={$t.window.close}
			>
				<Icon icon="mdi:close" class="w-3 h-3" />
			</button>
		</div>
	</div>

	<!-- Window Content -->
	<div class="window-content">
		<slot />
	</div>
</div>

<style>
	.window {
		position: absolute;
		display: flex;
		flex-direction: column;
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 12px;
		overflow: hidden;
		box-shadow:
			0 25px 50px -12px rgba(0, 0, 0, 0.25),
			0 0 0 1px rgba(0, 0, 0, 0.05);
	}

	.window-header {
		height: 44px;
		background: rgba(248, 250, 252, 0.9);
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		cursor: move;
		user-select: none;
	}

	.window-title {
		font-size: 13px;
		font-weight: 500;
		color: #1e293b;
	}

	.window-controls {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.window-btn {
		width: 26px;
		height: 26px;
		border-radius: 6px;
		border: none;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.15s ease;
		color: #64748b;
		background: transparent;
	}

	.window-btn:hover {
		background: rgba(0, 0, 0, 0.08);
	}

	.window-btn-help:hover {
		color: #3b82f6;
	}

	.window-btn-minimize:hover {
		color: #f59e0b;
	}

	.window-btn-maximize:hover {
		color: #22c55e;
	}

	.window-btn-close:hover {
		background: #ef4444;
		color: white;
	}

	.window-content {
		flex: 1;
		overflow: auto;
		background: white;
	}
</style>
