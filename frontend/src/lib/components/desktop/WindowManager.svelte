<script lang="ts">
	import { windows, type WindowState } from '#lib/stores/windows.js';
	import Window from './Window.svelte';
	import { getAppComponent } from '#lib/components/apps/index.js';

	// Get component and prepare props for a window
	function getComponentForWindow(window: WindowState) {
		return getAppComponent(window.component);
	}

	// Build props to pass to the component
	function getComponentProps(window: WindowState): Record<string, unknown> {
		const props: Record<string, unknown> = {};

		// Pass config if present
		if (window.appConfig) {
			props.config = window.appConfig;
		}

		// Pass metadata for generic components
		if (window.title) {
			props.name = window.title;
		}
		if (window.icon) {
			props.icon = window.icon;
		}
		if (window.gradient) {
			props.gradient = window.gradient;
		}

		return props;
	}
</script>

{#each $windows as win (win.id)}
	{#if !win.minimized}
		<Window window={win}>
			<svelte:component this={getComponentForWindow(win)} {...getComponentProps(win)} />
		</Window>
	{/if}
{/each}
