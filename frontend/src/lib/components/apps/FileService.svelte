<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api, type SambaStatus, type SmbGlobalConfig, type ShareInfo } from '$lib/stores/api';
	import { openWindow } from '$lib/stores/windows';

	// Tabs
	type TabId = 'smb' | 'nfs' | 'ftp';
	let activeTab: TabId = 'smb';

	const tabs: { id: TabId; icon: string; label: string }[] = [
		{ id: 'smb', icon: 'mdi:microsoft-windows', label: 'SMB/CIFS' },
		{ id: 'nfs', icon: 'mdi:folder-network', label: 'NFS' },
		{ id: 'ftp', icon: 'mdi:folder-upload', label: 'FTP' }
	];

	function selectTab(tabId: TabId) {
		activeTab = tabId;
	}

	// SMB state
	let sambaStatus: SambaStatus | null = null;
	let smbLoading = false;
	let smbError: string | null = null;
	let smbToggling = false;

	// SMB config
	let smbConfig: SmbGlobalConfig = {
		workgroup: 'WORKGROUP',
		server_string: 'PiNAS',
		min_protocol: 'SMB2',
		max_protocol: 'SMB3'
	};
	let configLoading = false;
	let configSaving = false;
	let configSaved = false;

	// Active shares
	let shares: ShareInfo[] = [];
	let sharesLoading = false;

	onMount(() => {
		loadSambaStatus();
	});

	async function loadSambaStatus() {
		smbLoading = true;
		smbError = null;
		try {
			sambaStatus = await api.getSambaStatus();
			if (sambaStatus.enabled) {
				await Promise.all([loadSmbConfig(), loadShares()]);
			}
		} catch (e) {
			smbError = e instanceof Error ? e.message : 'Failed to load Samba status';
		} finally {
			smbLoading = false;
		}
	}

	async function toggleSamba() {
		if (!sambaStatus || smbToggling) return;

		smbToggling = true;
		smbError = null;

		try {
			if (sambaStatus.enabled) {
				await api.disableSamba();
			} else {
				await api.enableSamba();
			}
			await loadSambaStatus();
		} catch (e) {
			smbError = e instanceof Error ? e.message : 'Failed to toggle Samba';
		} finally {
			smbToggling = false;
		}
	}

	async function loadSmbConfig() {
		configLoading = true;
		try {
			smbConfig = await api.getSmbConfig();
		} catch (e) {
			console.error('Failed to load SMB config:', e);
		} finally {
			configLoading = false;
		}
	}

	async function saveSmbConfig() {
		configSaving = true;
		configSaved = false;
		try {
			await api.updateSmbConfig(smbConfig);
			configSaved = true;
			setTimeout(() => { configSaved = false; }, 3000);
		} catch (e) {
			smbError = e instanceof Error ? e.message : 'Failed to save SMB config';
		} finally {
			configSaving = false;
		}
	}

	async function loadShares() {
		sharesLoading = true;
		try {
			const all = await api.getShares();
			shares = all.filter(s => s.enabled);
		} catch (e) {
			console.error('Failed to load shares:', e);
			shares = [];
		} finally {
			sharesLoading = false;
		}
	}

	function openShareManager() {
		openWindow({
			id: 'share-manager',
			title: 'Shared Folders',
			icon: 'mdi:folder-network',
			component: 'ShareManager',
			x: 150,
			y: 100,
			width: 900,
			height: 600
		});
	}

	const protocolOptions = [
		{ value: 'SMB2', label: 'SMB2' },
		{ value: 'SMB2_10', label: 'SMB2.1' },
		{ value: 'SMB3', label: 'SMB3' },
		{ value: 'SMB3_02', label: 'SMB3.0.2' },
		{ value: 'SMB3_11', label: 'SMB3.1.1' }
	];
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
						<h2>SMB/CIFS</h2>
						<p>{$t.fileService?.smb?.description || 'Windows file sharing protocol'}</p>
					</div>
				</div>

				{#if smbLoading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
						<span>{$t.common.loading}</span>
					</div>
				{:else if smbError}
					<div class="error-state">
						<Icon icon="mdi:alert-circle" class="w-5 h-5" />
						<span>{smbError}</span>
						<button class="retry-btn" on:click={loadSambaStatus}>
							{$t.common.retry || 'Retry'}
						</button>
					</div>
				{:else if sambaStatus}
					<div class="service-content">
						<!-- Enable toggle -->
						<div class="service-row">
							<div class="service-label">
								<span class="label">{$t.fileService?.smb?.enable || 'Enable SMB/CIFS'}</span>
								<span class="sublabel">{$t.fileService?.smb?.enableHint || 'Start Samba service to share folders on the network'}</span>
							</div>
							<button
								class="toggle-switch"
								class:active={sambaStatus.enabled}
								class:loading={smbToggling}
								on:click={toggleSamba}
								disabled={smbToggling}
							>
								<span class="toggle-knob"></span>
							</button>
						</div>

						{#if sambaStatus.enabled}
							<!-- Status row -->
							<div class="service-row">
								<div class="service-label">
									<span class="label">{$t.fileService?.smb?.status || 'Status'}</span>
								</div>
								<div class="status-badge" class:running={sambaStatus.running}>
									<Icon icon={sambaStatus.running ? 'mdi:check-circle' : 'mdi:close-circle'} class="w-4 h-4" />
									{sambaStatus.running
										? ($t.fileService?.smb?.running || 'Running')
										: ($t.fileService?.smb?.stopped || 'Stopped')}
								</div>
							</div>

							<!-- Info row: connected users + version -->
							<div class="info-row">
								{#if sambaStatus.version}
									<div class="info-chip">
										<Icon icon="mdi:information-outline" class="w-4 h-4" />
										<span>{$t.fileService?.smb?.version || 'Version'}: {sambaStatus.version}</span>
									</div>
								{/if}
								<div class="info-chip">
									<Icon icon="mdi:account-multiple" class="w-4 h-4" />
									<span>{$t.fileService?.smb?.connectedUsers || 'Connected Users'}: {sambaStatus.connected_users}</span>
								</div>
							</div>

							<!-- Global Settings Section -->
							<div class="subsection">
								<div class="subsection-header">
									<h3>{$t.fileService?.smb?.globalSettings || 'Global Settings'}</h3>
								</div>

								{#if configLoading}
									<div class="loading-state small">
										<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
									</div>
								{:else}
									<div class="config-form">
										<div class="form-row">
											<label for="smb-workgroup">{$t.fileService?.smb?.workgroup || 'Workgroup'}</label>
											<input id="smb-workgroup" type="text" bind:value={smbConfig.workgroup} />
										</div>
										<div class="form-row">
											<label for="smb-description">{$t.fileService?.smb?.serverDescription || 'Server Description'}</label>
											<input id="smb-description" type="text" bind:value={smbConfig.server_string} />
										</div>
										<div class="form-row">
											<label for="smb-min-protocol">{$t.fileService?.smb?.minProtocol || 'Minimum Protocol'}</label>
											<select id="smb-min-protocol" bind:value={smbConfig.min_protocol}>
												{#each protocolOptions as opt}
													<option value={opt.value}>{opt.label}</option>
												{/each}
											</select>
										</div>
										<div class="form-row">
											<label for="smb-max-protocol">{$t.fileService?.smb?.maxProtocol || 'Maximum Protocol'}</label>
											<select id="smb-max-protocol" bind:value={smbConfig.max_protocol}>
												{#each protocolOptions as opt}
													<option value={opt.value}>{opt.label}</option>
												{/each}
											</select>
										</div>
										<div class="form-actions">
											{#if configSaved}
												<span class="saved-feedback">
													<Icon icon="mdi:check" class="w-4 h-4" />
													{$t.fileService?.smb?.configUpdated || 'Configuration updated'}
												</span>
											{/if}
											<button class="btn-primary" on:click={saveSmbConfig} disabled={configSaving}>
												{#if configSaving}
													<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
													{$t.fileService?.smb?.applying || 'Applying...'}
												{:else}
													<Icon icon="mdi:check" class="w-4 h-4" />
													{$t.fileService?.smb?.apply || 'Apply'}
												{/if}
											</button>
										</div>
									</div>
								{/if}
							</div>

							<!-- Active Shares Section -->
							<div class="subsection">
								<div class="subsection-header">
									<h3>{$t.fileService?.smb?.activeShares || 'Active Shares'}</h3>
									<button class="btn-small primary" on:click={openShareManager}>
										<Icon icon="mdi:folder-cog" class="w-4 h-4" />
										{$t.fileService?.smb?.manageShares || 'Manage Shared Folders'}
									</button>
								</div>

								{#if sharesLoading}
									<div class="loading-state small">
										<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
									</div>
								{:else if shares.length === 0}
									<div class="empty-hint">
										<Icon icon="mdi:folder-off" class="w-8 h-8 text-slate-300" />
										<p>{$t.shareManager?.messages?.noShares || 'No shared folders configured'}</p>
										<span class="hint-text">{$t.shareManager?.messages?.noSharesHint || 'Create a shared folder to start sharing files'}</span>
									</div>
								{:else}
									{#each shares as share}
										<div class="share-card">
											<div class="share-info">
												<Icon icon="mdi:folder-network" class="w-5 h-5 text-blue-500" />
												<div class="share-details">
													<span class="share-name">{share.name}</span>
													<span class="share-path">{share.path}</span>
												</div>
											</div>
											<div class="share-badges">
												{#if share.config?.guest_ok}
													<span class="badge guest">
														<Icon icon="mdi:account-off" class="w-3 h-3" />
														Guest
													</span>
												{/if}
												{#if share.config?.read_only}
													<span class="badge readonly">
														<Icon icon="mdi:lock" class="w-3 h-3" />
														RO
													</span>
												{/if}
											</div>
										</div>
									{/each}
								{/if}
							</div>
						{/if}
					</div>
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

	/* Service content */
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

	/* Info row */
	.info-row {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
	}

	.info-chip {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		background: #f3f4f6;
		border-radius: 8px;
		font-size: 12px;
		color: #6b7280;
	}

	/* Subsections */
	.subsection {
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		padding: 16px;
	}

	.subsection-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}

	.subsection-header h3 {
		font-size: 14px;
		font-weight: 600;
		color: #374151;
	}

	/* Config form */
	.config-form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.form-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.form-row label {
		flex: 0 0 160px;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
	}

	.form-row input,
	.form-row select {
		flex: 1;
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		background: white;
	}

	.form-row input:focus,
	.form-row select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-actions {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
		padding-top: 8px;
	}

	.saved-feedback {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: #16a34a;
	}

	/* Share cards */
	.share-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		margin-bottom: 8px;
	}

	.share-card:last-of-type {
		margin-bottom: 0;
	}

	.share-info {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.share-details {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.share-name {
		font-size: 14px;
		font-weight: 500;
		color: #1f2937;
	}

	.share-path {
		font-size: 12px;
		color: #9ca3af;
		font-family: monospace;
	}

	.share-badges {
		display: flex;
		gap: 6px;
	}

	.badge {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 500;
	}

	.badge.guest {
		background: #fef3c7;
		color: #b45309;
	}

	.badge.readonly {
		background: #e0e7ff;
		color: #4338ca;
	}

	/* Buttons */
	.btn-small {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		background: white;
		font-size: 13px;
		color: #374151;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-small:hover:not(:disabled) {
		background: #f9fafb;
	}

	.btn-small:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-small.primary {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.btn-small.primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		border: none;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Empty and loading states */
	.empty-hint {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 24px;
		text-align: center;
	}

	.empty-hint p {
		font-size: 14px;
		color: #6b7280;
	}

	.hint-text {
		font-size: 12px;
		color: #9ca3af;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px 20px;
		color: #9ca3af;
	}

	.loading-state.small {
		padding: 20px;
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
		cursor: pointer;
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
</style>
