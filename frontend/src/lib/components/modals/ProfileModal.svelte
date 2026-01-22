<script lang="ts">
	import Icon from '@iconify/svelte';
	import { auth, api } from '$stores/api';
	import { t } from '$lib/i18n';

	export let show = false;

	let email = '';
	let saving = false;
	let error = '';
	let success = false;

	// Load current user data when modal opens
	$: if (show && $auth.user) {
		loadProfile();
	}

	async function loadProfile() {
		try {
			const profile = await api.getProfile();
			email = profile.email || '';
			error = '';
		} catch (e) {
			error = 'Failed to load profile';
		}
	}

	function handleClose() {
		show = false;
		error = '';
		success = false;
	}

	async function handleSave() {
		if (!$auth.user) return;

		saving = true;
		error = '';

		try {
			await api.updateUser($auth.user.id, { email: email || undefined });
			success = true;
			setTimeout(() => {
				success = false;
			}, 2000);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save profile';
		} finally {
			saving = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			handleClose();
		}
	}

	$: userInitial = $auth.user?.username?.[0]?.toUpperCase() || 'U';
	$: userName = $auth.user?.username || 'User';
	$: userRole = $auth.user?.role === 'admin' ? 'Administrator' : 'User';
</script>

{#if show}
	<div class="modal-overlay" on:click={handleClose} on:keydown={handleKeydown} role="presentation">
		<div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:account" class="w-5 h-5" />
					{$t.topBar?.profile || 'Profile'}
				</h2>
				<button class="close-btn" on:click={handleClose}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				<!-- Profile header -->
				<div class="profile-header">
					<div class="profile-avatar">
						<span>{userInitial}</span>
					</div>
					<div class="profile-info">
						<h3>{userName}</h3>
						<span class="profile-role">
							<Icon icon="mdi:shield-account" class="w-4 h-4" />
							{userRole}
						</span>
					</div>
				</div>

				{#if error}
					<div class="error-message">
						<Icon icon="mdi:alert-circle" class="w-5 h-5" />
						<span>{error}</span>
					</div>
				{/if}

				{#if success}
					<div class="success-message">
						<Icon icon="mdi:check-circle" class="w-5 h-5" />
						<span>Profile saved successfully!</span>
					</div>
				{/if}

				<!-- Profile form -->
				<div class="profile-form">
					<div class="form-group">
						<label for="username">Username</label>
						<input
							id="username"
							type="text"
							value={userName}
							disabled
							class="disabled"
						/>
						<span class="field-hint">Username cannot be changed</span>
					</div>

					<div class="form-group">
						<label for="email">Email</label>
						<input
							id="email"
							type="email"
							bind:value={email}
							placeholder="Enter your email"
							disabled={saving}
						/>
					</div>

					<div class="form-group">
						<label>Role</label>
						<div class="role-badge" class:admin={$auth.user?.role === 'admin'}>
							<Icon icon={$auth.user?.role === 'admin' ? 'mdi:shield-crown' : 'mdi:account'} class="w-4 h-4" />
							{userRole}
						</div>
					</div>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={handleClose}>
					Close
				</button>
				<button class="btn-primary" on:click={handleSave} disabled={saving}>
					{#if saving}
						<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
						Saving...
					{:else}
						<Icon icon="mdi:content-save" class="w-4 h-4" />
						Save Changes
					{/if}
				</button>
			</div>
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
		max-width: 480px;
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

	.profile-header {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px;
		background: linear-gradient(135deg, #f8fafc, #f1f5f9);
		border-radius: 12px;
		margin-bottom: 24px;
	}

	.profile-avatar {
		width: 64px;
		height: 64px;
		border-radius: 50%;
		background: linear-gradient(135deg, #3b82f6, #8b5cf6);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 24px;
		font-weight: 600;
		color: white;
	}

	.profile-info h3 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 4px;
	}

	.profile-role {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 14px;
		color: #64748b;
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
		gap: 8px;
		padding: 12px 16px;
		background: #f0fdf4;
		border: 1px solid #bbf7d0;
		border-radius: 8px;
		color: #16a34a;
		font-size: 14px;
		margin-bottom: 16px;
	}

	.profile-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.form-group {
		display: flex;
		flex-direction: column;
	}

	.form-group label {
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

	.form-group input.disabled,
	.form-group input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
		background: #f1f5f9;
	}

	.field-hint {
		font-size: 12px;
		color: #94a3b8;
		margin-top: 4px;
	}

	.role-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		background: #f1f5f9;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: #64748b;
		width: fit-content;
	}

	.role-badge.admin {
		background: #eff6ff;
		color: #3b82f6;
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

	.btn-secondary:hover {
		background: #f1f5f9;
		color: #475569;
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
