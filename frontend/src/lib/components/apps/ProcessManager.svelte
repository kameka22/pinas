<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	interface Process {
		pid: number;
		name: string;
		user: string;
		cpu: number;
		memory: number;
		status: 'running' | 'sleeping' | 'stopped';
	}

	// Fake process data
	const fakeProcesses: Process[] = [
		{ pid: 1, name: 'systemd', user: 'root', cpu: 0.1, memory: 12.4, status: 'running' },
		{ pid: 245, name: 'kodi', user: 'kodi', cpu: 8.2, memory: 324.5, status: 'running' },
		{ pid: 312, name: 'pinas-backend', user: 'root', cpu: 2.4, memory: 48.2, status: 'running' },
		{ pid: 456, name: 'smbd', user: 'root', cpu: 0.3, memory: 18.7, status: 'running' },
		{ pid: 489, name: 'nmbd', user: 'root', cpu: 0.1, memory: 8.2, status: 'running' },
		{ pid: 523, name: 'avahi-daemon', user: 'avahi', cpu: 0.0, memory: 4.1, status: 'sleeping' },
		{ pid: 567, name: 'sshd', user: 'root', cpu: 0.0, memory: 6.8, status: 'sleeping' },
		{ pid: 612, name: 'dockerd', user: 'root', cpu: 1.2, memory: 86.4, status: 'running' },
		{ pid: 678, name: 'containerd', user: 'root', cpu: 0.8, memory: 42.1, status: 'running' },
		{ pid: 734, name: 'rsyslogd', user: 'root', cpu: 0.0, memory: 5.6, status: 'sleeping' },
		{ pid: 789, name: 'crond', user: 'root', cpu: 0.0, memory: 2.1, status: 'sleeping' },
		{ pid: 823, name: 'nginx', user: 'www-data', cpu: 0.2, memory: 12.3, status: 'running' },
		{ pid: 891, name: 'php-fpm', user: 'www-data', cpu: 0.5, memory: 28.9, status: 'sleeping' },
		{ pid: 934, name: 'mariadb', user: 'mysql', cpu: 1.8, memory: 156.2, status: 'running' },
		{ pid: 1023, name: 'node', user: 'pinas', cpu: 3.1, memory: 92.4, status: 'running' },
	];

	let processes: Process[] = [...fakeProcesses];
	let sortColumn: keyof Process = 'cpu';
	let sortDirection: 'asc' | 'desc' = 'desc';
	let searchQuery = '';
	let refreshInterval: ReturnType<typeof setInterval>;

	// System stats (fake)
	let cpuUsage = 15.4;
	let memoryUsage = 42.8;
	let totalProcesses = processes.length;
	let runningProcesses = processes.filter(p => p.status === 'running').length;

	onMount(() => {
		// Simulate random CPU/memory fluctuations
		refreshInterval = setInterval(() => {
			processes = processes.map(p => ({
				...p,
				cpu: Math.max(0, p.cpu + (Math.random() - 0.5) * 0.5),
				memory: Math.max(0, p.memory + (Math.random() - 0.5) * 2)
			}));
			cpuUsage = Math.max(5, Math.min(95, cpuUsage + (Math.random() - 0.5) * 3));
			memoryUsage = Math.max(20, Math.min(90, memoryUsage + (Math.random() - 0.5) * 2));
		}, 2000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	function sortBy(column: keyof Process) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}

	$: filteredProcesses = processes
		.filter(p => p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
					 p.user.toLowerCase().includes(searchQuery.toLowerCase()))
		.sort((a, b) => {
			const aVal = a[sortColumn];
			const bVal = b[sortColumn];
			const modifier = sortDirection === 'asc' ? 1 : -1;
			if (typeof aVal === 'string' && typeof bVal === 'string') {
				return aVal.localeCompare(bVal) * modifier;
			}
			return ((aVal as number) - (bVal as number)) * modifier;
		});

	function getStatusColor(status: Process['status']): string {
		switch (status) {
			case 'running': return 'text-green-500';
			case 'sleeping': return 'text-blue-400';
			case 'stopped': return 'text-red-500';
			default: return 'text-gray-400';
		}
	}

	function formatMemory(mb: number): string {
		if (mb >= 1024) {
			return (mb / 1024).toFixed(1) + ' GB';
		}
		return mb.toFixed(1) + ' MB';
	}

	function handleKillProcess(pid: number) {
		// Fake kill - just remove from list
		processes = processes.filter(p => p.pid !== pid);
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
				<div class="stat-bar-fill cpu" style="width: {cpuUsage}%"></div>
			</div>
		</div>

		<div class="stat">
			<div class="stat-icon memory">
				<Icon icon="mdi:memory" class="w-5 h-5" />
			</div>
			<div class="stat-info">
				<span class="stat-label">{$t.widgets.memory}</span>
				<span class="stat-value">{memoryUsage.toFixed(1)}%</span>
			</div>
			<div class="stat-bar">
				<div class="stat-bar-fill memory" style="width: {memoryUsage}%"></div>
			</div>
		</div>

		<div class="stat-count">
			<span class="count-value">{totalProcesses}</span>
			<span class="count-label">Processes</span>
		</div>

		<div class="stat-count">
			<span class="count-value running">{runningProcesses}</span>
			<span class="count-label">Running</span>
		</div>
	</div>

	<!-- Toolbar -->
	<div class="toolbar">
		<div class="search-box">
			<Icon icon="mdi:magnify" class="w-4 h-4 text-gray-400" />
			<input
				type="text"
				placeholder={$t.common.filter}
				bind:value={searchQuery}
			/>
		</div>
		<button class="btn-refresh" on:click={() => processes = [...fakeProcesses]}>
			<Icon icon="mdi:refresh" class="w-4 h-4" />
			{$t.common.refresh}
		</button>
	</div>

	<!-- Process Table -->
	<div class="table-container">
		<table class="process-table">
			<thead>
				<tr>
					<th class="sortable" on:click={() => sortBy('pid')}>
						PID
						{#if sortColumn === 'pid'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th class="sortable" on:click={() => sortBy('name')}>
						Name
						{#if sortColumn === 'name'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th class="sortable" on:click={() => sortBy('user')}>
						User
						{#if sortColumn === 'user'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th class="sortable" on:click={() => sortBy('cpu')}>
						CPU %
						{#if sortColumn === 'cpu'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th class="sortable" on:click={() => sortBy('memory')}>
						Memory
						{#if sortColumn === 'memory'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th class="sortable" on:click={() => sortBy('status')}>
						Status
						{#if sortColumn === 'status'}
							<Icon icon={sortDirection === 'asc' ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="w-4 h-4" />
						{/if}
					</th>
					<th>Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each filteredProcesses as process}
					<tr>
						<td class="pid">{process.pid}</td>
						<td class="name">
							<Icon icon="mdi:application" class="w-4 h-4 text-gray-400" />
							{process.name}
						</td>
						<td class="user">{process.user}</td>
						<td class="cpu">
							<div class="cpu-bar-container">
								<div class="cpu-bar" style="width: {Math.min(process.cpu * 10, 100)}%"></div>
							</div>
							{process.cpu.toFixed(1)}%
						</td>
						<td class="memory">{formatMemory(process.memory)}</td>
						<td class="status">
							<span class="status-dot {getStatusColor(process.status)}">●</span>
							{process.status}
						</td>
						<td class="actions">
							<button
								class="btn-kill"
								title="End process"
								on:click={() => handleKillProcess(process.pid)}
							>
								<Icon icon="mdi:close" class="w-4 h-4" />
							</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
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

	.text-green-500 { color: #22c55e; }
	.text-blue-400 { color: #60a5fa; }
	.text-red-500 { color: #ef4444; }
	.text-gray-400 { color: #9ca3af; }
</style>
