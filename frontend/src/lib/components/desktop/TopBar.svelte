<script lang="ts">
	import Icon from '@iconify/svelte';
	import { systemStats } from '$stores/system';
	import { createEventDispatcher } from 'svelte';
	import { t, locale } from '$lib/i18n';

	const dispatch = createEventDispatcher();

	function getLocaleCode(loc: string) {
		return loc === 'fr' ? 'fr-FR' : 'en-US';
	}

	$: localeCode = getLocaleCode($locale);

	$: time = new Date().toLocaleTimeString(localeCode, {
		hour: '2-digit',
		minute: '2-digit'
	});

	$: date = new Date().toLocaleDateString(localeCode, {
		weekday: 'short',
		day: 'numeric',
		month: 'short'
	});

	setInterval(() => {
		time = new Date().toLocaleTimeString(localeCode, {
			hour: '2-digit',
			minute: '2-digit'
		});
	}, 60000);

	let showNotifications = false;

	function toggleAppLauncher() {
		dispatch('toggleLauncher');
	}
</script>

<header class="topbar">
	<!-- Left: App launcher -->
	<div class="flex items-center gap-4">
		<button class="topbar-btn" on:click={toggleAppLauncher} title={$t.desktop.appLauncher.title}>
			<Icon icon="mdi:apps" class="w-5 h-5" />
		</button>
	</div>

	<!-- Right: System indicators -->
	<div class="flex items-center gap-1">
		<!-- System stats -->
		<div class="stats-group">
			<div class="stat-item" title={$t.widgets.cpu}>
				<Icon icon="mdi:chip" class="w-4 h-4 opacity-60" />
				<span>{$systemStats.cpuUsage.toFixed(0)}%</span>
			</div>
			<div class="stat-item" title={$t.widgets.memory}>
				<Icon icon="mdi:memory" class="w-4 h-4 opacity-60" />
				<span>{$systemStats.memoryUsage.toFixed(0)}%</span>
			</div>
		</div>

		<div class="divider"></div>

		<!-- Network stats -->
		<div class="stats-group">
			<div class="stat-item text-xs" title={$t.widgets.upload}>
				<Icon icon="mdi:arrow-up" class="w-3 h-3 text-green-500" />
				<span>432.2 KB/s</span>
			</div>
			<div class="stat-item text-xs" title={$t.widgets.download}>
				<Icon icon="mdi:arrow-down" class="w-3 h-3 text-blue-500" />
				<span>323.4 KB/s</span>
			</div>
		</div>

		<div class="divider"></div>

		<!-- Quick actions -->
		<button class="topbar-btn" title="Widgets">
			<Icon icon="mdi:widgets-outline" class="w-5 h-5" />
		</button>

		<button class="topbar-btn relative" title={$t.topBar.notifications} on:click={() => showNotifications = !showNotifications}>
			<Icon icon="mdi:bell-outline" class="w-5 h-5" />
			<span class="notification-badge">3</span>
		</button>

		<button class="topbar-btn" title={$t.common.search}>
			<Icon icon="mdi:magnify" class="w-5 h-5" />
		</button>

		<div class="divider"></div>

		<!-- User -->
		<button class="user-btn">
			<div class="avatar">
				<span>A</span>
			</div>
		</button>
	</div>
</header>

<style>
	.topbar {
		height: 40px;
		background: rgba(30, 41, 59, 0.85);
		backdrop-filter: blur(20px);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		z-index: 100;
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.topbar-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		color: rgba(255, 255, 255, 0.8);
		transition: all 0.2s;
	}

	.topbar-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: white;
	}

	.stats-group {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0 8px;
	}

	.stat-item {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: rgba(255, 255, 255, 0.7);
	}

	.divider {
		width: 1px;
		height: 20px;
		background: rgba(255, 255, 255, 0.15);
		margin: 0 4px;
	}

	.notification-badge {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 16px;
		height: 16px;
		background: #ef4444;
		border-radius: 50%;
		font-size: 10px;
		font-weight: 600;
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.user-btn {
		padding: 4px;
		border-radius: 8px;
		transition: all 0.2s;
	}

	.user-btn:hover {
		background: rgba(255, 255, 255, 0.1);
	}

	.avatar {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: linear-gradient(135deg, #3b82f6, #8b5cf6);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 600;
		color: white;
	}
</style>
