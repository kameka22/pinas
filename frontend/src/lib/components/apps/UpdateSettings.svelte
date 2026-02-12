<script lang="ts">
	import Icon from '@iconify/svelte';
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { api } from '$stores/api';
	import type { UpdateCheckResult, UpdateHistoryEntry } from '$stores/api';
	import { updateScreen } from '$stores/update';
	import { systemInfo } from '$stores/system';
	import { t, locale } from '$lib/i18n';

	// Update state
	let updateCheck: UpdateCheckResult | null = null;
	let updateHistory: UpdateHistoryEntry[] = [];
	let checking = false;
	let updateError = '';
	let showConfirmModal = false;

	onMount(() => {
		checkForUpdates();
		loadHistory();
	});

	async function checkForUpdates() {
		checking = true;
		updateError = '';
		try {
			updateCheck = await api.checkForUpdate();
		} catch (e) {
			updateError = ($t as any).systemUpdate?.failedToCheck || 'Failed to check for updates';
		} finally {
			checking = false;
		}
	}

	function openConfirmModal() {
		showConfirmModal = true;
	}

	async function confirmInstall() {
		showConfirmModal = false;
		updateError = '';
		try {
			const result = await api.installUpdate();
			updateScreen.set({
				active: true,
				taskId: result.task_id,
				rebootRequired: updateCheck?.reboot_required ?? false,
				version: updateCheck?.latest_version || '',
				changelog: updateCheck?.changelog || null,
				devTest: false
			});
		} catch (e) {
			updateError = ($t as any).systemUpdate?.failedToInstall || 'Failed to install update';
		}
	}

	function launchDevTest() {
		updateScreen.set({
			active: true,
			taskId: null,
			rebootRequired: false,
			version: '1.0.0-dev',
			changelog: null,
			devTest: true
		});
	}

	async function loadHistory() {
		try {
			updateHistory = await api.getUpdateHistory();
		} catch (_) {
			// Ignore
		}
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function formatDate(dateStr: string): string {
		try {
			return new Date(dateStr).toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	function getChangelog(changelog: Record<string, string> | string | null): string {
		if (!changelog) return '';
		const lang = get(locale);
		if (typeof changelog === 'string') {
			try {
				const parsed = JSON.parse(changelog);
				return parsed[lang] || parsed['en'] || '';
			} catch {
				return changelog;
			}
		}
		return changelog[lang] || changelog['en'] || '';
	}

	$: tScreen = ($t as any).systemUpdate?.screen || {};
	$: isDevMode = $systemInfo?.devMode === true;
</script>

<div class="update-settings">
	<div class="section-header">
		<div class="section-icon">
			<Icon icon="mdi:update" class="w-6 h-6" />
		</div>
		<div>
			<h2>{($t as any).systemUpdate?.title || 'System Update'}</h2>
			<p class="section-desc">{($t as any).systemUpdate?.currentVersion || 'Current version'}: {updateCheck?.current_version || '...'}</p>
		</div>
	</div>

	{#if updateError}
		<div class="update-error">
			<Icon icon="mdi:alert-circle" class="w-5 h-5" />
			<span>{updateError}</span>
		</div>
	{/if}

	{#if checking}
		<!-- Checking state -->
		<div class="update-card">
			<div class="update-icon spinning-icon">
				<Icon icon="mdi:loading" class="w-6 h-6" />
			</div>
			<div class="update-info">
				<h4>{($t as any).systemUpdate?.checking || 'Checking...'}</h4>
			</div>
		</div>
	{:else if updateCheck?.available}
		<!-- Update available -->
		<div class="update-card available">
			<div class="update-icon available-icon">
				<Icon icon="mdi:arrow-up-circle" class="w-6 h-6" />
			</div>
			<div class="update-info">
				<h4>{($t as any).systemUpdate?.updateAvailable || 'Update available'}: v{updateCheck.latest_version}</h4>
				{#if updateCheck.changelog}
					<p class="update-changelog">{getChangelog(updateCheck.changelog)}</p>
				{/if}
				<div class="update-meta">
					{#if updateCheck.download_size}
						<span>{formatSize(updateCheck.download_size)}</span>
						<span class="meta-dot">&middot;</span>
					{/if}
					<span>
						{#if updateCheck.reboot_required}
							{($t as any).systemUpdate?.rebootRequired || 'Reboot required'}
						{:else}
							{($t as any).systemUpdate?.noRebootRequired || 'No reboot required'}
						{/if}
					</span>
				</div>
			</div>
			<button class="btn-primary install-btn" on:click={openConfirmModal}>
				<Icon icon="mdi:download" class="w-4 h-4" />
				{($t as any).systemUpdate?.installUpdate || 'Install update'}
			</button>
		</div>
	{:else if updateCheck}
		<!-- Up to date -->
		<div class="update-card up-to-date">
			<div class="update-icon uptodate-icon">
				<Icon icon="mdi:check-circle" class="w-6 h-6" />
			</div>
			<div class="update-info">
				<h4>{($t as any).systemUpdate?.upToDate || 'Your system is up to date'}</h4>
				<p>{($t as any).systemUpdate?.upToDateDesc || 'You are running the latest version.'}</p>
			</div>
			<button class="btn-check" on:click={checkForUpdates} disabled={checking}>
				<Icon icon="mdi:refresh" class="w-4 h-4" />
				{($t as any).systemUpdate?.checkForUpdates || 'Check for updates'}
			</button>
		</div>
	{/if}

	<!-- Dev mode test button -->
	{#if isDevMode}
		<div class="dev-test-section">
			<button class="btn-dev-test" on:click={launchDevTest}>
				<Icon icon="mdi:bug-play" class="w-4 h-4" />
				{tScreen.devTest || 'Test update screen'}
			</button>
		</div>
	{/if}

	<!-- Update History -->
	{#if updateHistory.length > 0}
		<div class="history-section">
			<h3>{($t as any).systemUpdate?.updateHistory || 'Update History'}</h3>
			<div class="history-list">
				{#each updateHistory as entry}
					<div class="history-item" class:failed={entry.status === 'failed'}>
						<div class="history-version">
							<span class="version-tag">v{entry.version}</span>
							<Icon icon="mdi:arrow-left" class="w-3 h-3 text-slate-400" />
							<span class="version-from">v{entry.previous_version}</span>
						</div>
						<div class="history-meta">
							<span class="history-type">{entry.update_type}</span>
							<span class="meta-dot">&middot;</span>
							<span>{formatDate(entry.created_at)}</span>
							{#if entry.status === 'failed'}
								<span class="meta-dot">&middot;</span>
								<span class="history-failed">failed</span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<!-- Confirmation Modal -->
{#if showConfirmModal}
	<div class="modal-overlay" on:click={() => showConfirmModal = false} on:keydown={(e) => e.key === 'Escape' && (showConfirmModal = false)} role="dialog" tabindex="-1">
		<div class="modal-card" on:click|stopPropagation role="document">
			<div class="modal-icon">
				<Icon icon="mdi:arrow-up-circle" class="w-8 h-8" />
			</div>
			<h3 class="modal-title">
				{(tScreen.confirm?.title || 'Install update v{version}?').replace('{version}', updateCheck?.latest_version || '')}
			</h3>
			<p class="modal-desc">
				{tScreen.confirm?.description || 'This will update your system. Do not unplug the device.'}
			</p>
			<div class="modal-meta">
				{#if updateCheck?.download_size}
					<span>{formatSize(updateCheck.download_size)}</span>
					<span class="meta-dot">&middot;</span>
				{/if}
				<span>
					{#if updateCheck?.reboot_required}
						{($t as any).systemUpdate?.rebootRequired || 'Reboot required'}
					{:else}
						{($t as any).systemUpdate?.noRebootRequired || 'No reboot required'}
					{/if}
				</span>
			</div>
			<div class="modal-actions">
				<button class="btn-modal-cancel" on:click={() => showConfirmModal = false}>
					{tScreen.confirm?.cancel || 'Cancel'}
				</button>
				<button class="btn-modal-confirm" on:click={confirmInstall}>
					{tScreen.confirm?.confirm || 'Confirm'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.update-settings {
		padding: 24px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 24px;
	}

	.section-icon {
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: linear-gradient(135deg, #3b82f6, #2563eb);
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
	}

	h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.section-desc {
		font-size: 13px;
		color: #64748b;
		margin-top: 2px;
	}

	.update-error {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		background: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 8px;
		color: #dc2626;
		font-size: 14px;
		margin-bottom: 16px;
	}

	.update-card {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 20px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
	}

	.update-card.available {
		border-color: #93c5fd;
		background: #eff6ff;
	}

	.update-card.up-to-date {
		border-color: #86efac;
		background: #f0fdf4;
	}

	.update-icon {
		width: 44px;
		height: 44px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		background: #e2e8f0;
		color: #64748b;
	}

	.update-icon.available-icon {
		background: #dbeafe;
		color: #3b82f6;
	}

	.update-icon.uptodate-icon {
		background: #dcfce7;
		color: #16a34a;
	}

	.spinning-icon {
		background: #dbeafe;
		color: #3b82f6;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.update-info {
		flex: 1;
		min-width: 0;
	}

	.update-info h4 {
		font-size: 15px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 4px;
	}

	.update-info p {
		font-size: 13px;
		color: #64748b;
		margin-bottom: 8px;
	}

	.update-changelog {
		font-size: 13px;
		color: #475569;
		line-height: 1.5;
		margin-bottom: 8px;
		white-space: pre-line;
	}

	.update-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: #94a3b8;
	}

	.meta-dot {
		color: #cbd5e1;
	}

	.install-btn {
		flex-shrink: 0;
		align-self: center;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 20px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
		cursor: pointer;
	}

	.install-btn:hover {
		background: #2563eb;
	}

	.btn-primary {
		/* used via install-btn */
	}

	.btn-check {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 13px;
		color: #475569;
		flex-shrink: 0;
		align-self: center;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-check:hover:not(:disabled) {
		background: #f1f5f9;
	}

	.btn-check:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Dev test section */
	.dev-test-section {
		margin-top: 16px;
	}

	.btn-dev-test {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		background: #fef3c7;
		border: 1px solid #fcd34d;
		border-radius: 8px;
		font-size: 13px;
		color: #92400e;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-dev-test:hover {
		background: #fde68a;
	}

	/* History */
	.history-section {
		margin-top: 32px;
	}

	.history-section h3 {
		font-size: 15px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 12px;
	}

	.history-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.history-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
	}

	.history-item.failed {
		border-color: #fecaca;
		background: #fef2f2;
	}

	.history-version {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.version-tag {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
	}

	.version-from {
		font-size: 13px;
		color: #94a3b8;
	}

	.history-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: #94a3b8;
	}

	.history-type {
		padding: 2px 8px;
		background: #e2e8f0;
		border-radius: 4px;
		font-size: 11px;
		color: #64748b;
		text-transform: uppercase;
	}

	.history-failed {
		color: #dc2626;
		font-weight: 500;
	}

	/* Confirmation Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}

	.modal-card {
		background: white;
		border-radius: 16px;
		padding: 32px;
		max-width: 420px;
		width: 90%;
		text-align: center;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
	}

	.modal-icon {
		width: 56px;
		height: 56px;
		border-radius: 14px;
		background: #dbeafe;
		color: #3b82f6;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 0 auto 16px;
	}

	.modal-title {
		font-size: 17px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 8px;
	}

	.modal-desc {
		font-size: 14px;
		color: #64748b;
		line-height: 1.5;
		margin-bottom: 12px;
	}

	.modal-meta {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		font-size: 12px;
		color: #94a3b8;
		margin-bottom: 24px;
	}

	.modal-actions {
		display: flex;
		gap: 12px;
		justify-content: center;
	}

	.btn-modal-cancel {
		padding: 10px 24px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		background: white;
		border: 1px solid #e2e8f0;
		color: #475569;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-modal-cancel:hover {
		background: #f1f5f9;
	}

	.btn-modal-confirm {
		padding: 10px 24px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		background: #3b82f6;
		color: white;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-modal-confirm:hover {
		background: #2563eb;
	}
</style>
