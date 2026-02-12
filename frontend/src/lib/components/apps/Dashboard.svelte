<script lang="ts">
	import Icon from '@iconify/svelte';
	import { systemStats } from '$stores/system';

	const storageOverview = {
		total: '4 TB',
		used: '2.1 TB',
		free: '1.9 TB',
		percent: 52.5
	};

	const recentActivity = [
		{ icon: 'mdi:folder-plus', text: 'Share "Media" created', time: '2 min ago' },
		{ icon: 'mdi:account-plus', text: 'User "john" added', time: '15 min ago' },
		{ icon: 'mdi:harddisk', text: 'Disk /dev/sdb mounted', time: '1 hour ago' },
		{ icon: 'mdi:update', text: 'System updated', time: '3 hours ago' }
	];

	const services = [
		{ name: 'SMB/CIFS', status: 'running', icon: 'mdi:microsoft-windows' },
		{ name: 'NFS', status: 'running', icon: 'mdi:folder-network' },
		{ name: 'SSH', status: 'running', icon: 'mdi:console' },
		{ name: 'FTP', status: 'stopped', icon: 'mdi:folder-upload' }
	];
</script>

<div class="dashboard">
	<!-- Header -->
	<header class="dashboard-header">
		<h1>Dashboard</h1>
		<button class="btn-refresh">
			<Icon icon="mdi:refresh" class="w-4 h-4" />
			Refresh
		</button>
	</header>

	<!-- Stats Grid -->
	<div class="stats-grid">
		<!-- CPU -->
		<div class="stat-card">
			<div class="stat-icon bg-blue-100">
				<Icon icon="mdi:cpu-64-bit" class="w-6 h-6 text-blue-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">CPU Usage</span>
				<span class="stat-value">{$systemStats.cpuUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-blue-500" style="width: {$systemStats.cpuUsage}%"></div>
			</div>
		</div>

		<!-- Memory -->
		<div class="stat-card">
			<div class="stat-icon bg-purple-100">
				<Icon icon="mdi:memory" class="w-6 h-6 text-purple-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">Memory</span>
				<span class="stat-value">{$systemStats.memoryUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-purple-500" style="width: {$systemStats.memoryUsage}%"></div>
			</div>
		</div>

		<!-- Storage -->
		<div class="stat-card">
			<div class="stat-icon bg-amber-100">
				<Icon icon="mdi:harddisk" class="w-6 h-6 text-amber-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">Storage</span>
				<span class="stat-value">{storageOverview.percent}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-amber-500" style="width: {storageOverview.percent}%"></div>
			</div>
		</div>

		<!-- Network -->
		<div class="stat-card">
			<div class="stat-icon bg-green-100">
				<Icon icon="mdi:lan" class="w-6 h-6 text-green-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">Network</span>
				<span class="stat-value text-green-600">Online</span>
			</div>
			<div class="stat-network">
				<span><Icon icon="mdi:arrow-down" class="w-3 h-3 text-green-500 inline" /> 1.2 MB/s</span>
				<span><Icon icon="mdi:arrow-up" class="w-3 h-3 text-blue-500 inline" /> 450 KB/s</span>
			</div>
		</div>
	</div>

	<!-- Main Content -->
	<div class="content-grid">
		<!-- Storage Overview -->
		<div class="card col-span-2">
			<h3 class="card-title">Storage Overview</h3>
			<div class="storage-stats">
				<div class="storage-bar">
					<div class="storage-bar-fill" style="width: {storageOverview.percent}%"></div>
				</div>
				<div class="storage-labels">
					<span>Used: {storageOverview.used}</span>
					<span>Free: {storageOverview.free}</span>
				</div>
			</div>

			<div class="storage-counts">
				<div class="count-item">
					<span class="count-value">3</span>
					<span class="count-label">Disks</span>
				</div>
				<div class="count-item">
					<span class="count-value">1</span>
					<span class="count-label">Pools</span>
				</div>
				<div class="count-item">
					<span class="count-value">5</span>
					<span class="count-label">Shares</span>
				</div>
			</div>
		</div>

		<!-- Services Status -->
		<div class="card">
			<h3 class="card-title">Services</h3>
			<div class="services-list">
				{#each services as service}
					<div class="service-item">
						<div class="service-info">
							<Icon icon={service.icon} class="w-4 h-4 text-slate-400" />
							<span>{service.name}</span>
						</div>
						<span class="service-status" class:running={service.status === 'running'}>
							{service.status}
						</span>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<!-- Recent Activity -->
	<div class="card">
		<h3 class="card-title">Recent Activity</h3>
		<div class="activity-list">
			{#each recentActivity as activity}
				<div class="activity-item">
					<div class="activity-icon">
						<Icon icon={activity.icon} class="w-4 h-4 text-slate-500" />
					</div>
					<span class="activity-text">{activity.text}</span>
					<span class="activity-time">{activity.time}</span>
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
	.dashboard {
		padding: 24px;
		background: white;
		height: 100%;
		overflow-y: auto;
	}

	.dashboard-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.dashboard-header h1 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
	}

	.btn-refresh {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #f1f5f9;
		border-radius: 8px;
		font-size: 13px;
		color: #475569;
		transition: all 0.15s ease;
	}

	.btn-refresh:hover {
		background: #e2e8f0;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 16px;
		margin-bottom: 24px;
	}

	.stat-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 16px;
	}

	.stat-icon {
		width: 44px;
		height: 44px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 12px;
	}

	.stat-info {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.stat-label {
		font-size: 13px;
		color: #64748b;
	}

	.stat-value {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.stat-bar {
		height: 4px;
		background: #e2e8f0;
		border-radius: 2px;
		overflow: hidden;
	}

	.stat-bar-fill {
		height: 100%;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.stat-network {
		display: flex;
		gap: 16px;
		font-size: 12px;
		color: #64748b;
	}

	.content-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 16px;
		margin-bottom: 24px;
	}

	.col-span-2 {
		grid-column: span 2;
	}

	.card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		padding: 20px;
	}

	.card-title {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 16px;
	}

	.storage-bar {
		height: 8px;
		background: #e2e8f0;
		border-radius: 4px;
		overflow: hidden;
		margin-bottom: 8px;
	}

	.storage-bar-fill {
		height: 100%;
		background: linear-gradient(90deg, #3b82f6, #8b5cf6);
		border-radius: 4px;
	}

	.storage-labels {
		display: flex;
		justify-content: space-between;
		font-size: 12px;
		color: #64748b;
		margin-bottom: 20px;
	}

	.storage-counts {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	.count-item {
		text-align: center;
		padding: 12px;
		background: white;
		border-radius: 8px;
	}

	.count-value {
		display: block;
		font-size: 24px;
		font-weight: 600;
		color: #1e293b;
	}

	.count-label {
		font-size: 12px;
		color: #64748b;
	}

	.services-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.service-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 12px;
		background: white;
		border-radius: 8px;
	}

	.service-info {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 13px;
		color: #475569;
	}

	.service-status {
		font-size: 11px;
		padding: 4px 10px;
		border-radius: 20px;
		background: #f1f5f9;
		color: #64748b;
	}

	.service-status.running {
		background: #dcfce7;
		color: #16a34a;
	}

	.activity-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.activity-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		border-radius: 8px;
		transition: background 0.15s ease;
	}

	.activity-item:hover {
		background: white;
	}

	.activity-icon {
		width: 32px;
		height: 32px;
		background: white;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.activity-text {
		flex: 1;
		font-size: 13px;
		color: #475569;
	}

	.activity-time {
		font-size: 12px;
		color: #94a3b8;
	}
</style>
