<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t, locale } from '$lib/i18n';
	import { auth, api, type StoragePool, type SystemServiceSummary, type DockerStats } from '$stores/api';
	import { systemStats, formatBytes, formatRate } from '$stores/system';
	import { notifications, loadNotifications, timeAgo, notificationVisual } from '$stores/notifications';
	import { widgetsVisible, enabledWidgets, loadWidgetPreferences } from '$stores/widgets';
	import { gradientStyle } from '$lib/utils/gradient';

	let pools: StoragePool[] = [];
	let services: SystemServiceSummary[] = [];
	let docker: DockerStats | null = null;
	let refreshTimer: ReturnType<typeof setInterval> | null = null;

	$: isAdmin = $auth.user?.role === 'admin';
	$: storageTotal = pools.reduce((a, p) => a + (p.total_size || 0), 0);
	$: storageUsed = pools.reduce((a, p) => a + (p.used_size || 0), 0);
	$: storagePercent = storageTotal > 0 ? Math.round((storageUsed / storageTotal) * 100) : 0;

	async function refresh() {
		if (!isAdmin) return;
		const [p, s, d] = await Promise.allSettled([api.getPools(), api.getSystemServices(), api.getDockerStatus()]);
		if (p.status === 'fulfilled') pools = p.value;
		if (s.status === 'fulfilled') services = s.value;
		if (d.status === 'fulfilled') docker = d.value;
		await loadNotifications();
	}

	$: if ($widgetsVisible) {
		loadWidgetPreferences();
		refresh();
		if (!refreshTimer) refreshTimer = setInterval(refresh, 15000);
	} else if (refreshTimer) {
		clearInterval(refreshTimer);
		refreshTimer = null;
	}
</script>

