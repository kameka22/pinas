<script lang="ts">
	import Icon from '@iconify/svelte';
	import { get } from 'svelte/store';
	import { onMount, onDestroy } from 'svelte';
	import { updateScreen } from '$stores/update';
	import { taskProgress } from '$stores/websocket';
	import { api } from '$stores/api';
	import { t } from '$lib/i18n';

	type Phase = 'starting' | 'progress' | 'completed' | 'error';

	let phase: Phase = 'starting';
	let progressPercent = 0;
	let currentStep = '';
	let errorMessage = '';
	let startingTimer: ReturnType<typeof setTimeout> | null = null;
	let devSimInterval: ReturnType<typeof setInterval> | null = null;
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	// Read store values reactively for template
	$: state = $updateScreen;
	$: tUpdate = ($t as any).systemUpdate?.screen || {};

	const unsubProgress = taskProgress.subscribe((tasks) => {
		const s = get(updateScreen);
		if (!s.taskId || s.devTest) return;
		const progress = tasks[s.taskId];
		if (!progress) return;

		if (phase === 'starting') return; // Wait for starting phase to finish

		progressPercent = progress.progress_percent;
		currentStep = progress.current_step || '';

		if (progress.status === 'completed') {
			phase = 'completed';
			stopPolling();
		} else if (progress.status === 'failed') {
			phase = 'error';
			errorMessage = progress.error_message || 'Unknown error';
			stopPolling();
		} else {
			phase = 'progress';
		}
	});

	onMount(() => {
		const s = get(updateScreen);
		// Phase 1: Starting (5 seconds)
		startingTimer = setTimeout(() => {
			if (s.devTest) {
				startDevSimulation();
			} else {
				phase = 'progress';
				// Start polling for server comeback (the WS broadcast is lost during restart)
				startPolling();
			}
		}, 5000);
	});

	onDestroy(() => {
		unsubProgress();
		if (startingTimer) clearTimeout(startingTimer);
		if (devSimInterval) clearInterval(devSimInterval);
		stopPolling();
	});

	function startPolling() {
		if (pollInterval) return;
		// Poll every 3 seconds to detect when the server is back after restart
		pollInterval = setInterval(async () => {
			try {
				const resp = await api.get<{ just_updated: boolean; version?: string }>('/system/update/just-updated');
				if (resp.just_updated) {
					// Server is back and confirms the update was applied
					progressPercent = 100;
					currentStep = '';
					phase = 'completed';
					if (resp.version) {
						updateScreen.update(s => ({ ...s, version: resp.version! }));
					}
					stopPolling();
				}
			} catch {
				// Server still down — keep polling silently
			}
		}, 3000);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	function startDevSimulation() {
		phase = 'progress';
		progressPercent = 0;

		const steps = [
			{ at: 0, text: 'Downloading update archive...' },
			{ at: 25, text: 'Extracting files...' },
			{ at: 50, text: 'Applying update...' },
			{ at: 75, text: 'Restarting services...' },
			{ at: 95, text: 'Finalizing...' }
		];

		let stepIndex = 0;
		currentStep = steps[0].text;

		devSimInterval = setInterval(() => {
			progressPercent += 1.25; // 0 to 100 in ~8s (80 ticks at 100ms)
			if (progressPercent >= 100) {
				progressPercent = 100;
				if (devSimInterval) clearInterval(devSimInterval);
				phase = 'completed';
				return;
			}
			// Update step text
			if (stepIndex < steps.length - 1 && progressPercent >= steps[stepIndex + 1].at) {
				stepIndex++;
				currentStep = steps[stepIndex].text;
			}
		}, 100);
	}

	function handleReboot() {
		api.post('/system/reboot').catch(() => {});
	}

	function handleReload() {
		window.location.reload();
	}

	function handleClose() {
		updateScreen.set({
			active: false,
			taskId: null,
			rebootRequired: false,
			version: '',
			changelog: null,
			devTest: false
		});
	}
</script>

<div class="update-screen">
	<div class="update-content">
		{#if phase === 'starting'}
			<div class="phase-container fade-in">
				<div class="icon-container starting-icon">
					<Icon icon="mdi:update" class="w-16 h-16" />
				</div>
				<h1>{tUpdate.starting || 'The update is about to start...'}</h1>
				<p class="subtitle">{tUpdate.doNotTurnOff || 'Please do not turn off your device'}</p>
			</div>

		{:else if phase === 'progress'}
			<div class="phase-container fade-in">
				<div class="icon-container progress-icon">
					<Icon icon="mdi:download" class="w-16 h-16" />
				</div>
				<h1>{tUpdate.inProgress || 'Update in progress'}</h1>
				<div class="progress-wrapper">
					<div class="progress-bar">
						<div class="progress-fill" style="width: {progressPercent}%"></div>
					</div>
					<span class="progress-text">{Math.round(progressPercent)}%</span>
				</div>
				{#if currentStep}
					<p class="step-text">{currentStep}</p>
				{/if}
			</div>

		{:else if phase === 'completed'}
			<div class="phase-container fade-in">
				<div class="icon-container success-icon">
					<Icon icon="mdi:check-circle" class="w-16 h-16" />
				</div>
				<h1>{tUpdate.completed || 'Update completed successfully!'}</h1>
				<p class="version-text">
					{(tUpdate.installed || 'PiNAS {version} installed').replace('{version}', state.version ? `v${state.version}` : '')}
				</p>
				<div class="action-buttons">
					{#if state.devTest}
						<button class="btn-action" on:click={handleClose}>
							{tUpdate.close || 'Close'}
						</button>
					{:else if state.rebootRequired}
						<button class="btn-action btn-primary-action" on:click={handleReboot}>
							<Icon icon="mdi:restart" class="w-5 h-5" />
							{tUpdate.reboot || 'Reboot'}
						</button>
					{:else}
						<button class="btn-action btn-primary-action" on:click={handleReload}>
							<Icon icon="mdi:refresh" class="w-5 h-5" />
							{tUpdate.reloadDesktop || 'Reload desktop'}
						</button>
					{/if}
				</div>
			</div>

		{:else if phase === 'error'}
			<div class="phase-container fade-in">
				<div class="icon-container error-icon">
					<Icon icon="mdi:close-circle" class="w-16 h-16" />
				</div>
				<h1>{tUpdate.failed || 'Update failed'}</h1>
				{#if errorMessage}
					<p class="error-text">{errorMessage}</p>
				{/if}
				<div class="action-buttons">
					<button class="btn-action" on:click={handleClose}>
						{tUpdate.close || 'Close'}
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.update-screen {
		position: fixed;
		inset: 0;
		z-index: 10000;
		background: linear-gradient(135deg, #0f172a 0%, #1e3a5f 50%, #0f172a 100%);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.update-content {
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

	.starting-icon {
		background: rgba(59, 130, 246, 0.15);
		color: #60a5fa;
		animation: pulse-rotate 3s ease-in-out infinite;
	}

	.progress-icon {
		background: rgba(59, 130, 246, 0.15);
		color: #60a5fa;
		animation: bounce-slow 2s ease-in-out infinite;
	}

	.success-icon {
		background: rgba(34, 197, 94, 0.15);
		color: #4ade80;
		animation: pop 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
	}

	.error-icon {
		background: rgba(239, 68, 68, 0.15);
		color: #f87171;
	}

	@keyframes pulse-rotate {
		0%, 100% { transform: rotate(0deg) scale(1); }
		50% { transform: rotate(180deg) scale(1.05); }
	}

	@keyframes bounce-slow {
		0%, 100% { transform: translateY(0); }
		50% { transform: translateY(-8px); }
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
		color: rgba(255, 255, 255, 0.6);
		margin: 0;
	}

	.progress-wrapper {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.progress-bar {
		flex: 1;
		height: 8px;
		background: rgba(255, 255, 255, 0.1);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: linear-gradient(90deg, #3b82f6, #60a5fa);
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.progress-text {
		font-size: 16px;
		font-weight: 600;
		color: #60a5fa;
		min-width: 48px;
		text-align: right;
	}

	.step-text {
		font-size: 14px;
		color: rgba(255, 255, 255, 0.5);
		margin: 0;
	}

	.version-text {
		font-size: 16px;
		color: rgba(255, 255, 255, 0.7);
		margin: 0;
	}

	.error-text {
		font-size: 14px;
		color: #fca5a5;
		margin: 0;
		padding: 12px 20px;
		background: rgba(239, 68, 68, 0.1);
		border-radius: 8px;
		border: 1px solid rgba(239, 68, 68, 0.2);
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
