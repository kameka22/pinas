<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	// Config passed from manifest
	export let config: {
		port: number;
		path?: string;
		title?: string;
	} = { port: 8080, path: '/' };

	let loading = true;
	let error: string | null = null;
	let iframeEl: HTMLIFrameElement;

	$: appUrl = `http://${window.location.hostname}:${config.port}${config.path || '/'}`;

	function handleLoad() {
		loading = false;
	}

	function handleError() {
		loading = false;
		error = $t.iframeApp?.connectionError || 'Unable to connect to the application';
	}

	function refresh() {
		loading = true;
		error = null;
		if (iframeEl) {
			iframeEl.src = appUrl;
		}
	}

	function openExternal() {
		window.open(appUrl, '_blank');
	}

	onMount(() => {
		// Timeout for connection error
		const timeout = setTimeout(() => {
			if (loading) {
				loading = false;
				error = $t.iframeApp?.timeout || 'Connection timeout - application may not be running';
			}
		}, 15000);

		return () => clearTimeout(timeout);
	});
</script>

<div class="iframe-app">
	<!-- Toolbar -->
	<div class="toolbar">
		<div class="toolbar-left">
			<span class="url-display">{appUrl}</span>
		</div>
		<div class="toolbar-right">
			<button class="toolbar-btn" on:click={refresh} title={$t.common?.refresh || 'Refresh'}>
				<Icon icon="mdi:refresh" class="w-4 h-4" />
			</button>
			<button class="toolbar-btn" on:click={openExternal} title={$t.iframeApp?.openExternal || 'Open in new tab'}>
				<Icon icon="mdi:open-in-new" class="w-4 h-4" />
			</button>
		</div>
	</div>

	<!-- Content -->
	<div class="iframe-container">
		{#if loading}
			<div class="loading-overlay">
				<Icon icon="mdi:loading" class="w-12 h-12 animate-spin text-blue-500" />
				<p>{$t.common?.loading || 'Loading...'}</p>
			</div>
		{/if}

		{#if error}
			<div class="error-state">
				<Icon icon="mdi:alert-circle" class="w-16 h-16 text-red-400" />
				<h3>{$t.iframeApp?.errorTitle || 'Connection Error'}</h3>
				<p>{error}</p>
				<div class="error-actions">
					<button class="btn-primary" on:click={refresh}>
						<Icon icon="mdi:refresh" class="w-4 h-4" />
						{$t.common?.retry || 'Retry'}
					</button>
					<button class="btn-secondary" on:click={openExternal}>
						<Icon icon="mdi:open-in-new" class="w-4 h-4" />
						{$t.iframeApp?.openExternal || 'Open in new tab'}
					</button>
				</div>
			</div>
		{:else}
			<iframe
				bind:this={iframeEl}
				src={appUrl}
				title={config.title || 'Application'}
				on:load={handleLoad}
				on:error={handleError}
				class:hidden={loading}
			/>
		{/if}
	</div>
</div>

<style>
	.iframe-app {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #f8fafc;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.url-display {
		font-size: 13px;
		color: #64748b;
		font-family: monospace;
		background: #f1f5f9;
		padding: 4px 10px;
		border-radius: 4px;
	}

	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.toolbar-btn {
		padding: 6px;
		border: none;
		background: transparent;
		border-radius: 6px;
		cursor: pointer;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.toolbar-btn:hover {
		background: #f1f5f9;
		color: #334155;
	}

	.iframe-container {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	iframe {
		width: 100%;
		height: 100%;
		border: none;
	}

	iframe.hidden {
		visibility: hidden;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		background: white;
	}

	.loading-overlay p {
		color: #64748b;
		font-size: 14px;
	}

	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 16px;
		padding: 40px;
		text-align: center;
	}

	.error-state h3 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.error-state p {
		font-size: 14px;
		color: #64748b;
		max-width: 400px;
	}

	.error-actions {
		display: flex;
		gap: 12px;
		margin-top: 8px;
	}

	.btn-primary,
	.btn-secondary {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		border: none;
		transition: all 0.15s ease;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-secondary {
		background: #f1f5f9;
		color: #334155;
	}

	.btn-secondary:hover {
		background: #e2e8f0;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	:global(.animate-spin) {
		animation: spin 1s linear infinite;
	}
</style>
