<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '#lib/i18n/index.js';
	import { api, type SshStatus } from '#lib/stores/api.js';
	import PasswordRules from '#lib/components/ui/PasswordRules.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';

	// SSH state
	let sshStatus: SshStatus | null = null;
	let sshLoading = false;
	let sshError: string | null = null;
	let sshToggling = false;

	// Password change state
	let showPasswordForm = false;
	let newPassword = '';
	let confirmPassword = '';
	let passwordError: string | null = null;
	let passwordSaving = false;

	onMount(() => {
		loadSshStatus();
	});

	async function loadSshStatus() {
		sshLoading = true;
		sshError = null;
		try {
			sshStatus = await api.getSshStatus();
		} catch (e) {
			sshError = errorMessage(e, $t.common.errors.loadSshStatus);
		} finally {
			sshLoading = false;
		}
	}

	async function toggleSsh() {
		if (!sshStatus || sshToggling) return;

		sshToggling = true;
		sshError = null;

		try {
			if (sshStatus.enabled) {
				await api.disableSsh();
			} else {
				await api.enableSsh();
			}
			await loadSshStatus();
		} catch (e) {
			sshError = errorMessage(e, $t.common.errors.toggleSsh);
		} finally {
			sshToggling = false;
		}
	}

	function openPasswordForm() {
		newPassword = '';
		confirmPassword = '';
		passwordError = null;
		showPasswordForm = true;
	}

	function closePasswordForm() {
		showPasswordForm = false;
	}

	async function savePassword() {
		passwordError = null;

		if (newPassword.length < 8) {
			passwordError = $t.fileService.ssh.passwordTooShort;
			return;
		}

		if (newPassword !== confirmPassword) {
			passwordError = $t.fileService.ssh.passwordMismatch;
			return;
		}

		passwordSaving = true;

		try {
			await api.changeSshPassword(newPassword);
			closePasswordForm();
			toasts.success($t.fileService.ssh.passwordChanged);
		} catch (e) {
			passwordError = errorMessage(e, $t.common.errors.changePassword);
		} finally {
			passwordSaving = false;
		}
	}
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.terminal} description={$t.settingsUi.descriptions.terminal} icon="mdi:console-line" />

	{#if sshLoading && !sshStatus}
		<SettingsState kind="loading" message={$t.common.loading} />
	{:else if sshError && !sshStatus}
		<SettingsState kind="error" message={sshError} onretry={loadSshStatus} />
	{:else if sshStatus}
		{#if sshError}<SettingsState kind="error" message={sshError} onretry={loadSshStatus} />{/if}
		<SettingsSection title="SSH" description={$t.fileService.ssh.description} icon="mdi:console">
			{#snippet actions()}
				{#if sshStatus?.enabled}
					<span class="settings-badge" class:success={sshStatus.running}>
						{sshStatus.running ? $t.fileService.ssh.running : $t.fileService.ssh.stopped}
					</span>
				{/if}
			{/snippet}
			<SettingsRow label={$t.fileService.ssh.enable} hint={$t.fileService.ssh.enableHint}>
				<SettingsToggle label={$t.fileService.ssh.enable} checked={sshStatus.enabled} disabled={sshToggling} onchange={toggleSsh} />
			</SettingsRow>

			{#if sshStatus.enabled}
				<SettingsRow label={$t.fileService.ssh.port}>
					<span class="settings-mono">{sshStatus.port}</span>
				</SettingsRow>
				<SettingsRow label={$t.fileService.ssh.password} hint={$t.fileService.ssh.passwordHint}>
					{#if !showPasswordForm}
						<button class="settings-button" on:click={openPasswordForm}>
							<Icon icon="mdi:key" width="16" />
							{$t.fileService.ssh.changePassword}
						</button>
					{/if}
				</SettingsRow>

				{#if showPasswordForm}
					<form class="inline-form" on:submit|preventDefault={savePassword}>
						<p class="settings-help">{$t.fileService.ssh.passwordDescription}</p>
						{#if passwordError}
							<div class="settings-notice danger" role="alert"><Icon icon="mdi:alert-circle" width="18" /><span>{passwordError}</span></div>
						{/if}
						<div class="settings-grid">
							<div class="settings-field">
								<label for="new-password">{$t.fileService.ssh.newPassword}</label>
								<input id="new-password" type="password" bind:value={newPassword} autocomplete="new-password" />
							</div>
							<div class="settings-field">
								<label for="confirm-password">{$t.fileService.ssh.confirmPassword}</label>
								<input id="confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" />
							</div>
						</div>
						<PasswordRules password={newPassword} {confirmPassword} showMatch={true} />
						<div class="settings-actions-buttons">
							<button type="button" class="settings-button" on:click={closePasswordForm} disabled={passwordSaving}>{$t.common.cancel}</button>
							<button type="submit" class="settings-button primary" disabled={passwordSaving}>
								{passwordSaving ? $t.common.loading : $t.common.save}
							</button>
						</div>
					</form>
				{/if}

				<div class="settings-notice">
					<Icon icon="mdi:information-outline" width="18" />
					<span>{$t.fileService.ssh.connectionInfo} <code class="settings-mono">ssh root@&lbrace;IP&rbrace; -p {sshStatus.port}</code></span>
				</div>
			{/if}
		</SettingsSection>
	{/if}
</div>

<style>
	.inline-form { display: flex; flex-direction: column; gap: 12px; padding: 16px; margin: 4px 0 12px; border: 1px solid #e2e8f0; border-radius: 10px; background: #f8fafc; }
</style>
