<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t, locale } from '$lib/i18n';
	import { systemStats, formatBytes } from '$stores/system';
	import { api } from '$stores/api';
	import type { StoragePool, SystemServiceSummary } from '$stores/api';
	import { notifications, loadNotifications, timeAgo, notificationVisual } from '$stores/notifications';

	let pools: StoragePool[] = [];
	let diskCount = 0;
	let volumeCount = 0;
	let shareCount = 0;
	let services: SystemServiceSummary[] = [];
	let loading = true;

	$: storageTotal = pools.reduce((acc, p) => acc + (p.total_size || 0), 0);
	$: storageUsed = pools.reduce((acc, p) => acc + (p.used_size || 0), 0);
	$: storagePercent = storageTotal > 0 ? Math.round((storageUsed / storageTotal) * 1000) / 10 : 0;
	$: recent = $notifications.slice(0, 8);

	const serviceIcons: Record<string, string> = {
		samba: 'mdi:microsoft-windows',
		nfs: 'mdi:folder-network',
		ssh: 'mdi:console',
		docker: 'mdi:docker'
	};

	async function refresh() {
		const [p, d, v, s, svc] = await Promise.allSettled([
			api.getPools(),
			api.getDisks(),
			api.getVolumes(),
			api.getShares(),
			api.getSystemServices()
		]);
		if (p.status === 'fulfilled') pools = p.value;
		if (d.status === 'fulfilled') diskCount = d.value.length;
		if (v.status === 'fulfilled') volumeCount = v.value.length;
		if (s.status === 'fulfilled') shareCount = (s.value as unknown[]).length;
		if (svc.status === 'fulfilled') services = svc.value;
		await loadNotifications();
		loading = false;
	}

	onMount(refresh);
</script>

<div class="dashboard">
	<header class="dashboard-header">
		<h1>{$t.dashboard.title}</h1>
		<button class="btn-refresh" on:click={refresh} disabled={loading}>
			<Icon icon="mdi:refresh" class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
			{$t.common.refresh}
		</button>
	</header>

	<div class="stats-grid">
		<div class="stat-card">
			<div class="stat-icon bg-blue-100">
				<Icon icon="mdi:cpu-64-bit" class="w-6 h-6 text-blue-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.dashboard.cpu}</span>
				<span class="stat-value">{$systemStats.cpuUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-blue-500" style="width: {$systemStats.cpuUsage}%"></div>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon bg-purple-100">
				<Icon icon="mdi:memory" class="w-6 h-6 text-purple-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.dashboard.memory}</span>
				<span class="stat-value">{$systemStats.memoryUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-purple-500" style="width: {$systemStats.memoryUsage}%"></div>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon bg-amber-100">
				<Icon icon="mdi:harddisk" class="w-6 h-6 text-amber-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.dashboard.storage}</span>
				<span class="stat-value">{storageTotal > 0 ? `${storagePercent}%` : '—'}</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill bg-amber-500" style="width: {storagePercent}%"></div>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon bg-green-100">
				<Icon icon="mdi:apps" class="w-6 h-6 text-green-500" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.dashboard.services}</span>
				<span class="stat-value">{services.filter((s) => s.status === 'running').length}/{services.length}</span>
			</div>
		</div>
	</div>

	<div class="content-grid">
		<div class="card col-span-2">
			<h3 class="card-title">{$t.dashboard.storageOverview}</h3>
			{#if storageTotal > 0}
				<div class="storage-stats">
					<div class="storage-bar">
						<div class="storage-bar-fill" style="width: {storagePercent}%"></div>
					</div>
					<div class="storage-labels">
						<span>{$t.dashboard.used}: {formatBytes(storageUsed)}</span>
						<span>{$t.dashboard.free}: {formatBytes(Math.max(storageTotal - storageUsed, 0))}</span>
					</div>
				</div>
			{:else}
				<p class="empty-hint">{$t.dashboard.noPools}</p>
			{/if}

			<div class="storage-counts">
				<div class="count-item">
					<span class="count-value">{diskCount}</span>
					<span class="count-label">{$t.dashboard.disks}</span>
				</div>
				<div class="count-item">
					<span class="count-value">{pools.length}</span>
					<span class="count-label">{$t.dashboard.pools}</span>
				</div>
				<div class="count-item">
					<span class="count-value">{volumeCount}</span>
					<span class="count-label">{$t.dashboard.volumes}</span>
				</div>
				<div class="count-item">
					<span class="count-value">{shareCount}</span>
					<span class="count-label">{$t.dashboard.shares}</span>
				</div>
			</div>
		</div>

		<div class="card">
			<h3 class="card-title">{$t.dashboard.services}</h3>
			<div class="services-list">
				{#each services as service (service.name)}
					<div class="service-item">
						<div class="service-info">
							<Icon icon={serviceIcons[service.name] || 'mdi:cog'} class="w-4 h-4 text-slate-400" />
							<span>{service.name.toUpperCase()}</span>
						</div>
						<span class="service-status" class:running={service.status === 'running'}>
							{service.status === 'running' ? $t.dashboard.running : $t.dashboard.stopped}
						</span>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<div class="card">
		<h3 class="card-title">{$t.dashboard.recentActivity}</h3>
		<div class="activity-list">
			{#if recent.length === 0}
				<p class="empty-hint">{$t.dashboard.noActivity}</p>
			{/if}
			{#each recent as n (n.id)}
				<div class="activity-item">
					<div class="activity-icon">
						<Icon icon={notificationVisual(n).icon} class="w-4 h-4 text-slate-500" />
					</div>
					<span class="activity-text"><strong>{n.title}</strong> — {n.message}</span>
					<span class="activity-time">{timeAgo(n.created_at, $locale)}</span>
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

	.empty-hint {
		font-size: 0.8125rem;
		color: #94a3b8;
		padding: 0.5rem 0 1rem;
	}
	.btn-refresh:disabled { opacity: 0.6; }
</style>
