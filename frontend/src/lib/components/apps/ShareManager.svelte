<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { api } from '$lib/stores/api';
	import type { ShareInfo, SmbShareConfig, CreateShareRequest, UpdateShareRequest } from '$lib/stores/api';
	import FolderPicker from '$lib/components/ui/FolderPicker.svelte';

	// State
	let shares: ShareInfo[] = [];
	let loading = true;
	let error: string | null = null;
	let actionLoading = false;
	let actionError: string | null = null;

	// Modals
	let showCreateModal = false;
	let showEditModal = false;
	let showDeleteConfirm = false;
	let showAdvanced = false;
	let selectedShare: ShareInfo | null = null;

	// Create form
	let newShare = {
		name: '',
		path: '',
		share_type: 'smb',
		description: '',
		guest_ok: false,
		browseable: true,
		read_only: false,
		create_mask: '0644',
		directory_mask: '0755',
		recycle_bin: false
	};

	// Edit form
	let editData = {
		name: '',
		description: '',
		guest_ok: false,
		browseable: true,
		read_only: false,
		create_mask: '0644',
		directory_mask: '0755',
		recycle_bin: false
	};

	onMount(loadShares);

	async function loadShares() {
		loading = true;
		error = null;
		try {
			shares = await api.getShares();
		} catch (e: any) {
			error = e.message || 'Failed to load shares';
		} finally {
			loading = false;
		}
	}

	function resetNewShare() {
		newShare = {
			name: '', path: '', share_type: 'smb', description: '',
			guest_ok: false, browseable: true, read_only: false,
			create_mask: '0644', directory_mask: '0755', recycle_bin: false
		};
		showAdvanced = false;
	}

	function openCreateModal() {
		resetNewShare();
		actionError = null;
		showCreateModal = true;
	}

	function openEditModal(share: ShareInfo) {
		selectedShare = share;
		editData = {
			name: share.name,
			description: share.description || '',
			guest_ok: share.config.guest_ok,
			browseable: share.config.browseable,
			read_only: share.config.read_only,
			create_mask: share.config.create_mask,
			directory_mask: share.config.directory_mask,
			recycle_bin: share.config.recycle_bin
		};
		actionError = null;
		showEditModal = true;
	}

	function openDeleteConfirm(share: ShareInfo) {
		selectedShare = share;
		actionError = null;
		showDeleteConfirm = true;
	}

	async function submitCreate() {
		if (!newShare.name.trim() || !newShare.path.trim()) return;
		actionLoading = true;
		actionError = null;
		try {
			const req: CreateShareRequest = {
				name: newShare.name.trim(),
				path: newShare.path.trim(),
				share_type: newShare.share_type,
				description: newShare.description.trim() || undefined,
				config: {
					guest_ok: newShare.guest_ok,
					browseable: newShare.browseable,
					read_only: newShare.read_only,
					create_mask: newShare.create_mask,
					directory_mask: newShare.directory_mask,
					recycle_bin: newShare.recycle_bin
				}
			};
			await api.createShare(req);
			showCreateModal = false;
			await loadShares();
		} catch (e: any) {
			actionError = e.message || ($t.shareManager?.messages?.createError || 'Failed to create share');
		} finally {
			actionLoading = false;
		}
	}

	async function submitEdit() {
		if (!selectedShare) return;
		actionLoading = true;
		actionError = null;
		try {
			const req: UpdateShareRequest = {
				name: editData.name.trim() || undefined,
				description: editData.description.trim() || null,
				config: {
					guest_ok: editData.guest_ok,
					browseable: editData.browseable,
					read_only: editData.read_only,
					create_mask: editData.create_mask,
					directory_mask: editData.directory_mask,
					recycle_bin: editData.recycle_bin
				}
			};
			await api.updateShare(selectedShare.id, req);
			showEditModal = false;
			await loadShares();
		} catch (e: any) {
			actionError = e.message || ($t.shareManager?.messages?.updateError || 'Failed to update share');
		} finally {
			actionLoading = false;
		}
	}

	async function submitDelete() {
		if (!selectedShare) return;
		actionLoading = true;
		actionError = null;
		try {
			await api.deleteShare(selectedShare.id);
			showDeleteConfirm = false;
			selectedShare = null;
			await loadShares();
		} catch (e: any) {
			actionError = e.message || ($t.shareManager?.messages?.deleteError || 'Failed to delete share');
		} finally {
			actionLoading = false;
		}
	}

	async function handleToggle(share: ShareInfo) {
		try {
			await api.toggleShare(share.id, !share.enabled);
			await loadShares();
		} catch (e: any) {
			error = e.message || 'Failed to toggle share';
		}
	}

	function getProtocolIcon(shareType: string): string {
		switch (shareType) {
			case 'smb': return 'mdi:microsoft-windows';
			case 'nfs': return 'mdi:folder-network';
			default: return 'mdi:folder';
		}
	}

	function getProtocolLabel(shareType: string): string {
		switch (shareType) {
			case 'smb': return 'SMB/CIFS';
			case 'nfs': return 'NFS';
			default: return shareType;
		}
	}
