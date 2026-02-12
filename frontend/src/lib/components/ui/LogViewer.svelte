<script lang="ts">
	import Icon from '@iconify/svelte';
	import { createEventDispatcher, afterUpdate } from 'svelte';

	export let visible = false;
	export let title = 'Logs';
	export let logs: string[] = [];
	export let loading = false;

	const dispatch = createEventDispatcher();

	let logContainer: HTMLDivElement;
	let tail = 100;
	const tailOptions = [100, 500, 1000];

	afterUpdate(() => {
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	});

	function handleClose() {
		dispatch('close');
	}

	function handleRefresh() {
		dispatch('refresh', { tail });
	}

	function handleTailChange(value: number) {
		tail = value;
		dispatch('refresh', { tail });
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') handleClose();
	}
</script>

{#if visible}
	<div class="modal-overlay" on:click={handleClose} on:keydown={handleKeydown} role="presentation">
		<div class="log-modal" on:click|stopPropagation role="dialog" aria-modal="true">
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:text-box-outline" class="w-5 h-5" />
					{title}
				</h2>
				<div class="header-actions">
					<div class="tail-selector">
						{#each tailOptions as opt}
							<button
								class="tail-btn"
								class:active={tail === opt}
								on:click={() => handleTailChange(opt)}
							>
								{opt}
							</button>
						{/each}
					</div>
					<button class="refresh-btn" on:click={handleRefresh} disabled={loading}>
						<span class:spinning={loading}>
							<Icon icon="mdi:refresh" class="w-4 h-4" />
						</span>
					</button>
					<button class="close-btn" on:click={handleClose}>
						<Icon icon="mdi:close" class="w-5 h-5" />
					</button>
				</div>
			</div>

			<div class="log-content" bind:this={logContainer}>
				{#if loading && logs.length === 0}
					<div class="log-loading">
						<Icon icon="mdi:loading" class="w-6 h-6 spinning" />
					</div>
				{:else if logs.length === 0}
					<div class="log-empty">No logs available</div>
				{:else}
					{#each logs as line}
						<div class="log-line">{line}</div>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
		animation: fade-in 0.15s ease-out;
	}

	@keyframes fade-in {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.log-modal {
		background: #1e1e2e;
		border-radius: 16px;
		width: 90%;
		max-width: 800px;
		height: 70vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
		animation: modal-slide-in 0.2s ease-out;
	}

	@keyframes modal-slide-in {
		from { opacity: 0; transform: translateY(-20px); }
		to { opacity: 1; transform: translateY(0); }
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #313244;
	}

	.modal-header h2 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 16px;
		font-weight: 600;
		color: #cdd6f4;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.tail-selector {
		display: flex;
		background: #313244;
		border-radius: 6px;
		overflow: hidden;
	}

	.tail-btn {
		padding: 4px 10px;
		border: none;
		background: transparent;
		color: #a6adc8;
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s;
	}

	.tail-btn:hover {
		color: #cdd6f4;
	}

	.tail-btn.active {
		background: #585b70;
		color: #cdd6f4;
	}

	.refresh-btn,
	.close-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		border: none;
		background: transparent;
		color: #a6adc8;
		cursor: pointer;
		transition: all 0.15s;
	}

	.refresh-btn:hover,
	.close-btn:hover {
		background: #313244;
		color: #cdd6f4;
	}

	.refresh-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.log-content {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
		font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
		font-size: 12px;
		line-height: 1.6;
	}

	.log-content::-webkit-scrollbar {
		width: 8px;
	}

	.log-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.log-content::-webkit-scrollbar-thumb {
		background: #585b70;
		border-radius: 4px;
	}

	.log-line {
		color: #a6e3a1;
		padding: 1px 0;
		white-space: pre-wrap;
		word-break: break-all;
	}

	.log-line:hover {
		background: rgba(255, 255, 255, 0.05);
	}

	.log-loading,
	.log-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: #6c7086;
		font-size: 14px;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
