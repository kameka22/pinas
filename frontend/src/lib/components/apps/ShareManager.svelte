<script lang="ts">
	import Icon from '@iconify/svelte';

	interface Share {
		id: string;
		name: string;
		path: string;
		protocol: 'smb' | 'nfs' | 'ftp';
		enabled: boolean;
		users: string[];
	}

	const shares: Share[] = [
		{ id: '1', name: 'Media', path: '/srv/data/media', protocol: 'smb', enabled: true, users: ['john', 'jane'] },
		{ id: '2', name: 'Documents', path: '/srv/data/documents', protocol: 'smb', enabled: true, users: ['john'] },
		{ id: '3', name: 'Backups', path: '/srv/data/backups', protocol: 'nfs', enabled: true, users: ['admin'] },
		{ id: '4', name: 'Public', path: '/srv/data/public', protocol: 'ftp', enabled: false, users: [] }
	];

	let showCreateModal = false;

	function getProtocolIcon(protocol: string): string {
		switch (protocol) {
			case 'smb': return 'mdi:microsoft-windows';
			case 'nfs': return 'mdi:folder-network';
			case 'ftp': return 'mdi:folder-upload';
			default: return 'mdi:folder';
		}
	}

	function getProtocolLabel(protocol: string): string {
		switch (protocol) {
			case 'smb': return 'SMB/CIFS';
			case 'nfs': return 'NFS';
			case 'ftp': return 'FTP';
			default: return protocol;
		}
	}
</script>

<div class="share-manager">
	<header class="header">
		<h1>Shared Folders</h1>
		<button class="btn-primary" on:click={() => showCreateModal = true}>
			<Icon icon="mdi:plus" class="w-4 h-4" />
			Create Share
		</button>
	</header>

	<div class="content">
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
						</div>
						<button class="btn-menu">
							<Icon icon="mdi:dots-vertical" class="w-5 h-5" />
						</button>
					</div>

					<div class="share-meta">
						<div class="meta-item">
							<Icon icon={getProtocolIcon(share.protocol)} class="w-4 h-4" />
							<span>{getProtocolLabel(share.protocol)}</span>
						</div>
						<div class="meta-item">
							<Icon icon="mdi:account-group" class="w-4 h-4" />
							<span>{share.users.length} users</span>
						</div>
					</div>

					<div class="share-footer">
						<span class="status-badge" class:active={share.enabled}>
							{share.enabled ? 'Active' : 'Disabled'}
						</span>
						<div class="action-btns">
							<button class="action-btn" title="Edit">
								<Icon icon="mdi:pencil" class="w-4 h-4" />
							</button>
							<button class="action-btn" title="Permissions">
								<Icon icon="mdi:shield-account" class="w-4 h-4" />
							</button>
							<button class="action-btn danger" title="Delete">
								<Icon icon="mdi:delete" class="w-4 h-4" />
							</button>
						</div>
					</div>
				</div>
			{/each}

			<!-- Add New Card -->
			<button class="add-card" on:click={() => showCreateModal = true}>
				<Icon icon="mdi:plus-circle-outline" class="w-8 h-8" />
				<span>Create New Share</span>
			</button>
		</div>
	</div>
</div>

{#if showCreateModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={() => showCreateModal = false}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>Create New Share</h2>
				<button class="btn-close" on:click={() => showCreateModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				<div class="form-group">
					<label>Share Name</label>
					<input type="text" placeholder="Enter share name" />
				</div>
				<div class="form-group">
					<label>Path</label>
					<div class="input-with-btn">
						<input type="text" placeholder="/srv/data/" />
						<button class="btn-browse">
							<Icon icon="mdi:folder-search" class="w-4 h-4" />
						</button>
					</div>
				</div>
				<div class="form-group">
					<label>Protocol</label>
					<select>
						<option value="smb">SMB/CIFS</option>
						<option value="nfs">NFS</option>
						<option value="ftp">FTP</option>
					</select>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showCreateModal = false}>Cancel</button>
				<button class="btn-primary">Create Share</button>
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

	.btn-menu {
		padding: 4px;
		border-radius: 6px;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.btn-menu:hover {
		background: #e2e8f0;
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
	}

	.status-badge.active {
		background: #dcfce7;
		color: #16a34a;
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
		max-width: 440px;
		background: white;
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px;
		border-bottom: 1px solid #e2e8f0;
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

	.input-with-btn {
		display: flex;
		gap: 8px;
	}

	.input-with-btn input {
		flex: 1;
	}

	.btn-browse {
		padding: 10px 12px;
		background: #f1f5f9;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.btn-browse:hover {
		background: #e2e8f0;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e2e8f0;
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

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-secondary {
		padding: 10px 20px;
		background: #f1f5f9;
		border-radius: 8px;
		font-size: 14px;
		color: #475569;
		transition: background 0.15s ease;
	}

	.btn-secondary:hover {
		background: #e2e8f0;
	}
</style>
