<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api, type SshStatus } from '$lib/stores/api';
	import PasswordRules from '$lib/components/ui/PasswordRules.svelte';

	// SSH state
	let sshStatus: SshStatus | null = null;
	let sshLoading = false;
	let sshError: string | null = null;
	let sshToggling = false;

	// Password change state
	let showPasswordModal = false;
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
			sshError = e instanceof Error ? e.message : 'Failed to load SSH status';
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
			sshError = e instanceof Error ? e.message : 'Failed to toggle SSH';
		} finally {
			sshToggling = false;
		}
	}

	function openPasswordModal() {
		newPassword = '';
		confirmPassword = '';
		passwordError = null;
		showPasswordModal = true;
	}

	function closePasswordModal() {
		showPasswordModal = false;
	}

	async function savePassword() {
		passwordError = null;

		if (newPassword.length < 8) {
			passwordError = $t.fileService?.ssh?.passwordTooShort || 'Password must be at least 8 characters';
			return;
		}

		if (newPassword !== confirmPassword) {
			passwordError = $t.fileService?.ssh?.passwordMismatch || 'Passwords do not match';
			return;
		}

		passwordSaving = true;

		try {
			await api.changeSshPassword(newPassword);
			closePasswordModal();
		} catch (e) {
			passwordError = e instanceof Error ? e.message : 'Failed to change password';
		} finally {
			passwordSaving = false;
		}
	}
</script>

