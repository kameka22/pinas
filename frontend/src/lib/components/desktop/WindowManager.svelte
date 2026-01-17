<script lang="ts">
	import { windows } from '$stores/windows';
	import Window from './Window.svelte';

	// App components
	import Dashboard from '$components/apps/Dashboard.svelte';
	import StorageManager from '$components/apps/StorageManager.svelte';
	import ShareManager from '$components/apps/ShareManager.svelte';
	import UserManager from '$components/apps/UserManager.svelte';
	import Settings from '$components/apps/Settings.svelte';
	import ControlPanel from '$components/apps/ControlPanel.svelte';
	import FileManager from '$components/apps/FileManager.svelte';

	// Component mapping
	const components: Record<string, any> = {
		Dashboard,
		StorageManager,
		ShareManager,
		UserManager,
		Settings,
		ControlPanel,
		FileManager,
		// Placeholder components for others
		NetdiskTools: Dashboard,
		Docker: Dashboard,
		Support: Dashboard,
		TaskManager: Dashboard
	};

	function getComponent(name: string) {
		return components[name] || Dashboard;
	}
</script>

{#each $windows as window (window.id)}
	{#if !window.minimized}
		<Window {window}>
			<svelte:component this={getComponent(window.component)} />
		</Window>
	{/if}
{/each}
