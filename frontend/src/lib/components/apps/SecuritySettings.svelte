<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { api, type SecurityStatus, type LoginAttempt } from '$stores/api';
	import { toasts, errorMessage } from '$stores/toasts';
	import { timeAgo } from '$stores/notifications';

	let status: SecurityStatus | null = null;
	let attempts: LoginAttempt[] = [];
	let sessionHours = 24;
	let tlsEnabled = false;
	let saving = false;

	$: dirty = status !== null && (sessionHours !== status.settings.session_hours || tlsEnabled !== status.settings.tls_enabled);

	async function load() {
		try {
			[status, attempts] = await Promise.all([api.getSecurityStatus(), api.getLoginAttempts(50)]);
			sessionHours = status.settings.session_hours;
			tlsEnabled = status.settings.tls_enabled;
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
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

<div class="sec">
	<section class="sec-section">
		<h3 class="sec-title"><Icon icon="mdi:account-clock-outline" class="w-5 h-5" />{$t.security.sessions}</h3>
		<div class="sec-row">
			<label for="session-hours">{$t.security.sessionHours}</label>
			<input id="session-hours" type="number" min="1" max="720" class="sec-input" bind:value={sessionHours} />
			<span class="sec-hint">{$t.security.sessionHint}</span>
		</div>
	</section>

	<section class="sec-section">
		<h3 class="sec-title"><Icon icon="mdi:lock-outline" class="w-5 h-5" />{$t.security.https}</h3>
		{#if status}
			<div class="sec-row">
				<span>{$t.security.httpsEnable}</span>
				<label class="toggle"><input type="checkbox" bind:checked={tlsEnabled} disabled={status.dev_mode} /><span class="toggle-slider"></span></label>
				<span class="sec-badge" class:on={status.tls_active}>{status.tls_active ? $t.security.tlsActive : $t.security.tlsInactive}</span>
			</div>
			{#if status.dev_mode}<p class="sec-hint">{$t.security.devModeTls}</p>{/if}
			{#if status.restart_required}
				<div class="sec-warning">
					<Icon icon="mdi:alert-outline" class="w-4 h-4" />
					<span>{$t.security.restartRequired}</span>
					<button class="btn-secondary" on:click={restart}>{$t.security.restartNow}</button>
				</div>
			{/if}
			<p class="sec-hint">
				{$t.security.selfSigned}
				{#if status.tls_cert_present && status.tls_cert_generated_at}
					· {$t.security.certGenerated} {new Date(status.tls_cert_generated_at).toLocaleString($locale)}
				{/if}
			</p>
			<button class="btn-secondary" on:click={resetCert} disabled={!status.tls_cert_present}>{$t.security.regenerateCert}</button>
		{/if}
	</section>

	<section class="sec-section">
		<h3 class="sec-title"><Icon icon="mdi:shield-alert-outline" class="w-5 h-5" />{$t.security.loginAudit}</h3>
		{#if status}
			<p class="sec-hint">{$t.security.failed24h.replace('{n}', String(status.failed_logins_24h))} · {$t.security.autoBlockHint}</p>
		{/if}
		{#if attempts.length === 0}
			<p class="sec-hint">{$t.security.noAttempts}</p>
		{:else}
			<table class="sec-table">
				<thead><tr><th>{$t.security.when}</th><th>{$t.security.username}</th><th>IP</th><th>{$t.common.status}</th></tr></thead>
				<tbody>
					{#each attempts as a (a.id)}
						<tr class:failed={!a.success}>
							<td title={new Date(a.created_at).toLocaleString($locale)}>{timeAgo(a.created_at, $locale)}</td>
							<td>{a.username}</td>
							<td class="mono">{a.ip}</td>
							<td>{a.success ? $t.security.success : $t.security.failure}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</section>

	<div class="sec-actions">
		<button class="btn-primary" on:click={save} disabled={!dirty || saving}>{$t.common.apply}</button>
	</div>
</div>

<style>
	.sec { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; overflow-y: auto; height: 100%; }
	.sec-section { background: white; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1rem 1.25rem; }
	.sec-title { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9375rem; font-weight: 600; color: #0f172a; margin: 0 0 0.75rem; }
	.sec-row { display: flex; align-items: center; gap: 0.75rem; font-size: 0.8125rem; flex-wrap: wrap; }
	.sec-input { width: 6rem; border: 1px solid #cbd5e1; border-radius: 0.5rem; padding: 0.375rem 0.625rem; font-size: 0.8125rem; }
	.sec-hint { font-size: 0.75rem; color: #64748b; margin: 0.5rem 0; }
	.sec-badge { font-size: 0.6875rem; padding: 0.125rem 0.5rem; border-radius: 999px; background: #fee2e2; color: #b91c1c; }
	.sec-badge.on { background: #dcfce7; color: #15803d; }
	.sec-warning { display: flex; align-items: center; gap: 0.5rem; margin: 0.75rem 0; padding: 0.5rem 0.75rem; border-radius: 0.5rem; background: #fffbeb; color: #92400e; font-size: 0.8125rem; }
	.sec-table { width: 100%; font-size: 0.75rem; border-collapse: collapse; }
	.sec-table th { text-align: left; color: #64748b; font-weight: 500; padding: 0.375rem 0.5rem; border-bottom: 1px solid #e2e8f0; }
	.sec-table td { padding: 0.375rem 0.5rem; border-bottom: 1px solid #f1f5f9; }
	.sec-table tr.failed td { color: #b91c1c; }
	.mono { font-family: ui-monospace, monospace; }
	.sec-actions { display: flex; justify-content: flex-end; }
	.btn-primary, .btn-secondary { display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.4rem 0.875rem; border-radius: 0.5rem; font-size: 0.8125rem; font-weight: 500; }
	.btn-primary { background: #3b82f6; color: white; }
	.btn-primary:disabled, .btn-secondary:disabled { opacity: 0.5; }
	.btn-secondary { background: #f1f5f9; color: #0f172a; }
	.toggle { position: relative; width: 2.25rem; height: 1.25rem; display: inline-block; }
	.toggle input { opacity: 0; width: 0; height: 0; }
	.toggle-slider { position: absolute; inset: 0; background: #cbd5e1; border-radius: 999px; transition: 0.2s; }
	.toggle-slider::before { content: ''; position: absolute; width: 1rem; height: 1rem; left: 0.125rem; top: 0.125rem; background: white; border-radius: 50%; transition: 0.2s; }
	.toggle input:checked + .toggle-slider { background: #3b82f6; }
	.toggle input:checked + .toggle-slider::before { transform: translateX(1rem); }
</style>
