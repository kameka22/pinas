<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { powerScreen } from '$stores/power';
	import { api } from '$stores/api';
	import { t } from '$lib/i18n';

	type Phase = 'waiting' | 'back' | 'off';

	let phase: Phase = 'waiting';
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	$: state = $powerScreen;
	$: menu = ($t as any).topBar?.userMenu || {};

	onMount(() => {
		if (state.action === 'restart') {
			// Poll until the server comes back
			startPolling();
		} else {
			// Shutdown: show "off" after a delay
			setTimeout(() => {
				phase = 'off';
			}, 8000);
		}
	});

	onDestroy(() => {
		stopPolling();
	});

	function startPolling() {
		if (pollInterval) return;
		// Wait a few seconds before starting to poll (server needs time to stop)
		setTimeout(() => {
			pollInterval = setInterval(async () => {
				try {
					await api.get('/system/info');
					// Server is back!
					phase = 'back';
					stopPolling();
				} catch {
					// Still down — keep polling
				}
			}, 3000);
		}, 5000);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	function handleReload() {
		window.location.reload();
	}

	function handleClose() {
		powerScreen.set({ active: false, action: 'restart' });
	}
</script>

<div class="power-screen">
	<div class="power-content">
		{#if state.action === 'restart' && phase === 'waiting'}
			<div class="phase-container fade-in">
				<div class="icon-container restarting-icon">
					<Icon icon="mdi:restart" class="w-16 h-16" />
				</div>
				<h1>{menu.restartingMessage || 'The system is restarting...'}</h1>
				<p class="subtitle">{menu.restartingSubtitle || 'Please wait, this may take a moment'}</p>
				<div class="dots">
					<span class="dot"></span>
					<span class="dot"></span>
					<span class="dot"></span>
				</div>
			</div>

		{:else if state.action === 'restart' && phase === 'back'}
			<div class="phase-container fade-in">
				<div class="icon-container success-icon">
					<Icon icon="mdi:check-circle" class="w-16 h-16" />
				</div>
				<h1>{menu.systemRestarted || 'System restarted successfully'}</h1>
				<div class="action-buttons">
					<button class="btn-action btn-primary-action" on:click={handleReload}>
						<Icon icon="mdi:refresh" class="w-5 h-5" />
						{menu.reloadDesktop || 'Reload desktop'}
					</button>
				</div>
			</div>

		{:else if state.action === 'shutdown' && phase === 'waiting'}
			<div class="phase-container fade-in">
				<div class="icon-container shutdown-icon">
					<Icon icon="mdi:power" class="w-16 h-16" />
				</div>
				<h1>{menu.shuttingDownMessage || 'The system is shutting down...'}</h1>
				<p class="subtitle">{menu.shuttingDownSubtitle || 'You can safely unplug the device'}</p>
				<div class="dots">
					<span class="dot"></span>
					<span class="dot"></span>
					<span class="dot"></span>
				</div>
			</div>

		{:else if state.action === 'shutdown' && phase === 'off'}
			<div class="phase-container fade-in">
				<div class="icon-container off-icon">
					<Icon icon="mdi:power-off" class="w-16 h-16" />
				</div>
				<h1>{menu.systemShutDown || 'System is shutting down'}</h1>
				<p class="subtitle">{menu.shuttingDownSubtitle || 'You can safely unplug the device'}</p>
				<div class="action-buttons">
					<button class="btn-action" on:click={handleClose}>
						{($t as any).common?.close || 'Close'}
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.power-screen {
		position: fixed;
		inset: 0;
		z-index: 10000;
		background: linear-gradient(135deg, #0f172a 0%, #1e3a5f 50%, #0f172a 100%);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.power-content {
		text-align: center;
		max-width: 500px;
		padding: 40px;
	}

	.phase-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 24px;
	}

	.fade-in {
		animation: fadeIn 0.6s ease-out;
	}

	@keyframes fadeIn {
		from { opacity: 0; transform: translateY(10px); }
		to { opacity: 1; transform: translateY(0); }
	}

	.icon-container {
		width: 96px;
		height: 96px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 8px;
	}

	.restarting-icon {
		background: rgba(59, 130, 246, 0.15);
		color: #60a5fa;
		animation: spin-slow 2s linear infinite;
	}

	.shutdown-icon {
		background: rgba(239, 68, 68, 0.12);
		color: #f87171;
		animation: pulse-fade 2s ease-in-out infinite;
	}

	.success-icon {
		background: rgba(34, 197, 94, 0.15);
		color: #4ade80;
		animation: pop 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
	}

	.off-icon {
		background: rgba(100, 116, 139, 0.15);
		color: #94a3b8;
	}

	@keyframes spin-slow {
		to { transform: rotate(360deg); }
	}

	@keyframes pulse-fade {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}

	@keyframes pop {
		0% { transform: scale(0.5); opacity: 0; }
		100% { transform: scale(1); opacity: 1; }
	}

	h1 {
		font-size: 24px;
		font-weight: 600;
		color: white;
		margin: 0;
	}

	.subtitle {
		font-size: 15px;
		color: rgba(255, 255, 255, 0.5);
		margin: 0;
	}

	/* Loading dots */
	.dots {
		display: flex;
		gap: 8px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: rgba(255, 255, 255, 0.3);
		animation: dot-bounce 1.4s ease-in-out infinite;
	}

	.dot:nth-child(2) { animation-delay: 0.2s; }
	.dot:nth-child(3) { animation-delay: 0.4s; }

	@keyframes dot-bounce {
		0%, 80%, 100% { transform: scale(1); opacity: 0.3; }
		40% { transform: scale(1.3); opacity: 1; background: #60a5fa; }
	}

	.action-buttons {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		margin-top: 8px;
	}

	.btn-action {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 32px;
		border-radius: 10px;
		font-size: 15px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		background: rgba(255, 255, 255, 0.1);
		color: white;
		border: 1px solid rgba(255, 255, 255, 0.15);
	}

	.btn-action:hover {
		background: rgba(255, 255, 255, 0.15);
	}

	.btn-primary-action {
		background: #3b82f6;
		border-color: #3b82f6;
	}

	.btn-primary-action:hover {
		background: #2563eb;
	}
</style>
