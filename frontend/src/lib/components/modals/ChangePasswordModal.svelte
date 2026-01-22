<script lang="ts">
	import Icon from '@iconify/svelte';
	import { api } from '$stores/api';
	import { t } from '$lib/i18n';

	export let show = false;

	let currentPassword = '';
	let newPassword = '';
	let confirmPassword = '';
	let saving = false;
	let error = '';
	let success = false;

	function resetForm() {
		currentPassword = '';
		newPassword = '';
		confirmPassword = '';
		error = '';
		success = false;
	}

	function handleClose() {
		resetForm();
		show = false;
	}

	async function handleSubmit() {
		error = '';

		// Validate
		if (!currentPassword) {
			error = 'Current password is required';
			return;
		}

		if (!newPassword) {
			error = 'New password is required';
			return;
		}

		if (newPassword.length < 6) {
			error = 'Password must be at least 6 characters';
			return;
		}

		if (newPassword !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		saving = true;

		try {
			await api.changePassword(currentPassword, newPassword);
			success = true;
			// Reset form after success
			currentPassword = '';
			newPassword = '';
			confirmPassword = '';
			// Close after a brief delay
			setTimeout(() => {
				handleClose();
			}, 1500);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to change password';
		} finally {
			saving = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			handleClose();
		} else if (e.key === 'Enter' && !saving) {
			handleSubmit();
		}
	}
</script>

{#if show}
	<div class="modal-overlay" on:click={handleClose} on:keydown={handleKeydown} role="presentation">
		<div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:key-variant" class="w-5 h-5" />
					{$t.topBar?.changePassword || 'Change Password'}
				</h2>
				<button class="close-btn" on:click={handleClose}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if success}
					<div class="success-message">
						<Icon icon="mdi:check-circle" class="w-6 h-6" />
						<span>Password changed successfully!</span>
					</div>
				{:else}
					{#if error}
						<div class="error-message">
							<Icon icon="mdi:alert-circle" class="w-5 h-5" />
							<span>{error}</span>
						</div>
					{/if}

					<div class="form-group">
						<label for="current-password">Current Password</label>
						<input
							id="current-password"
							type="password"
							bind:value={currentPassword}
							placeholder="Enter current password"
							disabled={saving}
						/>
					</div>

					<div class="form-group">
						<label for="new-password">New Password</label>
						<input
							id="new-password"
							type="password"
							bind:value={newPassword}
							placeholder="Enter new password"
							disabled={saving}
						/>
					</div>

					<div class="form-group">
						<label for="confirm-password">Confirm New Password</label>
						<input
							id="confirm-password"
							type="password"
							bind:value={confirmPassword}
							placeholder="Confirm new password"
							disabled={saving}
						/>
					</div>
				{/if}
			</div>

			{#if !success}
				<div class="modal-footer">
					<button class="btn-secondary" on:click={handleClose} disabled={saving}>
						Cancel
					</button>
					<button class="btn-primary" on:click={handleSubmit} disabled={saving}>
						{#if saving}
							<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
							Saving...
						{:else}
							<Icon icon="mdi:check" class="w-4 h-4" />
							Change Password
						{/if}
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
		animation: fade-in 0.15s ease-out;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.modal {
		background: white;
		border-radius: 16px;
		width: 100%;
		max-width: 420px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
		animation: modal-slide-in 0.2s ease-out;
	}

	@keyframes modal-slide-in {
		from {
			opacity: 0;
			transform: translateY(-20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px 24px;
		border-bottom: 1px solid #e2e8f0;
	}

	.modal-header h2 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.close-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		color: #64748b;
		transition: all 0.15s;
	}

	.close-btn:hover {
		background: #f1f5f9;
		color: #1e293b;
	}

	.modal-body {
		padding: 24px;
	}

	.error-message {
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

	.success-message {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 24px;
		background: #f0fdf4;
		border: 1px solid #bbf7d0;
		border-radius: 12px;
		color: #16a34a;
		font-size: 16px;
		font-weight: 500;
	}

	.form-group {
		margin-bottom: 16px;
	}

	.form-group:last-child {
		margin-bottom: 0;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #475569;
		margin-bottom: 6px;
	}

	.form-group input {
		width: 100%;
		padding: 12px 16px;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		font-size: 14px;
		transition: all 0.15s;
		background: #f8fafc;
	}

	.form-group input:focus {
		outline: none;
		border-color: #3b82f6;
		background: white;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-group input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 24px;
		border-top: 1px solid #e2e8f0;
		background: #f8fafc;
		border-radius: 0 0 16px 16px;
	}

	.btn-primary,
	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		border-radius: 10px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
		border: none;
	}

	.btn-primary {
		background: linear-gradient(135deg, #3b82f6, #2563eb);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: linear-gradient(135deg, #2563eb, #1d4ed8);
	}

	.btn-primary:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: white;
		color: #64748b;
		border: 1px solid #e2e8f0;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #f1f5f9;
		color: #475569;
	}

	.btn-secondary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
