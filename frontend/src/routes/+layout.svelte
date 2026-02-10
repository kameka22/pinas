<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import TopBar from '$components/desktop/TopBar.svelte';
	import Dock from '$components/desktop/Dock.svelte';
	import WindowManager from '$components/desktop/WindowManager.svelte';
	import NotificationCenter from '$components/desktop/NotificationCenter.svelte';
	import AppLauncher from '$components/desktop/AppLauncher.svelte';
	import Onboarding from '$components/onboarding/Onboarding.svelte';
	import Login from '$components/auth/Login.svelte';
	import ProfileModal from '$components/modals/ProfileModal.svelte';
	import ChangePasswordModal from '$components/modals/ChangePasswordModal.svelte';
	import UpdateSuccessModal from '$components/modals/UpdateSuccessModal.svelte';
	import UpdateScreen from '$components/apps/UpdateScreen.svelte';
	import TaskManager from '$components/desktop/TaskManager.svelte';
	import { connectWebSocket } from '$stores/websocket';
	import { isSetupComplete, isLoading, initOnboarding } from '$stores/onboarding';
	import { auth, api } from '$stores/api';
	import { systemInfo } from '$stores/system';
	import { updateScreen } from '$stores/update';
	import type { JustUpdatedResult } from '$stores/api';

	let showNotifications = false;
	let showAppLauncher = false;
	let showTaskManager = false;
	let showProfileModal = false;
	let showChangePasswordModal = false;
	let showUpdateModal = false;
	let updateModalVersion = '';
	let updateModalPreviousVersion = '';
	let updateModalChangelog: Record<string, string> | null = null;

	// Check if user is authenticated
	$: isAuthenticated = $auth.isAuthenticated;

	function toggleAppLauncher() {
		showAppLauncher = !showAppLauncher;
	}

	function closeAppLauncher() {
		showAppLauncher = false;
	}

	function toggleTaskManager() {
		showTaskManager = !showTaskManager;
		if (showTaskManager) showAppLauncher = false;
	}

	function closeTaskManager() {
		showTaskManager = false;
	}

	function openProfile() {
		showProfileModal = true;
	}

	function openChangePassword() {
		showChangePasswordModal = true;
	}

	let wsDisconnect: (() => void) | null = null;

	onMount(() => {
		console.log('[Layout] Calling initOnboarding...');
		initOnboarding();

		return () => {
			if (wsDisconnect) {
				wsDisconnect();
				wsDisconnect = null;
			}
		};
	});

	// Connect WebSocket only when authenticated
	$: if (isAuthenticated && !wsDisconnect) {
		wsDisconnect = connectWebSocket();
		// Check if system was just updated
		checkJustUpdated();
	} else if (!isAuthenticated && wsDisconnect) {
		wsDisconnect();
		wsDisconnect = null;
	}

	async function checkJustUpdated() {
		try {
			const result: JustUpdatedResult = await api.getJustUpdated();
			if (result.just_updated && result.version) {
				updateModalVersion = result.version;
				updateModalPreviousVersion = result.previous_version || '';
				updateModalChangelog = result.changelog || null;
				showUpdateModal = true;
			}
		} catch (e) {
			// Silently ignore - not critical
		}
	}
</script>

<!-- Show loading screen while checking setup state -->
{#if $isLoading}
	<div class="loading-screen">
		<div class="loading-spinner"></div>
		<p style="color: white; margin-top: 16px;">Loading...</p>
	</div>
<!-- Show onboarding if setup is not complete -->
{:else if !$isSetupComplete}
	<Onboarding />
<!-- Show login if setup is complete but user is not authenticated -->
{:else if !isAuthenticated}
	<Login />
{:else}
<div class="desktop">
	<!-- Wallpaper Background -->
	<div class="wallpaper"></div>

	<!-- Top Bar -->
	<TopBar
		on:toggleLauncher={toggleAppLauncher}
		on:toggleTaskManager={toggleTaskManager}
		on:openProfile={openProfile}
		on:openChangePassword={openChangePassword}
	/>

	<!-- App Launcher -->
	<AppLauncher visible={showAppLauncher} on:close={closeAppLauncher} />

	<!-- Task Manager -->
	<TaskManager visible={showTaskManager} on:close={closeTaskManager} />

	<!-- Bottom Dock -->
	<Dock />

	<!-- Desktop Area with icons -->
	<main class="desktop-area">
		<slot />
	</main>

	<!-- Window Manager -->
	<WindowManager />

	<!-- Notification Center -->
	<NotificationCenter bind:visible={showNotifications} />

	<!-- User Modals -->
	<ProfileModal bind:show={showProfileModal} />
	<ChangePasswordModal bind:show={showChangePasswordModal} />
	<UpdateSuccessModal
		bind:show={showUpdateModal}
		version={updateModalVersion}
		previousVersion={updateModalPreviousVersion}
		changelog={updateModalChangelog}
	/>

	<!-- Fullscreen Update Screen -->
	{#if $updateScreen.active}
		<UpdateScreen />
	{/if}

	<!-- Version label -->
	{#if $systemInfo?.version}
		<span class="version-label">PiNAS v{$systemInfo.version}</span>
	{/if}
</div>
{/if}

<style>
	.desktop {
		position: fixed;
		inset: 0;
		overflow: hidden;
	}

	.wallpaper {
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, #1a365d 0%, #2d3748 50%, #1a202c 100%);
		background-image: url('https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=1920&q=80');
		background-size: cover;
		background-position: center;
		z-index: 0;
	}

	.wallpaper::after {
		content: '';
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
	}

	.desktop-area {
		position: absolute;
		top: 40px;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 1;
	}

	.version-label {
		position: absolute;
		bottom: 72px;
		left: 12px;
		font-size: 11px;
		color: rgba(255, 255, 255, 0.35);
		font-weight: 500;
		z-index: 2;
		pointer-events: none;
		user-select: none;
	}

	.loading-screen {
		position: fixed;
		inset: 0;
		background: linear-gradient(135deg, #1e3a5f 0%, #0f172a 100%);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.loading-spinner {
		width: 48px;
		height: 48px;
		border: 3px solid rgba(255, 255, 255, 0.1);
		border-top-color: #3b82f6;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
