<script lang="ts">
	import Icon from '@iconify/svelte';
	import { fly } from 'svelte/transition';
	import { toasts, type ToastKind } from '$stores/toasts';

	const icons: Record<ToastKind, string> = {
		success: 'mdi:check-circle',
		error: 'mdi:alert-circle',
		info: 'mdi:information',
		warning: 'mdi:alert'
	};
</script>

<div class="toast-host" aria-live="polite">
	{#each $toasts as toast (toast.id)}
		<div class="toast toast-{toast.kind}" role={toast.kind === 'error' ? 'alert' : 'status'} transition:fly={{ y: 16, duration: 180 }}>
			<Icon icon={icons[toast.kind]} class="w-5 h-5 shrink-0" />
			<span class="toast-message">{toast.message}</span>
			<button class="toast-close" on:click={() => toasts.dismiss(toast.id)} aria-label="×">
				<Icon icon="mdi:close" class="w-4 h-4" />
			</button>
		</div>
	{/each}
</div>

<style>
	.toast-host {
		position: fixed;
		right: 1rem;
		bottom: 5rem;
		z-index: 10000;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-width: 24rem;
		pointer-events: none;
	}
	.toast {
		pointer-events: auto;
		display: flex;
		align-items: flex-start;
		gap: 0.5rem;
		padding: 0.75rem 0.875rem;
		border-radius: 0.75rem;
		background: rgb(15 23 42 / 0.92);
		color: #f8fafc;
		box-shadow: 0 10px 30px rgb(0 0 0 / 0.25);
		backdrop-filter: blur(8px);
		font-size: 0.875rem;
		line-height: 1.25rem;
	}
	.toast-success { border-left: 3px solid #22c55e; }
	.toast-error { border-left: 3px solid #ef4444; }
	.toast-info { border-left: 3px solid #3b82f6; }
	.toast-warning { border-left: 3px solid #f59e0b; }
	.toast-message { flex: 1; word-break: break-word; }
	.toast-close {
		opacity: 0.6;
		padding: 0.125rem;
		border-radius: 0.25rem;
	}
	.toast-close:hover { opacity: 1; background: rgb(255 255 255 / 0.1); }
</style>
