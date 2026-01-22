<script lang="ts">
	import Icon from '@iconify/svelte';
	import { api } from '$lib/stores/api';
	import { t } from '$lib/i18n';

	let username = '';
	let password = '';
	let isLoading = false;
	let error: string | null = null;

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!username || !password) {
			error = 'Username and password are required';
			return;
		}

		isLoading = true;
		error = null;

		try {
			await api.login(username, password);
			// The auth store will be updated automatically
			// The layout will detect the change and show the desktop
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="login-container">
	<div class="login-background"></div>

	<div class="login-card">
		<div class="login-header">
			<div class="logo">
				<Icon icon="mdi:nas" class="w-12 h-12" />
			</div>
			<h1>PiNAS</h1>
			<p>Sign in to your account</p>
		</div>

		<form on:submit={handleSubmit}>
			{#if error}
				<div class="error-message">
					<Icon icon="mdi:alert-circle" class="w-4 h-4" />
					<span>{error}</span>
				</div>
			{/if}

			<div class="form-group">
				<label for="username">Username</label>
				<div class="input-wrapper">
					<Icon icon="mdi:account" class="input-icon" />
					<input
						type="text"
						id="username"
						bind:value={username}
						placeholder="Enter your username"
						disabled={isLoading}
						autocomplete="username"
					/>
				</div>
			</div>

			<div class="form-group">
				<label for="password">Password</label>
				<div class="input-wrapper">
					<Icon icon="mdi:lock" class="input-icon" />
					<input
						type="password"
						id="password"
						bind:value={password}
						placeholder="Enter your password"
						disabled={isLoading}
						autocomplete="current-password"
					/>
				</div>
			</div>

			<button type="submit" class="login-btn" disabled={isLoading}>
				{#if isLoading}
					<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
					<span>Signing in...</span>
				{:else}
					<Icon icon="mdi:login" class="w-5 h-5" />
					<span>Sign In</span>
				{/if}
			</button>
		</form>

		<div class="login-footer">
			<p>Secure NAS Management System</p>
		</div>
	</div>
</div>

<style>
	.login-container {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.login-background {
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, #1e3a5f 0%, #0f172a 100%);
		background-image: url('https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=1920&q=80');
		background-size: cover;
		background-position: center;
	}

	.login-background::after {
		content: '';
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
	}

	.login-card {
		position: relative;
		width: 100%;
		max-width: 400px;
		margin: 20px;
		padding: 40px;
		background: rgba(255, 255, 255, 0.95);
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
	}

	.login-header {
		text-align: center;
		margin-bottom: 32px;
	}

	.logo {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 64px;
		height: 64px;
		background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
		border-radius: 16px;
		color: white;
		margin-bottom: 16px;
	}

	.login-header h1 {
		font-size: 28px;
		font-weight: 700;
		color: #1f2937;
		margin: 0 0 8px 0;
	}

	.login-header p {
		font-size: 14px;
		color: #6b7280;
		margin: 0;
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
		margin-bottom: 20px;
	}

	.form-group {
		margin-bottom: 20px;
	}

	.form-group label {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 8px;
	}

	.input-wrapper {
		position: relative;
	}

	.input-icon {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		width: 20px;
		height: 20px;
		color: #9ca3af;
		pointer-events: none;
	}

	.input-wrapper input {
		width: 100%;
		padding: 12px 14px 12px 44px;
		font-size: 15px;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		background: #f9fafb;
		transition: all 0.2s ease;
	}

	.input-wrapper input:focus {
		outline: none;
		border-color: #3b82f6;
		background: white;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.input-wrapper input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.input-wrapper input::placeholder {
		color: #9ca3af;
	}

	.login-btn {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 14px 20px;
		font-size: 15px;
		font-weight: 600;
		color: white;
		background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%);
		border: none;
		border-radius: 10px;
		cursor: pointer;
		transition: all 0.2s ease;
		margin-top: 8px;
	}

	.login-btn:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
	}

	.login-btn:active:not(:disabled) {
		transform: translateY(0);
	}

	.login-btn:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.login-footer {
		margin-top: 32px;
		text-align: center;
	}

	.login-footer p {
		font-size: 12px;
		color: #9ca3af;
		margin: 0;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