<div class="terminal-settings">
	<div class="section">
		<div class="section-header">
			<Icon icon="mdi:console" class="w-6 h-6" />
			<div class="section-info">
				<h2>SSH</h2>
				<p>{$t.fileService?.ssh?.description || 'Secure Shell access for remote administration'}</p>
			</div>
		</div>

		{#if sshLoading}
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
				<span>{$t.common.loading}</span>
			</div>
		{:else if sshError}
			<div class="error-state">
				<Icon icon="mdi:alert-circle" class="w-5 h-5" />
				<span>{sshError}</span>
				<button class="retry-btn" on:click={loadSshStatus}>
					{$t.common.retry || 'Retry'}
				</button>
			</div>
		{:else if sshStatus}
			<div class="service-content">
				<div class="service-row">
					<div class="service-label">
						<span class="label">{$t.fileService?.ssh?.enable || 'Enable SSH'}</span>
						<span class="sublabel">{$t.fileService?.ssh?.enableHint || 'Allow remote terminal access'}</span>
					</div>
					<button
						class="toggle-switch"
						class:active={sshStatus.enabled}
						class:loading={sshToggling}
						on:click={toggleSsh}
						disabled={sshToggling}
					>
						<span class="toggle-knob"></span>
					</button>
				</div>

				{#if sshStatus.enabled}
					<div class="service-row">
						<div class="service-label">
							<span class="label">{$t.fileService?.ssh?.status || 'Status'}</span>
						</div>
						<div class="status-badge" class:running={sshStatus.running}>
							<Icon icon={sshStatus.running ? 'mdi:check-circle' : 'mdi:close-circle'} class="w-4 h-4" />
							{sshStatus.running ? ($t.fileService?.ssh?.running || 'Running') : ($t.fileService?.ssh?.stopped || 'Stopped')}
						</div>
					</div>

					<div class="service-row">
						<div class="service-label">
							<span class="label">{$t.fileService?.ssh?.port || 'Port'}</span>
						</div>
						<span class="port-value">{sshStatus.port}</span>
					</div>

					<div class="service-row">
						<div class="service-label">
							<span class="label">{$t.fileService?.ssh?.password || 'Password'}</span>
							<span class="sublabel">{$t.fileService?.ssh?.passwordHint || 'Used for SSH login as root'}</span>
						</div>
						<button class="btn-secondary" on:click={openPasswordModal}>
							<Icon icon="mdi:key" class="w-4 h-4" />
							{$t.fileService?.ssh?.changePassword || 'Change Password'}
						</button>
					</div>

					<div class="info-box">
						<Icon icon="mdi:information-outline" class="w-5 h-5" />
						<div class="info-content">
							<p>{$t.fileService?.ssh?.connectionInfo || 'Connect using:'}</p>
							<code>ssh root@{'{'}IP{'}'} -p {sshStatus.port}</code>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<!-- Password Change Modal -->
{#if showPasswordModal}
	<div class="modal-overlay" on:click={closePasswordModal}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:key" class="w-5 h-5" />
					{$t.fileService?.ssh?.changePassword || 'Change SSH Password'}
				</h2>
				<button class="modal-close" on:click={closePasswordModal}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				<p class="modal-description">
					{$t.fileService?.ssh?.passwordDescription || 'This password is used to connect via SSH as the root user.'}
				</p>

				{#if passwordError}
					<div class="error-message">
						<Icon icon="mdi:alert-circle" class="w-4 h-4" />
						{passwordError}
					</div>
				{/if}

				<div class="form-group">
					<label for="new-password">{$t.fileService?.ssh?.newPassword || 'New Password'}</label>
					<input
						id="new-password"
						type="password"
						bind:value={newPassword}
						placeholder="********"
						autocomplete="new-password"
					/>
				</div>

				<div class="form-group">
					<label for="confirm-password">{$t.fileService?.ssh?.confirmPassword || 'Confirm Password'}</label>
					<input
						id="confirm-password"
						type="password"
						bind:value={confirmPassword}
						placeholder="********"
						autocomplete="new-password"
					/>
				</div>
				<PasswordRules password={newPassword} confirmPassword={confirmPassword} showMatch={true} />
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={closePasswordModal}>
					{$t.common.cancel}
				</button>
				<button class="btn-primary" on:click={savePassword} disabled={passwordSaving}>
					{#if passwordSaving}
						<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
					{/if}
					{$t.common.save}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.terminal-settings {
		padding: 24px;
	}

	.section {
		margin-bottom: 32px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.section-header > :global(svg) {
		color: #6b7280;
	}

	.section-info h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 4px;
	}

	.section-info p {
		font-size: 13px;
		color: #6b7280;
	}

	.service-content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.service-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
	}

	.service-label {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.service-label .label {
		font-size: 14px;
		font-weight: 500;
		color: #374151;
	}

	.service-label .sublabel {
		font-size: 12px;
		color: #9ca3af;
	}

	/* Toggle switch */
	.toggle-switch {
		position: relative;
		width: 48px;
		height: 28px;
		background: #d1d5db;
		border-radius: 14px;
		cursor: pointer;
		transition: background 0.2s ease;
	}

	.toggle-switch.active {
		background: #3b82f6;
	}

	.toggle-switch.loading {
		opacity: 0.6;
		cursor: wait;
	}

	.toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 24px;
		height: 24px;
		background: white;
		border-radius: 12px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle-switch.active .toggle-knob {
		transform: translateX(20px);
	}

	/* Status badge */
	.status-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		background: #fef2f2;
		color: #dc2626;
	}

	.status-badge.running {
		background: #dcfce7;
		color: #16a34a;
	}

	.port-value {
		font-family: monospace;
		font-size: 14px;
		color: #374151;
		background: #f3f4f6;
		padding: 6px 12px;
		border-radius: 6px;
	}

	.info-box {
		display: flex;
		gap: 12px;
		padding: 16px;
		background: #eff6ff;
		border: 1px solid #bfdbfe;
		border-radius: 8px;
		color: #1d4ed8;
	}

	.info-box .info-content {
		flex: 1;
	}

	.info-box p {
		font-size: 13px;
		margin-bottom: 8px;
	}

	.info-box code {
		display: block;
		font-family: monospace;
		font-size: 13px;
		background: rgba(255, 255, 255, 0.5);
		padding: 8px 12px;
		border-radius: 6px;
	}

	/* States */
	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px 20px;
		color: #9ca3af;
	}

	.error-state {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px;
		background: #fef2f2;
		border-radius: 8px;
		color: #dc2626;
	}

	.retry-btn {
		margin-left: auto;
		padding: 6px 12px;
		background: white;
		border: 1px solid #fecaca;
		border-radius: 6px;
		font-size: 13px;
		color: #dc2626;
	}

	.retry-btn:hover {
		background: #fef2f2;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Buttons */
	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		width: 90%;
		max-width: 420px;
		background: white;
		border-radius: 12px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
	}

	.modal-header h2 {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 16px;
		font-weight: 600;
		color: #1f2937;
	}

	.modal-close {
		color: #6b7280;
		padding: 4px;
		border-radius: 4px;
	}

	.modal-close:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.modal-body {
		padding: 20px;
	}

	.modal-description {
		font-size: 13px;
		color: #6b7280;
		margin-bottom: 16px;
	}

	.error-message {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px;
		background: #fef2f2;
		border-radius: 8px;
		color: #dc2626;
		font-size: 13px;
		margin-bottom: 16px;
	}

	.form-group {
		margin-bottom: 16px;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 6px;
	}

	.form-group input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
	}

	.form-group input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e5e7eb;
	}
</style>
