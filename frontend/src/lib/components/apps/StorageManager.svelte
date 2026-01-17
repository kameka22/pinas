<script lang="ts">
	import Icon from '@iconify/svelte';

	interface Disk {
		id: string;
		name: string;
		device: string;
		size: string;
		used: string;
		health: 'healthy' | 'warning' | 'critical';
		temperature: number;
		type: 'HDD' | 'SSD' | 'NVMe';
	}

	const disks: Disk[] = [
		{ id: '1', name: 'System Drive', device: '/dev/sda', size: '256 GB', used: '45 GB', health: 'healthy', temperature: 38, type: 'SSD' },
		{ id: '2', name: 'Data Drive 1', device: '/dev/sdb', size: '2 TB', used: '1.2 TB', health: 'healthy', temperature: 42, type: 'HDD' },
		{ id: '3', name: 'Data Drive 2', device: '/dev/sdc', size: '2 TB', used: '890 GB', health: 'warning', temperature: 48, type: 'HDD' }
	];

	let selectedDisk: Disk | null = null;
	let activeTab: 'disks' | 'pools' | 'filesystems' = 'disks';

	function getHealthColor(health: string): string {
		switch (health) {
			case 'healthy': return 'bg-green-100 text-green-600';
			case 'warning': return 'bg-amber-100 text-amber-600';
			case 'critical': return 'bg-red-100 text-red-600';
			default: return 'bg-slate-100 text-slate-600';
		}
	}

	function getTempColor(temp: number): string {
		if (temp < 40) return 'text-green-600';
		if (temp < 50) return 'text-amber-600';
		return 'text-red-600';
	}
</script>

