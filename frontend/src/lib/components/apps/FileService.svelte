<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { api } from '$lib/stores/api';
	import type { SambaStatus, SmbGlobalConfig, ShareInfo } from '$lib/stores/api';
	import { openWindow } from '$lib/stores/windows';
	import { getAppById } from '$lib/stores/desktop';

	// Tabs
	type TabId = 'smb' | 'nfs' | 'ftp';
	let activeTab: TabId = 'smb';

	const tabs: { id: TabId; icon: string; label: string }[] = [
		{ id: 'smb', icon: 'mdi:microsoft-windows', label: 'SMB/CIFS' },
		{ id: 'nfs', icon: 'mdi:folder-network', label: 'NFS' },
		{ id: 'ftp', icon: 'mdi:folder-upload', label: 'FTP' }
	];

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
		if (tabId === 'smb') {
			loadSmbData();
		}
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
								<p class="no-shares">No active SMB shares</p>
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
			<!-- NFS Tab -->
			<div class="tab-panel">
				<div class="panel-header">
					<Icon icon="mdi:folder-network" class="w-6 h-6" />
					<div class="panel-info">
						<h2>NFS</h2>
						<p>{$t.fileService?.nfs?.description || 'Network File System for Unix/Linux'}</p>
					</div>
				</div>
				<div class="placeholder-content">
					<Icon icon="mdi:cog" class="w-12 h-12" />
					<span>{$t.controlPanel.underDevelopment}</span>
				</div>
			</div>
		{:else if activeTab === 'ftp'}
			<!-- FTP Tab -->
			<div class="tab-panel">
				<div class="panel-header">
					<Icon icon="mdi:folder-upload" class="w-6 h-6" />
					<div class="panel-info">
						<h2>FTP</h2>
						<p>{$t.fileService?.ftp?.description || 'File Transfer Protocol'}</p>
					</div>
				</div>
				<div class="placeholder-content">
					<Icon icon="mdi:cog" class="w-12 h-12" />
					<span>{$t.controlPanel.underDevelopment}</span>
				</div>
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
</style>
