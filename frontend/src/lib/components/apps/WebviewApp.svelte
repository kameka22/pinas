<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	// Config passed from manifest
	export let config: {
		port: number;
		path?: string;
		name?: string;
		icon?: string;
		gradient?: string;
		checkHealth?: boolean;
	} = { port: 8080, path: '/' };

	// App metadata (can be overridden by window props)
	export let name: string = config.name || 'Application';
	export let icon: string = config.icon || 'mdi:application';
	export let gradient: string = config.gradient || 'from-slate-500 to-slate-600';

	let status: 'checking' | 'online' | 'offline' = 'checking';
	let healthCheckInterval: ReturnType<typeof setInterval>;

	$: appUrl = `http://${window.location.hostname}:${config.port}${config.path || '/'}`;

	async function checkHealth() {
		if (!config.checkHealth) {
			status = 'online';
			return;
		}

		try {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 5000);

			const response = await fetch(appUrl, {
				method: 'HEAD',
				mode: 'no-cors',
				signal: controller.signal
			});

			clearTimeout(timeout);
			status = 'online';
		} catch (e) {
			status = 'offline';
		}
	}

	function openApp() {
		window.open(appUrl, '_blank', 'noopener,noreferrer');
	}

	function copyUrl() {
		navigator.clipboard.writeText(appUrl);
	}

	onMount(() => {
		checkHealth();
		if (config.checkHealth) {
			healthCheckInterval = setInterval(checkHealth, 30000);
		}
	});

	onDestroy(() => {
		if (healthCheckInterval) clearInterval(healthCheckInterval);
	});
</script>

<div class="webview-app">
	<div class="content">
		<!-- App Icon -->
		<div class="app-icon bg-gradient-to-br {gradient}">
			<Icon icon={icon} class="w-20 h-20 text-white" />
		</div>

		<!-- App Info -->
		<h1 class="app-name">{name}</h1>

		<p class="app-description">
			{$t.webviewApp?.description || 'This application opens in a new browser tab'}
		</p>

		<!-- Status -->
		<div class="status-row">
			{#if status === 'checking'}
				<span class="status-badge checking">
					<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
					{$t.webviewApp?.checking || 'Checking status...'}
				</span>
			{:else if status === 'online'}
				<span class="status-badge online">
					<Icon icon="mdi:check-circle" class="w-4 h-4" />
					{$t.webviewApp?.online || 'Online'}
				</span>
			{:else}
				<span class="status-badge offline">
					<Icon icon="mdi:alert-circle" class="w-4 h-4" />
					{$t.webviewApp?.offline || 'Offline'}
				</span>
			{/if}
		</div>

		<!-- URL Display -->
		<div class="url-container">
			<code class="url-display">{appUrl}</code>
			<button class="copy-btn" on:click={copyUrl} title={$t.common?.copy || 'Copy'}>
				<Icon icon="mdi:content-copy" class="w-4 h-4" />
			</button>
		</div>

		<!-- Actions -->
		<div class="actions">
			<button class="btn-primary" on:click={openApp}>
				<Icon icon="mdi:open-in-new" class="w-5 h-5" />
				{$t.webviewApp?.openApp || 'Open Application'}
			</button>
		</div>

		<!-- Tips -->
		<div class="tips">
			<Icon icon="mdi:information-outline" class="w-4 h-4" />
			<span>{$t.webviewApp?.tip || 'You can also access this application directly at the URL above'}</span>
		</div>
	</div>
</div>

<style>
	.webview-app {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
	}

	.content {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		max-width: 400px;
		padding: 40px;
	}

	.app-icon {
		width: 120px;
		height: 120px;
		border-radius: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
		margin-bottom: 24px;
	}

	.app-name {
		font-size: 28px;
		font-weight: 700;
		color: #1e293b;
		margin-bottom: 12px;
	}

	.app-description {
		font-size: 15px;
		color: #64748b;
		margin-bottom: 20px;
		line-height: 1.5;
	}

	.status-row {
		margin-bottom: 20px;
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border-radius: 20px;
		font-size: 13px;
		font-weight: 500;
	}

	.status-badge.checking {
		background: #f1f5f9;
		color: #64748b;
	}

	.status-badge.online {
		background: #dcfce7;
		color: #16a34a;
	}

	.status-badge.offline {
		background: #fee2e2;
		color: #dc2626;
	}

	.url-container {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 24px;
	}

	.url-display {
		font-size: 13px;
		color: #64748b;
		background: white;
		padding: 10px 16px;
		border-radius: 8px;
		border: 1px solid #e2e8f0;
		font-family: monospace;
	}

	.copy-btn {
		padding: 10px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		cursor: pointer;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.copy-btn:hover {
		background: #f8fafc;
		color: #334155;
	}

	.actions {
		margin-bottom: 32px;
	}

	.btn-primary {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		padding: 14px 28px;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 12px;
		font-size: 16px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
		box-shadow: 0 4px 14px rgba(59, 130, 246, 0.4);
	}

	.btn-primary:hover {
		background: #2563eb;
		transform: translateY(-1px);
		box-shadow: 0 6px 20px rgba(59, 130, 246, 0.5);
	}

	.tips {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		color: #94a3b8;
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