</script>

<div class="share-manager">
	<header class="header">
		<h1>{$t.shareManager?.title || 'Shared Folders'}</h1>
		<button class="btn-primary" on:click={openCreateModal}>
			<Icon icon="mdi:plus" class="w-4 h-4" />
			{$t.shareManager?.createShare || 'Create Share'}
		</button>
	</header>

	<div class="content">
		{#if loading}
			<div class="loading">
				<Icon icon="mdi:loading" class="w-8 h-8 spin" />
			</div>
		{:else if error}
			<div class="error-state">
				<Icon icon="mdi:alert-circle" class="w-8 h-8" />
				<p>{error}</p>
				<button class="btn-secondary" on:click={loadShares}>Retry</button>
			</div>
		{:else if shares.length === 0}
			<div class="empty-state">
				<Icon icon="mdi:folder-off" class="w-12 h-12" />
				<p>{$t.shareManager?.messages?.noShares || 'No shared folders configured'}</p>
				<button class="btn-primary" on:click={openCreateModal}>
					<Icon icon="mdi:plus" class="w-4 h-4" />
					{$t.shareManager?.createShare || 'Create Share'}
				</button>
			</div>
		{:else}
			<div class="share-grid">
				{#each shares as share}
					<div class="share-card">
						<div class="share-header">
							<div class="share-icon" class:enabled={share.enabled}>
								<Icon icon="mdi:folder-open" class="w-6 h-6" />
							</div>
							<div class="share-info">
								<h3>{share.name}</h3>
								<p>{share.path}</p>
								{#if share.description}
									<p class="share-desc">{share.description}</p>
								{/if}
							</div>
						</div>

						<div class="share-meta">
							<div class="meta-item">
								<Icon icon={getProtocolIcon(share.share_type)} class="w-4 h-4" />
								<span>{getProtocolLabel(share.share_type)}</span>
							</div>
							<div class="meta-item">
								<Icon icon="mdi:account-group" class="w-4 h-4" />
								<span>{share.permissions.length} permissions</span>
							</div>
						</div>

						<div class="share-footer">
							<button
								class="status-badge"
								class:active={share.enabled}
								on:click={() => handleToggle(share)}
								title={share.enabled ? ($t.shareManager?.toggleDisabled || 'Disable') : ($t.shareManager?.toggleEnabled || 'Enable')}
							>
								{share.enabled ? 'Active' : 'Disabled'}
							</button>
							<div class="action-btns">
								<button class="action-btn" title={$t.shareManager?.editShare || 'Edit'} on:click={() => openEditModal(share)}>
									<Icon icon="mdi:pencil" class="w-4 h-4" />
								</button>
								<button class="action-btn danger" title={$t.shareManager?.deleteShare || 'Delete'} on:click={() => openDeleteConfirm(share)}>
									<Icon icon="mdi:delete" class="w-4 h-4" />
								</button>
							</div>
						</div>
					</div>
				{/each}

				<button class="add-card" on:click={openCreateModal}>
					<Icon icon="mdi:plus-circle-outline" class="w-8 h-8" />
					<span>{$t.shareManager?.createShare || 'Create New Share'}</span>
				</button>
			</div>
		{/if}
	</div>
</div>

<!-- Create Modal -->
{#if showCreateModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={() => showCreateModal = false}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.shareManager?.createShare || 'Create New Share'}</h2>
				<button class="btn-close" on:click={() => showCreateModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if actionError}
					<div class="form-error">{actionError}</div>
				{/if}
				<div class="form-group">
					<label>{$t.shareManager?.fields?.name || 'Share Name'}</label>
					<input type="text" bind:value={newShare.name} placeholder={$t.shareManager?.fields?.namePlaceholder || 'Enter share name'} />
				</div>
				<div class="form-group">
					<FolderPicker
						bind:value={newShare.path}
						label={$t.shareManager?.fields?.path || 'Path'}
						placeholder="/storage/shares/"
					/>
				</div>
				<div class="form-group">
					<label>{$t.shareManager?.fields?.protocol || 'Protocol'}</label>
					<select bind:value={newShare.share_type}>
						<option value="smb">SMB/CIFS</option>
						<option value="nfs">NFS</option>
					</select>
				</div>
				<div class="form-group">
					<label>{$t.shareManager?.description || 'Description'}</label>
					<input type="text" bind:value={newShare.description} placeholder={$t.shareManager?.descriptionPlaceholder || 'Enter share description'} />
				</div>

				<!-- Advanced Options -->
				<button class="advanced-toggle" on:click={() => showAdvanced = !showAdvanced}>
					<Icon icon={showAdvanced ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
					{$t.shareManager?.advancedOptions || 'Advanced Options'}
				</button>

				{#if showAdvanced}
					<div class="advanced-options">
						<div class="form-group checkbox">
							<label>
								<input type="checkbox" bind:checked={newShare.guest_ok} />
								{$t.shareManager?.guestAccess || 'Guest Access'}
							</label>
						</div>
						<div class="form-group checkbox">
							<label>
								<input type="checkbox" bind:checked={newShare.browseable} />
								{$t.shareManager?.browseable || 'Browseable'}
							</label>
						</div>
						<div class="form-group checkbox">
							<label>
								<input type="checkbox" bind:checked={newShare.read_only} />
								{$t.shareManager?.readOnly || 'Read Only'}
							</label>
						</div>
						<div class="form-group checkbox">
							<label>
								<input type="checkbox" bind:checked={newShare.recycle_bin} />
								{$t.shareManager?.recycleBin || 'Recycle Bin'}
							</label>
						</div>
						<div class="form-row">
							<div class="form-group">
								<label>{$t.shareManager?.createMask || 'File Mask'}</label>
								<input type="text" bind:value={newShare.create_mask} placeholder="0644" />
							</div>
							<div class="form-group">
								<label>{$t.shareManager?.directoryMask || 'Dir Mask'}</label>
								<input type="text" bind:value={newShare.directory_mask} placeholder="0755" />
							</div>
						</div>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showCreateModal = false} disabled={actionLoading}>{$t.common.cancel}</button>
				<button class="btn-primary" on:click={submitCreate} disabled={actionLoading || !newShare.name.trim() || !newShare.path.trim()}>
					{#if actionLoading}
						<Icon icon="mdi:loading" class="w-4 h-4 spin" />
					{/if}
					{$t.shareManager?.createShare || 'Create Share'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Edit Modal -->
{#if showEditModal && selectedShare}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={() => showEditModal = false}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.shareManager?.editShare || 'Edit Share'}</h2>
				<button class="btn-close" on:click={() => showEditModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if actionError}
					<div class="form-error">{actionError}</div>
				{/if}
				<div class="form-group">
					<label>{$t.shareManager?.fields?.name || 'Share Name'}</label>
					<input type="text" bind:value={editData.name} />
				</div>
				<div class="form-group">
					<label>{$t.shareManager?.description || 'Description'}</label>
					<input type="text" bind:value={editData.description} placeholder={$t.shareManager?.descriptionPlaceholder || 'Enter share description'} />
				</div>
				<div class="form-group checkbox">
					<label>
						<input type="checkbox" bind:checked={editData.guest_ok} />
						{$t.shareManager?.guestAccess || 'Guest Access'}
					</label>
				</div>
				<div class="form-group checkbox">
					<label>
						<input type="checkbox" bind:checked={editData.browseable} />
						{$t.shareManager?.browseable || 'Browseable'}
					</label>
				</div>
				<div class="form-group checkbox">
					<label>
						<input type="checkbox" bind:checked={editData.read_only} />
						{$t.shareManager?.readOnly || 'Read Only'}
					</label>
				</div>
				<div class="form-group checkbox">
					<label>
						<input type="checkbox" bind:checked={editData.recycle_bin} />
						{$t.shareManager?.recycleBin || 'Recycle Bin'}
					</label>
				</div>
				<div class="form-row">
					<div class="form-group">
						<label>{$t.shareManager?.createMask || 'File Mask'}</label>
						<input type="text" bind:value={editData.create_mask} />
					</div>
					<div class="form-group">
						<label>{$t.shareManager?.directoryMask || 'Dir Mask'}</label>
						<input type="text" bind:value={editData.directory_mask} />
					</div>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showEditModal = false} disabled={actionLoading}>{$t.common.cancel}</button>
				<button class="btn-primary" on:click={submitEdit} disabled={actionLoading}>
					{#if actionLoading}
						<Icon icon="mdi:loading" class="w-4 h-4 spin" />
					{/if}
					{$t.common.save || 'Save'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation -->
{#if showDeleteConfirm && selectedShare}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={() => showDeleteConfirm = false}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="modal modal-sm" on:click|stopPropagation>
			<div class="modal-header">
				<h2>{$t.shareManager?.deleteShare || 'Delete Share'}</h2>
				<button class="btn-close" on:click={() => showDeleteConfirm = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if actionError}
					<div class="form-error">{actionError}</div>
				{/if}
				<p class="confirm-text">
					{$t.shareManager?.messages?.deleteConfirm || 'Are you sure you want to delete this share?'}
				</p>
				<p class="confirm-detail"><strong>{selectedShare.name}</strong> ({selectedShare.path})</p>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showDeleteConfirm = false} disabled={actionLoading}>{$t.common.cancel}</button>
				<button class="btn-danger" on:click={submitDelete} disabled={actionLoading}>
					{#if actionLoading}
						<Icon icon="mdi:loading" class="w-4 h-4 spin" />
					{/if}
					{$t.shareManager?.deleteShare || 'Delete'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.share-manager {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e2e8f0;
	}

	.header h1 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	.loading, .empty-state, .error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 60px 20px;
		color: #64748b;
	}

	.error-state {
		color: #dc2626;
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.share-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 16px;
	}

	.share-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 16px;
	}

	.share-header {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin-bottom: 16px;
	}

	.share-icon {
		width: 48px;
		height: 48px;
		background: #e2e8f0;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #94a3b8;
		flex-shrink: 0;
	}

	.share-icon.enabled {
		background: #dbeafe;
		color: #3b82f6;
	}

	.share-info {
		flex: 1;
		min-width: 0;
	}

	.share-info h3 {
		font-size: 15px;
		font-weight: 600;
		color: #1e293b;
	}

	.share-info p {
		font-size: 12px;
		color: #64748b;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.share-desc {
		margin-top: 2px;
		font-style: italic;
	}

	.share-meta {
		display: flex;
		gap: 16px;
		margin-bottom: 16px;
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: #64748b;
	}

	.share-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.status-badge {
		padding: 4px 12px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
		background: #f1f5f9;
		color: #64748b;
		cursor: pointer;
		border: none;
		transition: all 0.15s ease;
	}

	.status-badge:hover {
		background: #e2e8f0;
	}

	.status-badge.active {
		background: #dcfce7;
		color: #16a34a;
	}

	.status-badge.active:hover {
		background: #bbf7d0;
	}

	.action-btns {
		display: flex;
		gap: 4px;
	}

	.action-btn {
		width: 32px;
		height: 32px;
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: #e2e8f0;
		color: #334155;
	}

	.action-btn.danger:hover {
		background: #fef2f2;
		color: #dc2626;
	}

	.add-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 180px;
		background: transparent;
		border: 2px dashed #cbd5e1;
		border-radius: 12px;
		color: #94a3b8;
		transition: all 0.15s ease;
	}

	.add-card:hover {
		border-color: #3b82f6;
		background: #eff6ff;
		color: #3b82f6;
	}

	.add-card span {
		font-size: 14px;
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		width: 100%;
		max-width: 480px;
		max-height: 80vh;
		background: white;
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
		display: flex;
		flex-direction: column;
	}

	.modal-sm {
		max-width: 400px;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px;
		border-bottom: 1px solid #e2e8f0;
		flex-shrink: 0;
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.btn-close {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.btn-close:hover {
		background: #f1f5f9;
	}

	.modal-body {
		padding: 20px;
		overflow-y: auto;
	}

	.form-group {
		margin-bottom: 16px;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #475569;
		margin-bottom: 6px;
	}

	.form-group.checkbox label {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}

	.form-group.checkbox input[type="checkbox"] {
		width: auto;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 10px 12px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 14px;
		color: #1e293b;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.form-group input:focus,
	.form-group select:focus {
		border-color: #3b82f6;
	}

	.form-row {
		display: flex;
		gap: 12px;
	}

	.form-row .form-group {
		flex: 1;
	}

	.form-error {
		background: #fef2f2;
		color: #dc2626;
		padding: 10px 14px;
		border-radius: 8px;
		font-size: 13px;
		margin-bottom: 16px;
	}

	.advanced-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: #3b82f6;
		padding: 8px 0;
		margin-bottom: 8px;
	}

	.advanced-toggle:hover {
		color: #2563eb;
	}

	.advanced-options {
		padding: 12px;
		background: #f8fafc;
		border-radius: 8px;
		border: 1px solid #e2e8f0;
	}

	.confirm-text {
		font-size: 14px;
		color: #475569;
		margin-bottom: 8px;
	}

	.confirm-detail {
		font-size: 13px;
		color: #64748b;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e2e8f0;
		flex-shrink: 0;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 20px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover { background: #2563eb; }
	.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

	.btn-secondary {
		padding: 10px 20px;
		background: #f1f5f9;
		border-radius: 8px;
		font-size: 14px;
		color: #475569;
		transition: background 0.15s ease;
	}

	.btn-secondary:hover { background: #e2e8f0; }

	.btn-danger {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 20px;
		background: #dc2626;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-danger:hover { background: #b91c1c; }
	.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