{#if $widgetsVisible}
	<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
	<div class="widgets-overlay" on:click={() => widgetsVisible.set(false)}></div>
{/if}

<aside class="widgets-panel" class:visible={$widgetsVisible}>
	<header class="widgets-header">
		<Icon icon="mdi:widgets-outline" class="w-5 h-5 text-slate-600" />
		<h2>{$t.widgets.title}</h2>
	</header>

	<div class="widgets-list">
		{#if $enabledWidgets.includes('system')}
			<section class="widget">
				<h3><Icon icon="mdi:chip" class="w-4 h-4" />{$t.widgets.system}</h3>
				<div class="meter"><span>{$t.widgets.cpu}</span><div class="bar"><div class="fill bg-blue-500" style="width: {$systemStats.cpuUsage}%"></div></div><strong>{$systemStats.cpuUsage.toFixed(0)}%</strong></div>
				<div class="meter"><span>{$t.widgets.memory}</span><div class="bar"><div class="fill bg-purple-500" style="width: {$systemStats.memoryUsage}%"></div></div><strong>{$systemStats.memoryUsage.toFixed(0)}%</strong></div>
				<div class="kv"><span>{$t.widgets.memory}</span><span>{formatBytes($systemStats.memoryUsed)} / {formatBytes($systemStats.memoryTotal)}</span></div>
				<div class="kv"><span>{$t.widgets.network}</span><span><Icon icon="mdi:arrow-down" class="w-3 h-3 inline text-blue-500" /> {formatRate($systemStats.networkRx)} · <Icon icon="mdi:arrow-up" class="w-3 h-3 inline text-green-500" /> {formatRate($systemStats.networkTx)}</span></div>
			</section>
		{/if}

		{#if isAdmin && $enabledWidgets.includes('storage')}
			<section class="widget">
				<h3><Icon icon="mdi:harddisk" class="w-4 h-4" />{$t.widgets.storage}</h3>
				{#if pools.length === 0}
					<p class="empty">{$t.dashboard.noPools}</p>
				{:else}
					<div class="meter"><span>{$t.dashboard.used}</span><div class="bar"><div class="fill bg-amber-500" style="width: {storagePercent}%"></div></div><strong>{storagePercent}%</strong></div>
					<div class="kv"><span>{formatBytes(storageUsed)} / {formatBytes(storageTotal)}</span><span>{pools.length} {$t.dashboard.pools.toLowerCase()}</span></div>
					{#each pools as pool (pool.id)}
						<div class="kv small"><span>{pool.name}</span><span class:warn={pool.status !== 'normal'}>{pool.status}</span></div>
					{/each}
				{/if}
			</section>
		{/if}

		{#if isAdmin && $enabledWidgets.includes('services')}
			<section class="widget">
				<h3><Icon icon="mdi:cog-outline" class="w-4 h-4" />{$t.dashboard.services}</h3>
				{#each services as s (s.name)}
					<div class="kv small"><span>{s.name.toUpperCase()}</span><span class="dot" class:on={s.status === 'running'}>{s.status === 'running' ? $t.dashboard.running : $t.dashboard.stopped}</span></div>
				{/each}
			</section>
		{/if}

		{#if isAdmin && $enabledWidgets.includes('docker') && docker}
			<section class="widget">
				<h3><Icon icon="mdi:docker" class="w-4 h-4" />Docker</h3>
				{#if !docker.running}
					<p class="empty">{$t.widgets.dockerOff}</p>
				{:else}
					<div class="kv"><span>{$t.widgets.containers}</span><span>{docker.containers_running} / {docker.containers_total}</span></div>
					<div class="kv small"><span>{$t.widgets.images}</span><span>{docker.images}</span></div>
				{/if}
			</section>
		{/if}

		{#if isAdmin && $enabledWidgets.includes('notifications')}
			<section class="widget">
				<h3><Icon icon="mdi:bell-outline" class="w-4 h-4" />{$t.notifications.title}</h3>
				{#if $notifications.length === 0}
					<p class="empty">{$t.notifications.empty}</p>
				{/if}
				{#each $notifications.slice(0, 5) as n (n.id)}
					<div class="notif" class:unread={!n.read}>
						<div class="notif-icon" style={gradientStyle(notificationVisual(n).gradient)}><Icon icon={notificationVisual(n).icon} class="w-3.5 h-3.5 text-white" /></div>
						<div class="notif-body"><span class="notif-title">{n.title}</span><span class="notif-time">{timeAgo(n.created_at, $locale)}</span></div>
					</div>
				{/each}
			</section>
		{/if}
	</div>
	<footer class="widgets-footer">{$t.widgets.configureHint}</footer>
</aside>

<style>
	.widgets-overlay { position: fixed; inset: 0; z-index: 199; }
	.widgets-panel {
		position: fixed; top: 48px; right: -360px; width: 340px; max-height: calc(100vh - 60px);
		background: rgba(255, 255, 255, 0.95); backdrop-filter: blur(20px); border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25); z-index: 200; display: flex; flex-direction: column;
		transition: right 0.3s ease; overflow: hidden;
	}
	.widgets-panel.visible { right: 12px; }
	.widgets-header { display: flex; align-items: center; gap: 0.5rem; padding: 0.875rem 1rem; border-bottom: 1px solid #e2e8f0; }
	.widgets-header h2 { font-size: 0.875rem; font-weight: 600; color: #0f172a; margin: 0; }
	.widgets-list { overflow-y: auto; padding: 0.75rem; display: flex; flex-direction: column; gap: 0.75rem; }
	.widget { background: white; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 0.75rem 0.875rem; font-size: 0.8125rem; }
	.widget h3 { display: flex; align-items: center; gap: 0.375rem; font-size: 0.75rem; font-weight: 600; color: #475569; text-transform: uppercase; letter-spacing: 0.02em; margin: 0 0 0.5rem; }
	.meter { display: grid; grid-template-columns: 4rem 1fr 2.5rem; align-items: center; gap: 0.5rem; margin: 0.25rem 0; color: #64748b; font-size: 0.75rem; }
	.meter strong { text-align: right; color: #0f172a; }
	.bar { height: 6px; background: #e2e8f0; border-radius: 999px; overflow: hidden; }
	.fill { height: 100%; border-radius: 999px; transition: width 0.4s; }
	.kv { display: flex; justify-content: space-between; gap: 0.5rem; margin: 0.25rem 0; color: #334155; }
	.kv.small { font-size: 0.75rem; color: #64748b; }
	.warn { color: #b45309; }
	.dot { color: #b45309; }
	.dot.on { color: #15803d; }
	.empty { font-size: 0.75rem; color: #94a3b8; margin: 0; }
	.notif { display: flex; align-items: center; gap: 0.5rem; padding: 0.3rem 0; border-top: 1px solid #f1f5f9; }
	.notif.unread .notif-title { font-weight: 600; }
	.notif-icon { width: 1.5rem; height: 1.5rem; border-radius: 0.5rem; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
	.notif-body { display: flex; flex-direction: column; min-width: 0; }
	.notif-title { font-size: 0.75rem; color: #0f172a; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.notif-time { font-size: 0.6875rem; color: #94a3b8; }
	.widgets-footer { padding: 0.5rem 1rem; font-size: 0.6875rem; color: #94a3b8; border-top: 1px solid #e2e8f0; }
</style>
