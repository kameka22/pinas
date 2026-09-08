<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { api } from '$lib/stores/api';
	import type { SambaStatus, SmbGlobalConfig, ShareInfo, NfsStatus, NfsShareConfig } from '$lib/stores/api';
	import { toasts, errorMessage } from '$stores/toasts';
	import FolderPicker from '$lib/components/ui/FolderPicker.svelte';
	import { openWindow } from '$lib/stores/windows';
	import { getAppById } from '$lib/stores/desktop';

	// Tabs
	type TabId = 'smb' | 'nfs';
	let activeTab: TabId = 'smb';

	const tabs: { id: TabId; icon: string; label: string }[] = [
		{ id: 'smb', icon: 'mdi:microsoft-windows', label: 'SMB/CIFS' },
		{ id: 'nfs', icon: 'mdi:folder-network', label: 'NFS' }
	];

	// NFS state
	let nfsStatus: NfsStatus | null = null;
	let nfsExports: ShareInfo[] = [];
	let nfsLoading = false;
	let nfsBusy = false;
	let showNfsForm = false;
	let nfsForm: { name: string; path: string; clients: string; read_only: boolean; sync: boolean; squash: NfsShareConfig['squash'] } = {
		name: '', path: '', clients: '192.168.1.0/24', read_only: false, sync: true, squash: 'root_squash'
	};

	async function loadNfsData() {
		nfsLoading = true;
		try {
			const [status, shares] = await Promise.all([api.getNfsStatus(), api.getShares()]);
			nfsStatus = status;
			nfsExports = shares.filter((s) => s.share_type === 'nfs');
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
		} finally {
			nfsLoading = false;
		}
	}

	async function toggleNfs() {
		if (!nfsStatus) return;
		nfsBusy = true;
		try {
			nfsStatus = nfsStatus.enabled ? await api.disableNfs() : await api.enableNfs();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			nfsBusy = false;
		}
	}

	async function createNfsExport() {
		nfsBusy = true;
		try {
			await api.createShare({
				name: nfsForm.name.trim(),
				path: nfsForm.path.trim(),
				share_type: 'nfs',
				config: {
					clients: nfsForm.clients.split(/[\s,]+/).filter(Boolean),
					read_only: nfsForm.read_only,
					sync: nfsForm.sync,
					squash: nfsForm.squash,
					subtree_check: false
				}
			});
			showNfsForm = false;
			nfsForm = { ...nfsForm, name: '', path: '' };
			toasts.success($t.fileService.nfs.exportCreated);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		} finally {
			nfsBusy = false;
		}
	}

	async function toggleExport(share: ShareInfo) {
		try {
			await api.toggleShare(share.id, !share.enabled);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		}
	}

	async function deleteExport(share: ShareInfo) {
		try {
			await api.deleteShare(share.id);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	// SMB state
	let sambaStatus: SambaStatus | null = null;
	let smbConfig: SmbGlobalConfig = {
		workgroup: 'WORKGROUP',
		server_string: 'PiNAS',
		min_protocol: 'SMB2',
		max_protocol: 'SMB3'
	};
	let activeShares: ShareInfo[] = [];
	let smbLoading = true;
	let smbError: string | null = null;
	let toggleLoading = false;
	let configLoading = false;
	let configSaved = false;

	function selectTab(tabId: TabId) {
		activeTab = tabId;
		if (tabId === 'smb') loadSmbData();
		if (tabId === 'nfs') loadNfsData();
	}

	onMount(() => {
		loadSmbData();
	});

	async function loadSmbData() {
		smbLoading = true;
		smbError = null;
		try {
			const [status, config, shares] = await Promise.all([
				api.getSambaStatus(),
				api.getSmbConfig(),
				api.getShares()
			]);
			sambaStatus = status;
			smbConfig = config;
			activeShares = shares.filter(s => s.share_type === 'smb' && s.enabled);
		} catch (e: any) {
			smbError = e.message || 'Failed to load SMB data';
		} finally {
			smbLoading = false;
		}
	}

	async function toggleSamba() {
		if (!sambaStatus) return;
		toggleLoading = true;
		try {
			if (sambaStatus.enabled) {
				await api.disableSamba();
			} else {
				await api.enableSamba();
			}
			await loadSmbData();
		} catch (e: any) {
			smbError = e.message || 'Failed to toggle Samba';
		} finally {
			toggleLoading = false;
		}
	}

	async function applyConfig() {
		configLoading = true;
		configSaved = false;
		try {
			await api.updateSmbConfig(smbConfig);
			configSaved = true;
			setTimeout(() => { configSaved = false; }, 3000);
		} catch (e: any) {
			smbError = e.message || ($t.fileService?.smb?.applyError || 'Failed to update SMB configuration');
		} finally {
			configLoading = false;
		}
	}

	function openShareManager() {
		const app = getAppById('shares');
		if (app) {
			openWindow({
				id: app.id,
				title: app.label,
				icon: app.icon,
				component: app.component,
				x: 200 + Math.random() * 100,
				y: 100 + Math.random() * 50,
				width: 900,
				height: 600,
				gradient: app.gradient
			});
		}
	}
</script>

<div class="file-service">
	<!-- Tabs Header -->
	<div class="tabs-header">
		{#each tabs as tab}
			<button
				class="tab-btn"
				class:active={activeTab === tab.id}
				on:click={() => selectTab(tab.id)}
			>
				<Icon icon={tab.icon} class="w-5 h-5" />
				<span>{tab.label}</span>
			</button>
		{/each}
	</div>

	<!-- Tab Content -->
	<div class="tab-content">
		{#if activeTab === 'smb'}
			<!-- SMB Tab -->
			<div class="tab-panel">
				<div class="panel-header">
					<Icon icon="mdi:microsoft-windows" class="w-6 h-6" />
					<div class="panel-info">
						<h2>{$t.fileService?.smb?.title || 'SMB/CIFS'}</h2>
						<p>{$t.fileService?.smb?.description || 'Windows file sharing protocol'}</p>
					</div>
				</div>

				{#if smbLoading}
					<div class="placeholder-content">
						<Icon icon="mdi:loading" class="w-8 h-8 spin" />
					</div>
				{:else if smbError}
					<div class="error-banner">
						<Icon icon="mdi:alert-circle" class="w-5 h-5" />
						<span>{smbError}</span>
						<button on:click={loadSmbData}>Retry</button>
					</div>
				{:else if sambaStatus}
					<!-- Status Section -->
					<div class="section">
						<div class="status-row">
							<div class="status-info">
								<span class="status-dot" class:running={sambaStatus.running}></span>
								<span class="status-label">
									{$t.fileService?.smb?.status || 'Status'}:
									<strong>{sambaStatus.running ? ($t.fileService?.smb?.running || 'Running') : ($t.fileService?.smb?.stopped || 'Stopped')}</strong>
								</span>
							</div>
							<button
								class="toggle-btn"
								class:enabled={sambaStatus.enabled}
								on:click={toggleSamba}
								disabled={toggleLoading}
							>
								{#if toggleLoading}
									<Icon icon="mdi:loading" class="w-4 h-4 spin" />
								{/if}
								{sambaStatus.enabled ? ($t.fileService?.smb?.disable || 'Disable SMB') : ($t.fileService?.smb?.enable || 'Enable SMB')}
							</button>
						</div>

						{#if sambaStatus.version}
							<div class="info-row">
								<span class="info-label">{$t.fileService?.smb?.version || 'Version'}</span>
								<span class="info-value">{sambaStatus.version}</span>
							</div>
						{/if}
						<div class="info-row">
							<span class="info-label">{$t.fileService?.smb?.activeShares || 'Active Shares'}</span>
							<span class="info-value">{sambaStatus.share_count}</span>
						</div>
						<div class="info-row">
							<span class="info-label">{$t.fileService?.smb?.connectedUsers || 'Connected Users'}</span>
							<span class="info-value">{sambaStatus.connected_users}</span>
						</div>
					</div>

					<!-- Global Config (visible when enabled) -->
					{#if sambaStatus.enabled}
						<div class="section">
							<h3>{$t.fileService?.smb?.globalConfig || 'Global Configuration'}</h3>

							<div class="form-group">
								<label>{$t.fileService?.smb?.workgroup || 'Workgroup'}</label>
								<input type="text" bind:value={smbConfig.workgroup} />
							</div>
							<div class="form-group">
								<label>{$t.fileService?.smb?.serverDescription || 'Server Description'}</label>
								<input type="text" bind:value={smbConfig.server_string} />
							</div>
							<div class="form-row">
								<div class="form-group">
									<label>{$t.fileService?.smb?.minProtocol || 'Min Protocol'}</label>
									<select bind:value={smbConfig.min_protocol}>
										<option value="SMB2">SMB2</option>
										<option value="SMB2_10">SMB2_10</option>
										<option value="SMB3">SMB3</option>
									</select>
								</div>
								<div class="form-group">
									<label>{$t.fileService?.smb?.maxProtocol || 'Max Protocol'}</label>
									<select bind:value={smbConfig.max_protocol}>
										<option value="SMB2">SMB2</option>
										<option value="SMB3">SMB3</option>
										<option value="SMB3_11">SMB3_11</option>
									</select>
								</div>
							</div>

							<div class="form-actions">
								<button class="btn-primary" on:click={applyConfig} disabled={configLoading}>
									{#if configLoading}
										<Icon icon="mdi:loading" class="w-4 h-4 spin" />
									{/if}
									{$t.fileService?.smb?.apply || 'Apply'}
								</button>
								{#if configSaved}
									<span class="save-msg">
										<Icon icon="mdi:check" class="w-4 h-4" />
										{$t.fileService?.smb?.applySuccess || 'Configuration updated'}
									</span>
								{/if}
							</div>
						</div>

						<!-- Active Shares Summary -->
						<div class="section">
							<div class="section-header">
								<h3>{$t.fileService?.smb?.activeShares || 'Active Shares'}</h3>
								<button class="btn-link" on:click={openShareManager}>
									{$t.fileService?.smb?.manageShares || 'Manage Shares'}
									<Icon icon="mdi:arrow-right" class="w-4 h-4" />
								</button>
							</div>

							{#if activeShares.length === 0}
								<p class="no-shares">{$t.fileService.noActiveShares}</p>
							{:else}
								<div class="shares-list">
									{#each activeShares as share}
										<div class="share-row">
											<Icon icon="mdi:folder-open" class="w-5 h-5" />
											<div class="share-details">
												<span class="share-name">{share.name}</span>
												<span class="share-path">{share.path}</span>
											</div>
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				{/if}
			</div>
		{:else if activeTab === 'nfs'}
			<div class="tab-panel">
				<div class="panel-header">
					<Icon icon="mdi:folder-network" class="w-6 h-6" />
					<div class="panel-info">
						<h2>NFS</h2>
						<p>{$t.fileService.nfs.description}</p>
					</div>
				</div>

				{#if nfsLoading && !nfsStatus}
					<p class="nfs-hint">{$t.common.loading}</p>
				{:else if nfsStatus}
					<div class="nfs-status-card">
						<div class="nfs-status-main">
							<span class="nfs-badge" class:on={nfsStatus.running}>{nfsStatus.running ? $t.fileService.smb.running : $t.fileService.smb.stopped}</span>
							<span>{$t.fileService.nfs.exportsCount.replace('{n}', String(nfsStatus.export_count))}</span>
						</div>
						<label class="toggle">
							<input type="checkbox" checked={nfsStatus.enabled} disabled={nfsBusy || !nfsStatus.nfsd_available} on:change={toggleNfs} />
							<span class="toggle-slider"></span>
						</label>
					</div>
					{#if !nfsStatus.nfsd_available}
						<p class="nfs-warning"><Icon icon="mdi:alert-outline" class="w-4 h-4" />{$t.fileService.nfs.notAvailable}</p>
					{/if}

					<div class="nfs-exports">
						<div class="nfs-exports-header">
							<h3>{$t.fileService.nfs.exports}</h3>
							<button class="btn-primary" on:click={() => (showNfsForm = !showNfsForm)}>
								<Icon icon={showNfsForm ? 'mdi:close' : 'mdi:plus'} class="w-4 h-4" />
								{showNfsForm ? $t.common.cancel : $t.fileService.nfs.addExport}
							</button>
						</div>

						{#if showNfsForm}
							<div class="nfs-form">
								<label>{$t.fileService.nfs.name}<input class="nfs-input" bind:value={nfsForm.name} placeholder="media" /></label>
								<FolderPicker bind:value={nfsForm.path} label={$t.fileService.nfs.path} placeholder="/storage/shares/media" />
								<label>{$t.fileService.nfs.clients}<input class="nfs-input" bind:value={nfsForm.clients} placeholder="192.168.1.0/24 10.0.0.5 *" /></label>
								<span class="nfs-hint">{$t.fileService.nfs.clientsHint}</span>
								<div class="nfs-form-row">
									<label class="nfs-check"><input type="checkbox" bind:checked={nfsForm.read_only} />{$t.fileService.nfs.readOnly}</label>
									<label class="nfs-check"><input type="checkbox" bind:checked={nfsForm.sync} />{$t.fileService.nfs.sync}</label>
									<label>{$t.fileService.nfs.squash}
										<select class="nfs-input" bind:value={nfsForm.squash}>
											<option value="root_squash">root_squash</option>
											<option value="no_root_squash">no_root_squash</option>
											<option value="all_squash">all_squash</option>
										</select>
									</label>
								</div>
								<div class="nfs-form-actions">
									<button class="btn-primary" on:click={createNfsExport} disabled={nfsBusy || !nfsForm.name.trim() || !nfsForm.path.trim() || !nfsForm.clients.trim()}>{$t.common.create}</button>
								</div>
							</div>
						{/if}

						{#if nfsExports.length === 0}
							<p class="nfs-hint">{$t.fileService.nfs.noExports}</p>
						{:else}
							<div class="nfs-list">
								{#each nfsExports as share (share.id)}
									<div class="nfs-row" class:disabled={!share.enabled}>
										<Icon icon="mdi:folder-network-outline" class="w-5 h-5" />
										<div class="nfs-row-main">
											<span class="nfs-row-name">{share.name}</span>
											<span class="nfs-row-path mono">{share.path}</span>
											<span class="nfs-row-opts">{(share.nfs?.clients || ['*']).join(', ')} · {share.nfs?.read_only ? 'ro' : 'rw'} · {share.nfs?.squash}</span>
										</div>
										<label class="toggle"><input type="checkbox" checked={share.enabled} on:change={() => toggleExport(share)} /><span class="toggle-slider"></span></label>
										<button class="nfs-icon-btn" on:click={() => deleteExport(share)} title={$t.common.delete}><Icon icon="mdi:delete-outline" class="w-4 h-4" /></button>
									</div>
								{/each}
							</div>
						{/if}
					</div>
					<p class="nfs-hint">{$t.fileService.nfs.mountHint} <span class="mono">mount -t nfs {window.location.hostname}:/storage/shares/&lt;name&gt; /mnt</span></p>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.file-service {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	/* Tabs */
	.tabs-header {
		display: flex;
		border-bottom: 1px solid #e5e7eb;
		background: #fafafa;
	}

	.tab-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px 20px;
		font-size: 14px;
		font-weight: 500;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		transition: all 0.15s ease;
	}

	.tab-btn:hover {
		color: #374151;
		background: rgba(0, 0, 0, 0.02);
	}

	.tab-btn.active {
		color: #3b82f6;
		border-bottom-color: #3b82f6;
		background: white;
	}

	/* Tab Content */
	.tab-content {
		flex: 1;
		overflow-y: auto;
	}

	.tab-panel {
		padding: 24px;
	}

	.panel-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.panel-header > :global(svg) {
		color: #6b7280;
	}

	.panel-info h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 4px;
	}

	.panel-info p {
		font-size: 13px;
		color: #6b7280;
	}

	.placeholder-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 80px 20px;
		color: #9ca3af;
		background: #f9fafb;
		border: 1px dashed #e5e7eb;
		border-radius: 12px;
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Sections */
	.section {
		margin-bottom: 24px;
		padding: 20px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 12px;
	}

	.section h3 {
		font-size: 15px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 16px;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	.section-header h3 {
		margin-bottom: 0;
	}

	/* Status */
	.status-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	.status-info {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.status-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: #dc2626;
	}

	.status-dot.running {
		background: #16a34a;
	}

	.status-label {
		font-size: 14px;
		color: #475569;
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		background: #3b82f6;
		color: white;
		transition: all 0.15s ease;
	}

	.toggle-btn.enabled {
		background: #f1f5f9;
		color: #64748b;
	}

	.toggle-btn:hover {
		opacity: 0.9;
	}

	.toggle-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.info-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
		border-bottom: 1px solid #e5e7eb;
	}

	.info-row:last-child {
		border-bottom: none;
	}

	.info-label {
		font-size: 13px;
		color: #6b7280;
	}

	.info-value {
		font-size: 13px;
		font-weight: 500;
		color: #1f2937;
	}

	/* Error */
	.error-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		background: #fef2f2;
		color: #dc2626;
		border-radius: 8px;
		font-size: 13px;
	}

	.error-banner button {
		margin-left: auto;
		font-size: 13px;
		color: #3b82f6;
		font-weight: 500;
	}

	/* Forms */
	.form-group {
		margin-bottom: 12px;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #475569;
		margin-bottom: 4px;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 8px 12px;
		background: white;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 14px;
		color: #1f2937;
		outline: none;
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

	.form-actions {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 16px;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover { background: #2563eb; }
	.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

	.save-msg {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		color: #16a34a;
	}

	.btn-link {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		font-weight: 500;
		color: #3b82f6;
	}

	.btn-link:hover {
		color: #2563eb;
	}

	/* Shares List */
	.no-shares {
		font-size: 13px;
		color: #9ca3af;
		text-align: center;
		padding: 16px;
	}

	.shares-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.share-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
	}

	.share-row > :global(svg) {
		color: #3b82f6;
		flex-shrink: 0;
	}

	.share-details {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.share-name {
		font-size: 14px;
		font-weight: 500;
		color: #1f2937;
	}

	.share-path {
		font-size: 12px;
		color: #6b7280;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.nfs-status-card { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.75rem 1rem; background: #f8fafc; border-radius: 0.75rem; margin: 1rem 0; font-size: 0.8125rem; }
	.nfs-status-main { display: flex; align-items: center; gap: 0.75rem; }
	.nfs-badge { font-size: 0.6875rem; padding: 0.125rem 0.5rem; border-radius: 999px; background: #fee2e2; color: #b91c1c; }
	.nfs-badge.on { background: #dcfce7; color: #15803d; }
	.nfs-warning { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #92400e; background: #fffbeb; padding: 0.5rem 0.75rem; border-radius: 0.5rem; }
	.nfs-exports-header { display: flex; align-items: center; justify-content: space-between; margin-top: 1rem; }
	.nfs-exports-header h3 { font-size: 0.9375rem; font-weight: 600; margin: 0; }
	.nfs-form { display: flex; flex-direction: column; gap: 0.625rem; padding: 1rem; margin-top: 0.75rem; border: 1px dashed #cbd5e1; border-radius: 0.75rem; font-size: 0.8125rem; }
	.nfs-form label { display: flex; flex-direction: column; gap: 0.25rem; color: #334155; }
	.nfs-form-row { display: flex; gap: 1.25rem; align-items: flex-end; flex-wrap: wrap; }
	.nfs-check { flex-direction: row !important; align-items: center; gap: 0.5rem !important; }
	.nfs-input { border: 1px solid #cbd5e1; border-radius: 0.5rem; padding: 0.375rem 0.625rem; font-size: 0.8125rem; background: white; }
	.nfs-form-actions { display: flex; justify-content: flex-end; }
	.nfs-hint { font-size: 0.75rem; color: #64748b; margin: 0.5rem 0; }
	.nfs-list { display: flex; flex-direction: column; margin-top: 0.75rem; }
	.nfs-row { display: flex; align-items: center; gap: 0.75rem; padding: 0.625rem 0; border-top: 1px solid #f1f5f9; font-size: 0.8125rem; }
	.nfs-row.disabled { opacity: 0.55; }
	.nfs-row-main { flex: 1; display: flex; flex-direction: column; gap: 0.125rem; }
	.nfs-row-name { font-weight: 500; }
	.nfs-row-path, .nfs-row-opts { font-size: 0.75rem; color: #64748b; }
	.nfs-icon-btn { padding: 0.25rem; border-radius: 0.375rem; color: #64748b; }
	.nfs-icon-btn:hover { color: #dc2626; background: #fef2f2; }
	.mono { font-family: ui-monospace, monospace; }
	.btn-primary { display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.4rem 0.875rem; border-radius: 0.5rem; font-size: 0.8125rem; font-weight: 500; background: #3b82f6; color: white; }
	.btn-primary:disabled { opacity: 0.5; }
</style>
