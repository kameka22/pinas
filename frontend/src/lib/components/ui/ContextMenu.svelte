<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, createEventDispatcher } from 'svelte';
	import { browser } from '$app/environment';

	export let x: number;
	export let y: number;
	export let visible = false;
	export let items: {
		label: string;
		icon?: string;
		action: () => void;
		danger?: boolean;
		disabled?: boolean;
	}[] = [];

	const dispatch = createEventDispatcher();

	let menuElement: HTMLDivElement;

	function handleClickOutside(e: MouseEvent) {
		if (menuElement && !menuElement.contains(e.target as Node)) {
			dispatch('close');
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			dispatch('close');
		}
	}

	function handleItemClick(item: (typeof items)[0]) {
		if (item.disabled) return;
		item.action();
		dispatch('close');
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('click', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});

	// Ajuster la position pour ne pas déborder de l'écran
	$: adjustedX = browser ? Math.min(x, window.innerWidth - 200) : x;
	$: adjustedY = browser ? Math.min(y, window.innerHeight - items.length * 40 - 20) : y;
</script>

{#if visible}
	<div
		class="context-menu"
		bind:this={menuElement}
		style="left: {adjustedX}px; top: {adjustedY}px;"
	>
		{#each items as item}
			<button
				class="menu-item"
				class:danger={item.danger}
				class:disabled={item.disabled}
				on:click={() => handleItemClick(item)}
			>
				{#if item.icon}
					<Icon icon={item.icon} class="w-4 h-4" />
				{/if}
				<span>{item.label}</span>
			</button>
		{/each}
	</div>
{/if}

<style>
	.context-menu {
		position: fixed;
		z-index: 1000;
		min-width: 180px;
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 10px;
		box-shadow:
			0 10px 40px rgba(0, 0, 0, 0.15),
			0 0 0 1px rgba(0, 0, 0, 0.05);
		padding: 6px;
		overflow: hidden;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		border: none;
		background: transparent;
		border-radius: 6px;
		font-size: 13px;
		color: #1e293b;
		cursor: pointer;
		transition: all 0.1s ease;
		text-align: left;
	}

	.menu-item:hover:not(.disabled) {
		background: rgba(59, 130, 246, 0.1);
		color: #3b82f6;
	}

	.menu-item.danger {
		color: #ef4444;
	}

	.menu-item.danger:hover:not(.disabled) {
		background: rgba(239, 68, 68, 0.1);
		color: #dc2626;
	}

	.menu-item.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
