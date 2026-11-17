<script lang="ts">
	import Icon from '@iconify/svelte';
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
	import { api } from '#lib/stores/api.js';
	import type { UpdateCheckResult, UpdateHistoryEntry } from '#lib/stores/api.js';
	import { updateScreen } from '#lib/stores/update.js';
	import { systemInfo } from '#lib/stores/system.js';
	import { t, locale } from '#lib/i18n/index.js';
	import ConfirmModal from '#lib/components/ui/ConfirmModal.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { errorMessage } from '#lib/stores/toasts.js';

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
			updateError = errorMessage(e, $t.systemUpdate.failedToCheck);
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
			updateError = errorMessage(e, $t.systemUpdate.failedToInstall);
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
			return new Date(dateStr).toLocaleDateString(get(locale), {
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

	$: isDevMode = $systemInfo?.devMode === true;
</script>

<div class="settings-page">
	<SettingsHeader title={$t.systemUpdate.title} description={`${$t.systemUpdate.currentVersion}${$t.common.labelSeparator} ${updateCheck?.current_version ?? $systemInfo?.version ?? '—'}`} icon="mdi:update" />

	{#if updateError}
		<SettingsState kind="error" message={updateError} onretry={checkForUpdates} />
	{/if}

	<SettingsSection title={$t.systemUpdate.title} icon="mdi:package-up">
		{#snippet actions()}
			{#if !updateCheck?.available}
				<button class="settings-button" on:click={checkForUpdates} disabled={checking}>
					<Icon icon="mdi:refresh" width="16" class={checking ? 'animate-spin' : ''} />
					{$t.systemUpdate.checkForUpdates}
				</button>
			{/if}
		{/snippet}
		{#if checking}
			<SettingsState kind="loading" message={$t.systemUpdate.checking} />
		{:else if updateCheck?.available}
			<div class="settings-notice info update-available">
				<Icon icon="mdi:arrow-up-circle" width="22" />
				<span>
					<strong>{$t.systemUpdate.updateAvailable}{$t.common.labelSeparator} v{updateCheck.latest_version}</strong>
					<span class="update-meta">
						{#if updateCheck.download_size}{formatSize(updateCheck.download_size)} · {/if}
						{updateCheck.reboot_required ? $t.systemUpdate.rebootRequired : $t.systemUpdate.noRebootRequired}
					</span>
				</span>
				<button class="settings-button primary" on:click={openConfirmModal}>
					<Icon icon="mdi:download" width="16" />
					{$t.systemUpdate.installUpdate}
				</button>
			</div>
			{#if updateCheck.changelog}
				<p class="changelog">{getChangelog(updateCheck.changelog)}</p>
			{/if}
		{:else if updateCheck}
			<div class="settings-notice success">
				<Icon icon="mdi:check-circle" width="22" />
				<span><strong>{$t.systemUpdate.upToDate}</strong> <span class="update-meta">{$t.systemUpdate.upToDateDesc}</span></span>
			</div>
		{/if}
		{#if isDevMode}
			<button class="settings-button" on:click={launchDevTest}>
				<Icon icon="mdi:bug-play" width="16" />
				{$t.systemUpdate.screen.devTest}
			</button>
		{/if}
	</SettingsSection>

	<SettingsSection title={$t.systemUpdate.updateHistory} icon="mdi:history">
		{#if updateHistory.length === 0}
			<SettingsState kind="empty" message={$t.systemUpdate.noHistory} />
		{:else}
			<div class="settings-table-wrap">
				<table class="settings-table">
					<tbody>
						{#each updateHistory as entry}
							<tr>
								<td class="settings-mono"><strong>v{entry.version}</strong> ← v{entry.previous_version}</td>
								<td>{entry.update_type}</td>
								<td>{formatDate(entry.created_at)}</td>
								<td>{#if entry.status === 'failed'}<span class="settings-badge danger">{$t.systemUpdate.failed}</span>{/if}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</SettingsSection>
</div>

<ConfirmModal
	visible={showConfirmModal}
	title={$t.systemUpdate.screen.confirm.title.replace('{version}', updateCheck?.latest_version || '')}
	message={`${$t.systemUpdate.screen.confirm.description} ${updateCheck?.reboot_required ? $t.systemUpdate.rebootRequired : $t.systemUpdate.noRebootRequired}.`}
	confirmLabel={$t.systemUpdate.screen.confirm.confirm}
	cancelLabel={$t.systemUpdate.screen.confirm.cancel}
	on:confirm={confirmInstall}
	on:cancel={() => (showConfirmModal = false)}
/>

<style>
	.update-meta { display: block; font-size: 12px; opacity: 0.85; margin-top: 2px; }
	.changelog { white-space: pre-line; font-size: 13px; color: #334155; margin: 0 0 12px; padding: 12px 14px; background: #f8fafc; border-radius: 8px; }
</style>

