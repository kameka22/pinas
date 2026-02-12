<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

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
				<div class="placeholder-content">
					<Icon icon="mdi:cog" class="w-12 h-12" />
					<span>{$t.controlPanel.underDevelopment}</span>
				</div>
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

</style>
