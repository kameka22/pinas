<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import TopBar from '$components/desktop/TopBar.svelte';
	import Dock from '$components/desktop/Dock.svelte';
	import WindowManager from '$components/desktop/WindowManager.svelte';
	import NotificationCenter from '$components/desktop/NotificationCenter.svelte';
	import AppLauncher from '$components/desktop/AppLauncher.svelte';
	import { connectWebSocket } from '$stores/websocket';

	let showNotifications = false;
	let showAppLauncher = false;

	function toggleAppLauncher() {
		showAppLauncher = !showAppLauncher;
	}

	function closeAppLauncher() {
		showAppLauncher = false;
	}

	onMount(() => {
		const disconnect = connectWebSocket();
		return () => disconnect();
	});
</script>

<div class="desktop">
	<!-- Wallpaper Background -->
	<div class="wallpaper"></div>

	<!-- Top Bar -->
	<TopBar on:toggleLauncher={toggleAppLauncher} />

	<!-- App Launcher -->
	<AppLauncher visible={showAppLauncher} on:close={closeAppLauncher} />

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
</div>

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
</style>
