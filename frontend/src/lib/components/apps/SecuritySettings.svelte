<script lang="ts">
	import Icon from '@iconify/svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsActions from '#lib/components/ui/settings/SettingsActions.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { onMount } from 'svelte';
	import { t, locale } from '#lib/i18n/index.js';
	import { api, type SecurityStatus, type LoginAttempt } from '#lib/stores/api.js';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';
	import { timeAgo } from '#lib/stores/notifications.js';

	let status: SecurityStatus | null = null;
	let attempts: LoginAttempt[] = [];
	let sessionHours = 24;
	let tlsEnabled = false;
	let saving = false;
	let loading = true;
	let loadError = '';

	$: dirty = status !== null && (sessionHours !== status.settings.session_hours || tlsEnabled !== status.settings.tls_enabled);

	function resetForm() {
		if (!status) return;
		sessionHours = status.settings.session_hours;
		tlsEnabled = status.settings.tls_enabled;
	}

	async function load() {
		loading = true;
		loadError = '';
		try {
			[status, attempts] = await Promise.all([api.getSecurityStatus(), api.getLoginAttempts(50)]);
			resetForm();
		} catch (e) {
			loadError = errorMessage(e, $t.common.errors.loadFailed);
		} finally {
			loading = false;
		}
	}

	async function save() {
		saving = true;
		try {
			status = await api.updateSecuritySettings({ session_hours: Number(sessionHours), tls_enabled: tlsEnabled });
			toasts.success($t.security.saved);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		} finally {
			saving = false;
		}
	}

	async function resetCert() {
		try {
			await api.resetTlsCertificate();
			toasts.success($t.security.certReset);
			await load();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function restart() {
		try {
			await api.restartPinasService();
			toasts.info($t.security.restarting, 10000);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	onMount(load);
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.security} description={$t.settingsUi.descriptions.security} icon="mdi:shield-check-outline" />

	{#if loading && !status}
		<SettingsState kind="loading" message={$t.common.loading} />
	{:else if loadError && !status}
		<SettingsState kind="error" message={loadError} onretry={load} />
	{:else if status}
		<SettingsSection title={$t.security.sessions} icon="mdi:account-clock-outline">
			<SettingsRow label={$t.security.sessionHours} hint={$t.security.sessionHint} forId="session-hours">
				<input id="session-hours" type="number" min="1" max="720" class="number-input" bind:value={sessionHours} />
			</SettingsRow>
		</SettingsSection>

		<SettingsSection title={$t.security.https} description={$t.security.selfSigned} icon="mdi:lock-outline">
			<SettingsRow label={$t.security.httpsEnable} hint={status.dev_mode ? $t.security.devModeTls : ''}>
				<span class="settings-badge" class:success={status.tls_active}>{status.tls_active ? $t.security.tlsActive : $t.security.tlsInactive}</span>
				<SettingsToggle label={$t.security.httpsEnable} bind:checked={tlsEnabled} disabled={status.dev_mode} />
			</SettingsRow>
			{#if status.restart_required}
				<div class="settings-notice warning">
					<Icon icon="mdi:alert-outline" width="18" />
					<span>{$t.security.restartRequired}</span>
					<button class="settings-button" on:click={restart}>{$t.security.restartNow}</button>
				</div>
			{/if}
			<SettingsRow label={$t.security.certificate} hint={status.tls_cert_present && status.tls_cert_generated_at ? `${$t.security.certGenerated} ${new Date(status.tls_cert_generated_at).toLocaleString($locale)}` : ''}>
				<button class="settings-button" on:click={resetCert} disabled={!status.tls_cert_present}>
					<Icon icon="mdi:certificate-outline" width="16" />{$t.security.regenerateCert}
				</button>
			</SettingsRow>
		</SettingsSection>

		<SettingsActions {dirty} {saving} onsave={save} oncancel={resetForm} />

		<SettingsSection title={$t.security.loginAudit} description={`${$t.security.failed24h.replace('{n}', String(status.failed_logins_24h))} · ${$t.security.autoBlockHint}`} icon="mdi:shield-alert-outline">
			{#if attempts.length === 0}
				<SettingsState kind="empty" message={$t.security.noAttempts} />
			{:else}
				<div class="settings-table-wrap">
					<table class="settings-table">
						<thead><tr><th>{$t.security.when}</th><th>{$t.security.username}</th><th>IP</th><th>{$t.common.status}</th></tr></thead>
						<tbody>
							{#each attempts as a (a.id)}
								<tr>
									<td title={new Date(a.created_at).toLocaleString($locale)}>{timeAgo(a.created_at, $locale)}</td>
									<td>{a.username}</td>
									<td class="settings-mono">{a.ip}</td>
									<td><span class="settings-badge {a.success ? 'success' : 'danger'}">{a.success ? $t.security.success : $t.security.failure}</span></td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</SettingsSection>
	{/if}
</div>

<style>
	.number-input { width: 6rem; }
</style>
