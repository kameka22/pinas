<script lang="ts">
	import Icon from '@iconify/svelte';
	import { systemStats } from '$stores/system';
	import { auth, api } from '$stores/api';
	import { createEventDispatcher, onMount, onDestroy } from 'svelte';
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
	let showUserMenu = false;
	let userMenuRef: HTMLDivElement;

	function toggleAppLauncher() {
		dispatch('toggleLauncher');
	}

	function toggleUserMenu() {
		showUserMenu = !showUserMenu;
	}

	async function handleLogout() {
		showUserMenu = false;
		await api.logout();
		// No need to reload - the layout will detect auth change and show login
	}

	function openProfileModal() {
		dispatch('openProfile');
		showUserMenu = false;
	}

	function openChangePasswordModal() {
		dispatch('openChangePassword');
		showUserMenu = false;
	}

	function handleClickOutside(event: MouseEvent) {
		if (userMenuRef && !userMenuRef.contains(event.target as Node)) {
			showUserMenu = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
	});

	onDestroy(() => {
		document.removeEventListener('click', handleClickOutside);
	});

	$: userInitial = $auth.user?.username?.[0]?.toUpperCase() || 'U';
	$: userName = $auth.user?.username || 'User';
	$: userRole = $auth.user?.role === 'admin' ? 'Administrator' : 'User';
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

		<!-- User dropdown -->
		<div class="user-dropdown" bind:this={userMenuRef}>
			<button class="user-btn" on:click={toggleUserMenu}>
				<div class="avatar">
					<span>{userInitial}</span>
				</div>
			</button>

			{#if showUserMenu}
				<div class="dropdown-menu">
					<div class="dropdown-header">
						<div class="dropdown-avatar">
							<span>{userInitial}</span>
						</div>
						<div class="dropdown-user-info">
							<span class="dropdown-username">{userName}</span>
							<span class="dropdown-role">{userRole}</span>
						</div>
					</div>

					<div class="dropdown-divider"></div>

					<button class="dropdown-item" on:click={openProfileModal}>
						<Icon icon="mdi:account" class="w-4 h-4" />
						<span>{$t.topBar?.profile || 'Profile'}</span>
					</button>

					<button class="dropdown-item" on:click={openChangePasswordModal}>
						<Icon icon="mdi:key-variant" class="w-4 h-4" />
						<span>{$t.topBar?.changePassword || 'Change Password'}</span>
					</button>

					<div class="dropdown-divider"></div>

					<button class="dropdown-item danger" on:click={handleLogout}>
						<Icon icon="mdi:logout" class="w-4 h-4" />
						<span>{$t.topBar?.logout || 'Logout'}</span>
					</button>
				</div>
			{/if}
		</div>
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

	.user-dropdown {
		position: relative;
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

	.dropdown-menu {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		width: 220px;
		background: white;
		border-radius: 12px;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
		overflow: hidden;
		z-index: 1000;
		animation: dropdown-fade-in 0.15s ease-out;
	}

	@keyframes dropdown-fade-in {
		from {
			opacity: 0;
			transform: translateY(-8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.dropdown-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px;
		background: linear-gradient(135deg, #f8fafc, #f1f5f9);
	}

	.dropdown-avatar {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: linear-gradient(135deg, #3b82f6, #8b5cf6);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 16px;
		font-weight: 600;
		color: white;
	}

	.dropdown-user-info {
		display: flex;
		flex-direction: column;
	}

	.dropdown-username {
		font-size: 14px;
		font-weight: 600;
		color: #1e293b;
	}

	.dropdown-role {
		font-size: 12px;
		color: #64748b;
	}

	.dropdown-divider {
		height: 1px;
		background: #e2e8f0;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 12px 16px;
		text-align: left;
		font-size: 14px;
		color: #475569;
		transition: background 0.15s;
	}

	.dropdown-item:hover {
		background: #f1f5f9;
	}

	.dropdown-item.danger {
		color: #dc2626;
	}

	.dropdown-item.danger:hover {
		background: #fef2f2;
	}
</style>
