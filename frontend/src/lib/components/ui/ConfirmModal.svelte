<script lang="ts">
	import Icon from '@iconify/svelte';
	import { createEventDispatcher } from 'svelte';

	export let visible = false;
	export let title = '';
	export let message = '';
	export let confirmLabel = 'Confirm';
	export let cancelLabel = 'Cancel';
	export let danger = false;
	export let loading = false;

	const dispatch = createEventDispatcher();

	function handleConfirm() {
		if (!loading) dispatch('confirm');
	}

	function handleCancel() {
		if (!loading) dispatch('cancel');
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && !loading) handleCancel();
	}
</script>

{#if visible}
	<div class="modal-overlay" on:click={handleCancel} on:keydown={handleKeydown} role="presentation">
		<div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
			<div class="modal-header">
				<h2>
					<Icon icon={danger ? 'mdi:alert-circle' : 'mdi:help-circle'} class="w-5 h-5" />
					{title}
				</h2>
				<button class="close-btn" on:click={handleCancel} disabled={loading}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				<p>{message}</p>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={handleCancel} disabled={loading}>
					{cancelLabel}
				</button>
				<button
					class="btn-confirm"
					class:danger
					on:click={handleConfirm}
					disabled={loading}
				>
					{#if loading}
						<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
					{/if}
					{confirmLabel}
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
		border-radius: 16px;
		width: 100%;
		max-width: 420px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
		animation: modal-slide-in 0.2s ease-out;
	}

	@keyframes modal-slide-in {
		from { opacity: 0; transform: translateY(-20px); }
		to { opacity: 1; transform: translateY(0); }
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
		border: none;
		background: transparent;
		cursor: pointer;
	}

	.close-btn:hover {
		background: #f1f5f9;
		color: #1e293b;
	}

	.modal-body {
		padding: 24px;
	}

	.modal-body p {
		font-size: 14px;
		color: #475569;
		line-height: 1.6;
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

	.btn-secondary,
	.btn-confirm {
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

	.btn-confirm {
		background: linear-gradient(135deg, #3b82f6, #2563eb);
		color: white;
	}

	.btn-confirm:hover:not(:disabled) {
		background: linear-gradient(135deg, #2563eb, #1d4ed8);
	}

	.btn-confirm.danger {
		background: linear-gradient(135deg, #ef4444, #dc2626);
	}

	.btn-confirm.danger:hover:not(:disabled) {
		background: linear-gradient(135deg, #dc2626, #b91c1c);
	}

	.btn-confirm:disabled {
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
