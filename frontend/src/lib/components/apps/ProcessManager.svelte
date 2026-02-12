<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api, type ProcessInfo } from '$lib/stores/api';

	let processes: ProcessInfo[] = [];
	let sortColumn: keyof ProcessInfo = 'cpu';
	let sortDirection: 'asc' | 'desc' = 'desc';
	let searchQuery = '';
	let refreshInterval: ReturnType<typeof setInterval>;
	let loading = true;
	let error: string | null = null;

	// System stats
	let cpuUsage = 0;
	let memoryUsage = 0;
	let totalMemory = 0;
	let usedMemory = 0;
	let totalProcesses = 0;
	let runningProcesses = 0;

	async function loadProcesses() {
		try {
			const data = await api.getProcesses();
			processes = data.processes;
			cpuUsage = data.cpu_usage;
			memoryUsage = data.memory_usage;
			totalMemory = data.total_memory;
			usedMemory = data.used_memory;
			totalProcesses = data.total_processes;
			runningProcesses = data.running_processes;
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : $t.processManager?.errors?.loadFailed || 'Failed to load processes';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadProcesses();
		// Refresh every 3 seconds
		refreshInterval = setInterval(loadProcesses, 3000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	function sortBy(column: keyof ProcessInfo) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}

	$: filteredProcesses = processes
		.filter(p => p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
					 p.user.toLowerCase().includes(searchQuery.toLowerCase()) ||
					 p.pid.toString().includes(searchQuery))
		.sort((a, b) => {
			const aVal = a[sortColumn];
			const bVal = b[sortColumn];
			const modifier = sortDirection === 'asc' ? 1 : -1;
			if (typeof aVal === 'string' && typeof bVal === 'string') {
				return aVal.localeCompare(bVal) * modifier;
			}
			return ((aVal as number) - (bVal as number)) * modifier;
		});

	function getStatusColor(status: string): string {
		switch (status) {
			case 'running': return 'text-green-500';
			case 'sleeping':
			case 'idle': return 'text-blue-400';
			case 'stopped': return 'text-red-500';
			case 'zombie': return 'text-orange-500';
			default: return 'text-gray-400';
		}
	}

	function formatMemory(bytes: number): string {
		if (bytes >= 1024 * 1024 * 1024) {
			return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
		}
		if (bytes >= 1024 * 1024) {
			return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
		}
		if (bytes >= 1024) {
			return (bytes / 1024).toFixed(1) + ' KB';
		}
		return bytes + ' B';
	}

	function formatTotalMemory(bytes: number): string {
		return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
	}

	async function handleKillProcess(pid: number) {
		if (!confirm($t.processManager?.confirmKill || `Are you sure you want to kill process ${pid}?`)) {
			return;
		}
		try {
			await api.killProcess(pid);
			// Remove from local list immediately for responsiveness
			processes = processes.filter(p => p.pid !== pid);
		} catch (e) {
			error = e instanceof Error ? e.message : $t.processManager?.errors?.killFailed || 'Failed to kill process';
		}
	}

	function getStatusLabel(status: string): string {
		return $t.processManager?.status?.[status] || status;
	}
</script>

<div class="process-manager">
	<!-- Header Stats -->
	<div class="stats-bar">
		<div class="stat">
			<div class="stat-icon cpu">
				<Icon icon="mdi:chip" class="w-5 h-5" />
			</div>
			<div class="stat-info">
				<span class="stat-label">CPU</span>
				<span class="stat-value">{cpuUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill cpu" style="width: {Math.min(cpuUsage, 100)}%"></div>
			</div>
		</div>

		<div class="stat">
			<div class="stat-icon memory">
				<Icon icon="mdi:memory" class="w-5 h-5" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.widgets?.memory || 'Memory'}</span>
				<span class="stat-value">{memoryUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill memory" style="width: {Math.min(memoryUsage, 100)}%"></div>
			</div>
		</div>

		<div class="stat-memory-info">
			<span class="memory-detail">{formatTotalMemory(usedMemory)} / {formatTotalMemory(totalMemory)}</span>
		</div>

		<div class="stat-count">
			<span class="count-value">{totalProcesses}</span>
			<span class="count-label">{$t.processManager?.processes || 'Processes'}</span>
		</div>

		<div class="stat-count">
			<span class="count-value running">{runningProcesses}</span>
			<span class="count-label">{$t.processManager?.running || 'Running'}</span>
		</div>
	</div>

	<!-- Toolbar -->
	<div class="toolbar">
		<div class="search-box">
			<Icon icon="mdi:magnify" class="w-4 h-4 text-gray-400" />
			<input
				type="text"
				placeholder={$t.common?.filter || 'Filter'}
				bind:value={searchQuery}
			/>
		</div>
		<button class="btn-refresh" onclick={loadProcesses}>
			<Icon icon="mdi:refresh" class="w-4 h-4" />
			{$t.common?.refresh || 'Refresh'}
		</button>
	</div>

	<!-- Error state -->
	{#if error}
		<div class="error-banner">
			<Icon icon="mdi:alert-circle" class="w-5 h-5" />
			<span>{error}</span>
			<button onclick={() => error = null}>
				<Icon icon="mdi:close" class="w-4 h-4" />
			</button>
		</div>
	{/if}

	<!-- Loading state -->
	{#if loading}
		<div class="loading-state">
			<Icon icon="mdi:loading" class="w-8 h-8 animate-spin text-blue-500" />
			<p>{$t.common?.loading || 'Loading...'}</p>
		</div>
	{:else}
		<!-- Process Table -->
		<div class="table-container">
			<table class="process-table">
				<thead>
					<tr>
						<th class="sortable" onclick={() => sortBy('pid')}>
							PID
							{#if sortColumn === 'pid'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th class="sortable" onclick={() => sortBy('name')}>
							{$t.processManager?.columns?.name || 'Name'}
							{#if sortColumn === 'name'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th class="sortable" onclick={() => sortBy('user')}>
							{$t.processManager?.columns?.user || 'User'}
							{#if sortColumn === 'user'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th class="sortable" onclick={() => sortBy('cpu')}>
							CPU %
							{#if sortColumn === 'cpu'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th class="sortable" onclick={() => sortBy('memory')}>
							{$t.processManager?.columns?.memory || 'Memory'}
							{#if sortColumn === 'memory'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th class="sortable" onclick={() => sortBy('status')}>
							{$t.processManager?.columns?.status || 'Status'}
							{#if sortColumn === 'status'}
								<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
							{/if}
						</th>
						<th>{$t.common?.actions || 'Actions'}</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredProcesses as process}
						<tr>
							<td class="pid">{process.pid}</td>
							<td class="name">
								<Icon icon="mdi:application" class="w-4 h-4 text-gray-400" />
								<span class="name-text" title={process.command || process.name}>{process.name}</span>
							</td>
							<td class="user">{process.user}</td>
							<td class="cpu">
								<div class="cpu-bar-container">
									<div class="cpu-bar" style="width: {Math.min(process.cpu, 100)}%"></div>
								</div>
								{process.cpu.toFixed(1)}%
							</td>
							<td class="memory">{formatMemory(process.memory)}</td>
							<td class="status">
								<span class="status-dot {getStatusColor(process.status)}">●</span>
								{getStatusLabel(process.status)}
							</td>
							<td class="actions">
								<button
									class="btn-kill"
									title={$t.processManager?.endProcess || 'End process'}
									onclick={() => handleKillProcess(process.pid)}
								>
									<Icon icon="mdi:close" class="w-4 h-4" />
								</button>
							</td>
						</tr>
					{/each}
					{#if filteredProcesses.length === 0}
						<tr>
							<td colspan="7" class="no-results">
								{$t.processManager?.noProcesses || 'No processes found'}
							</td>
						</tr>
					{/if}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.process-manager {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #f8fafc;
		font-size: 14px;
	}

	/* Stats Bar */
	.stats-bar {
		display: flex;
		align-items: center;
		gap: 24px;
		padding: 16px 20px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.stat {
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: 180px;
	}

	.stat-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
	}

	.stat-icon.cpu {
		background: linear-gradient(135deg, #3b82f6, #1d4ed8);
	}

	.stat-icon.memory {
		background: linear-gradient(135deg, #8b5cf6, #6d28d9);
	}

	.stat-info {
		display: flex;
		flex-direction: column;
	}

	.stat-label {
		font-size: 12px;
		color: #64748b;
	}

	.stat-value {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
	}

	.stat-bar {
		width: 80px;
		height: 6px;
		background: #e2e8f0;
		border-radius: 3px;
		overflow: hidden;
	}

	.stat-bar-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.3s ease;
	}

	.stat-bar-fill.cpu {
		background: linear-gradient(90deg, #3b82f6, #1d4ed8);
	}

	.stat-bar-fill.memory {
		background: linear-gradient(90deg, #8b5cf6, #6d28d9);
	}

	.stat-memory-info {
		padding: 0 16px;
		border-left: 1px solid #e2e8f0;
	}

	.memory-detail {
		font-size: 13px;
		color: #64748b;
	}

	.stat-count {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0 16px;
		border-left: 1px solid #e2e8f0;
	}

	.count-value {
		font-size: 24px;
		font-weight: 700;
		color: #1e293b;
	}

	.count-value.running {
		color: #22c55e;
	}

	.count-label {
		font-size: 12px;
		color: #64748b;
	}

	/* Toolbar */
	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 20px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.search-box {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #f1f5f9;
		border-radius: 8px;
		width: 250px;
	}

	.search-box input {
		flex: 1;
		border: none;
		background: transparent;
		outline: none;
		font-size: 14px;
		color: #1e293b;
	}

	.btn-refresh {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: #f1f5f9;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		color: #475569;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-refresh:hover {
		background: #e2e8f0;
	}

	/* Error banner */
	.error-banner {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 20px;
		background: #fef2f2;
		border-bottom: 1px solid #fecaca;
		color: #dc2626;
	}

	.error-banner span {
		flex: 1;
	}

	.error-banner button {
		color: #dc2626;
		opacity: 0.7;
	}

	.error-banner button:hover {
		opacity: 1;
	}

	/* Loading state */
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		padding: 60px 20px;
		color: #64748b;
	}

	/* Table */
	.table-container {
		flex: 1;
		overflow: auto;
		padding: 0 20px 20px;
	}

	.process-table {
		width: 100%;
		border-collapse: collapse;
		background: white;
		border-radius: 8px;
		overflow: hidden;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
	}

	.process-table th {
		padding: 12px 16px;
		text-align: left;
		font-size: 12px;
		font-weight: 600;
		color: #64748b;
		background: #f8fafc;
		border-bottom: 1px solid #e2e8f0;
		white-space: nowrap;
	}

	.process-table th.sortable {
		cursor: pointer;
		user-select: none;
	}

	.process-table th.sortable:hover {
		color: #3b82f6;
	}

	.process-table td {
		padding: 10px 16px;
		border-bottom: 1px solid #f1f5f9;
		color: #334155;
	}

	.process-table tr:hover {
		background: #f8fafc;
	}

	.pid {
		font-family: 'JetBrains Mono', monospace;
		color: #64748b;
	}

	.name {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 500;
	}

	.name-text {
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.user {
		color: #64748b;
	}

	.cpu {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.cpu-bar-container {
		width: 60px;
		height: 4px;
		background: #e2e8f0;
		border-radius: 2px;
		overflow: hidden;
	}

	.cpu-bar {
		height: 100%;
		background: #3b82f6;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.memory {
		font-family: 'JetBrains Mono', monospace;
	}

	.status {
		display: flex;
		align-items: center;
		gap: 6px;
		text-transform: capitalize;
	}

	.status-dot {
		font-size: 8px;
	}

	.actions {
		text-align: center;
	}

	.btn-kill {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border: none;
		background: transparent;
		border-radius: 6px;
		color: #94a3b8;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-kill:hover {
		background: #fee2e2;
		color: #dc2626;
	}

	.no-results {
		text-align: center;
		color: #64748b;
		padding: 40px 20px !important;
	}

	.text-green-500 { color: #22c55e; }
	.text-blue-400 { color: #60a5fa; }
	.text-red-500 { color: #ef4444; }
	.text-orange-500 { color: #f97316; }
	.text-gray-400 { color: #9ca3af; }

	/* Animation */
	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.animate-spin { animation: spin 1s linear infinite; }
</style>