<div class="storage-manager">
	<!-- Header -->
	<header class="header">
		<div class="header-top">
			<h1>Storage Manager</h1>
			<button class="btn-primary">
				<Icon icon="mdi:plus" class="w-4 h-4" />
				Create Pool
			</button>
		</div>

		<!-- Tabs -->
		<nav class="tabs">
			<button class="tab" class:active={activeTab === 'disks'} on:click={() => activeTab = 'disks'}>
				<Icon icon="mdi:harddisk" class="w-4 h-4" />
				Disks
			</button>
			<button class="tab" class:active={activeTab === 'pools'} on:click={() => activeTab = 'pools'}>
				<Icon icon="mdi:database" class="w-4 h-4" />
				Pools
			</button>
			<button class="tab" class:active={activeTab === 'filesystems'} on:click={() => activeTab = 'filesystems'}>
				<Icon icon="mdi:folder-multiple" class="w-4 h-4" />
				Filesystems
			</button>
		</nav>
	</header>

	<!-- Content -->
	<div class="content">
		<!-- Disk List -->
		<aside class="sidebar">
			{#each disks as disk}
				<button
					class="disk-item"
					class:selected={selectedDisk?.id === disk.id}
					on:click={() => selectedDisk = disk}
				>
					<div class="disk-icon">
						<Icon icon={disk.type === 'SSD' || disk.type === 'NVMe' ? 'mdi:chip' : 'mdi:harddisk'} class="w-5 h-5 text-slate-500" />
					</div>
					<div class="disk-info">
						<span class="disk-name">{disk.name}</span>
						<span class="disk-device">{disk.device} - {disk.size}</span>
					</div>
					<span class="health-dot {disk.health === 'healthy' ? 'bg-green-500' : disk.health === 'warning' ? 'bg-amber-500' : 'bg-red-500'}"></span>
				</button>
			{/each}
		</aside>

		<!-- Details -->
		<main class="details">
			{#if selectedDisk}
				<div class="detail-header">
					<div>
						<h2>{selectedDisk.name}</h2>
						<p>{selectedDisk.device}</p>
					</div>
					<div class="detail-actions">
						<button class="btn-icon" title="Info"><Icon icon="mdi:information" class="w-5 h-5" /></button>
						<button class="btn-icon" title="Settings"><Icon icon="mdi:cog" class="w-5 h-5" /></button>
					</div>
				</div>

				<div class="stats-grid">
					<div class="stat-card">
						<span class="stat-label">Type</span>
						<span class="stat-value">{selectedDisk.type}</span>
					</div>
					<div class="stat-card">
						<span class="stat-label">Capacity</span>
						<span class="stat-value">{selectedDisk.size}</span>
					</div>
					<div class="stat-card">
						<span class="stat-label">Used</span>
						<span class="stat-value">{selectedDisk.used}</span>
					</div>
					<div class="stat-card">
						<span class="stat-label">Temperature</span>
						<span class="stat-value {getTempColor(selectedDisk.temperature)}">{selectedDisk.temperature}°C</span>
					</div>
				</div>

				<div class="card">
					<h3>Health Status</h3>
					<div class="health-row">
						<span class="health-badge {getHealthColor(selectedDisk.health)}">
							{selectedDisk.health.charAt(0).toUpperCase() + selectedDisk.health.slice(1)}
						</span>
						<span class="health-time">Last checked: 5 minutes ago</span>
					</div>

					<div class="smart-grid">
						<div class="smart-item">
							<span class="smart-label">Read Errors</span>
							<span class="smart-value">0</span>
						</div>
						<div class="smart-item">
							<span class="smart-label">Write Errors</span>
							<span class="smart-value">0</span>
						</div>
						<div class="smart-item">
							<span class="smart-label">Power On Hours</span>
							<span class="smart-value">2,456</span>
						</div>
						<div class="smart-item">
							<span class="smart-label">Power Cycles</span>
							<span class="smart-value">124</span>
						</div>
					</div>
				</div>

				<div class="action-buttons">
					<button class="btn-secondary">
						<Icon icon="mdi:chart-line" class="w-4 h-4" />
						S.M.A.R.T. Info
					</button>
					<button class="btn-secondary btn-danger">
						<Icon icon="mdi:eraser" class="w-4 h-4" />
						Wipe Disk
					</button>
				</div>
			{:else}
				<div class="empty-state">
					<Icon icon="mdi:harddisk" class="w-16 h-16 text-slate-300" />
					<p>Select a disk to view details</p>
				</div>
			{/if}
		</main>
	</div>
</div>

<style>
	.storage-manager {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	.header {
		padding: 16px 20px;
		border-bottom: 1px solid #e2e8f0;
	}

	.header-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	.header-top h1 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.tabs {
		display: flex;
		gap: 4px;
	}

	.tab {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 8px;
		font-size: 13px;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.tab:hover { background: #f1f5f9; }
	.tab.active { background: #dbeafe; color: #2563eb; }

	.content {
		flex: 1;
		display: flex;
		overflow: hidden;
	}

	.sidebar {
		width: 280px;
		border-right: 1px solid #e2e8f0;
		overflow-y: auto;
		padding: 8px;
	}

	.disk-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border-radius: 10px;
		transition: all 0.15s ease;
		text-align: left;
	}

	.disk-item:hover { background: #f8fafc; }
	.disk-item.selected { background: #dbeafe; }

	.disk-icon {
		width: 40px;
		height: 40px;
		background: #f1f5f9;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.disk-info {
		flex: 1;
		min-width: 0;
	}

	.disk-name {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: #1e293b;
	}

	.disk-device {
		font-size: 12px;
		color: #64748b;
	}

	.health-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}

	.details {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.detail-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.detail-header h2 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
	}

	.detail-header p {
		font-size: 13px;
		color: #64748b;
	}

	.detail-actions {
		display: flex;
		gap: 8px;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
		margin-bottom: 24px;
	}

	.stat-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		padding: 16px;
	}

	.stat-label {
		display: block;
		font-size: 12px;
		color: #64748b;
		margin-bottom: 4px;
	}

	.stat-value {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 20px;
		margin-bottom: 24px;
	}

	.card h3 {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 16px;
	}

	.health-row {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 20px;
	}

	.health-badge {
		padding: 6px 14px;
		border-radius: 20px;
		font-size: 13px;
		font-weight: 500;
	}

	.health-time {
		font-size: 13px;
		color: #64748b;
	}

	.smart-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
	}

	.smart-item {
		background: white;
		border-radius: 8px;
		padding: 12px 16px;
	}

	.smart-label {
		display: block;
		font-size: 12px;
		color: #64748b;
		margin-bottom: 4px;
	}

	.smart-value {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.action-buttons {
		display: flex;
		gap: 12px;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover { background: #2563eb; }

	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: #f1f5f9;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 13px;
		color: #475569;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover { background: #e2e8f0; }
	.btn-secondary.btn-danger { color: #dc2626; }
	.btn-secondary.btn-danger:hover { background: #fef2f2; }

	.btn-icon {
		width: 36px;
		height: 36px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #64748b;
		transition: all 0.15s ease;
	}

	.btn-icon:hover { background: #f1f5f9; color: #334155; }

	.empty-state {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
	}

	.empty-state p {
		font-size: 14px;
		color: #94a3b8;
	}
</style>
