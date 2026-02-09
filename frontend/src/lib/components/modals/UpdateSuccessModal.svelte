<script lang="ts">
	import Icon from '@iconify/svelte';
	import { api } from '$stores/api';
	import { t, locale } from '$lib/i18n';

	export let show = false;
	export let version = '';
	export let previousVersion = '';
	export let changelog: Record<string, string> | null = null;

	let dismissing = false;

	$: currentLocale = $locale;
	$: changelogText = changelog
		? (changelog[currentLocale] || changelog['en'] || '')
		: '';

	async function handleDismiss() {
		dismissing = true;
		try {
			await api.dismissUpdate();
		} catch (e) {
			console.warn('Failed to dismiss update notification:', e);
		}
		dismissing = false;
		show = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			handleDismiss();
		}
	}
</script>

{#if show}
	<div class="modal-overlay" on:click={handleDismiss} on:keydown={handleKeydown} role="presentation">
		<div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
			<div class="modal-content">
				<!-- Animated check icon -->
				<div class="success-icon">
					<Icon icon="mdi:check-circle" class="w-16 h-16" />
				</div>

				<h2>{($t as any).systemUpdate?.modal?.title || 'Update Successful'}</h2>
				<p class="subtitle">{($t as any).systemUpdate?.modal?.subtitle || 'PiNAS has been updated successfully'}</p>

				<div class="version-badge">
					<span class="version-label">{($t as any).systemUpdate?.modal?.updatedTo || 'Updated to version'}</span>
					<span class="version-number">v{version}</span>
					{#if previousVersion}
						<span class="version-from">(v{previousVersion})</span>
					{/if}
				</div>

				{#if changelogText}
					<div class="changelog">
						<h4>{($t as any).systemUpdate?.modal?.changelog || "What's new"}</h4>
						<div class="changelog-content">
							{changelogText}
						</div>
					</div>
				{/if}

				<p class="thanks">{($t as any).systemUpdate?.modal?.thanks || 'Thank you for using PiNAS.'}</p>

				<button class="btn-dismiss" on:click={handleDismiss} disabled={dismissing}>
					{#if dismissing}
						<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
					{/if}
					{($t as any).systemUpdate?.modal?.dismiss || 'Got it!'}
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
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.modal {
		background: white;
		border-radius: 20px;
		width: 100%;
		max-width: 440px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
		animation: modal-slide-in 0.3s ease-out;
		overflow: hidden;
	}

	@keyframes modal-slide-in {
		from {
			opacity: 0;
			transform: translateY(-30px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.modal-content {
		padding: 40px 32px 32px;
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
	}

	.success-icon {
		color: #16a34a;
		margin-bottom: 20px;
		animation: icon-pop 0.4s ease-out 0.15s both;
	}

	@keyframes icon-pop {
		0% {
			opacity: 0;
			transform: scale(0.5);
		}
		70% {
			transform: scale(1.1);
		}
		100% {
			opacity: 1;
			transform: scale(1);
		}
	}

	h2 {
		font-size: 22px;
		font-weight: 700;
		color: #1e293b;
		margin-bottom: 6px;
	}

	.subtitle {
		font-size: 14px;
		color: #64748b;
		margin-bottom: 20px;
	}

	.version-badge {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		background: linear-gradient(135deg, #f0fdf4, #ecfdf5);
		border: 1px solid #bbf7d0;
		border-radius: 10px;
		margin-bottom: 20px;
	}

	.version-label {
		font-size: 13px;
		color: #64748b;
	}

	.version-number {
		font-size: 16px;
		font-weight: 700;
		color: #16a34a;
	}

	.version-from {
		font-size: 12px;
		color: #94a3b8;
	}

	.changelog {
		width: 100%;
		margin-bottom: 20px;
	}

	.changelog h4 {
		font-size: 13px;
		font-weight: 600;
		color: #475569;
		margin-bottom: 8px;
		text-align: left;
	}

	.changelog-content {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		padding: 12px 16px;
		font-size: 13px;
		color: #475569;
		line-height: 1.6;
		text-align: left;
		white-space: pre-line;
		max-height: 150px;
		overflow-y: auto;
	}

	.thanks {
		font-size: 13px;
		color: #94a3b8;
		margin-bottom: 24px;
	}

	.btn-dismiss {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		padding: 14px 24px;
		background: linear-gradient(135deg, #3b82f6, #2563eb);
		color: white;
		border: none;
		border-radius: 12px;
		font-size: 15px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn-dismiss:hover:not(:disabled) {
		background: linear-gradient(135deg, #2563eb, #1d4ed8);
		transform: translateY(-1px);
	}

	.btn-dismiss:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
