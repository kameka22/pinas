<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { auth } from '$stores/api';
	import { wallpaper, WALLPAPERS, setWallpaper, loadWallpaper } from '$stores/wallpaper';
	import { ALL_WIDGETS, enabledWidgets, toggleWidget, loadWidgetPreferences, type WidgetId } from '$stores/widgets';

	$: isAdmin = $auth.user?.role === 'admin';
	const widgetIcons: Record<WidgetId, string> = { system: 'mdi:chip', storage: 'mdi:harddisk', services: 'mdi:cog-outline', notifications: 'mdi:bell-outline', docker: 'mdi:docker' };

	onMount(() => { loadWallpaper(); loadWidgetPreferences(); });
</script>

<div class="perso">
	<section class="perso-section">
		<h3 class="perso-title"><Icon icon="mdi:image-outline" class="w-5 h-5" />{$t.personalization.wallpaper}</h3>
		<p class="perso-hint">{$t.personalization.wallpaperHint}</p>
		<div class="wallpapers">
			{#each WALLPAPERS as w (w.id)}
				<button class="wallpaper" class:selected={$wallpaper === w.id} style="background: {w.css}" on:click={() => setWallpaper(w.id)} title={w.label}>
					{#if $wallpaper === w.id}<Icon icon="mdi:check-circle" class="w-6 h-6" />{/if}
					<span>{w.label}</span>
				</button>
			{/each}
		</div>
	</section>

	<section class="perso-section">
		<h3 class="perso-title"><Icon icon="mdi:widgets-outline" class="w-5 h-5" />{$t.widgets.title}</h3>
		<p class="perso-hint">{$t.personalization.widgetsHint}</p>
		<div class="widget-toggles">
			{#each ALL_WIDGETS as id}
				<label class="widget-toggle" class:disabled={id !== 'system' && !isAdmin}>
					<input type="checkbox" checked={$enabledWidgets.includes(id)} disabled={id !== 'system' && !isAdmin} on:change={() => toggleWidget(id)} />
					<Icon icon={widgetIcons[id]} class="w-4 h-4" />
					<span>{($t.personalization.widgets as Record<string, string>)[id]}</span>
				</label>
			{/each}
		</div>
	</section>
</div>

<style>
	.perso { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; overflow-y: auto; height: 100%; }
	.perso-section { background: white; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1rem 1.25rem; }
	.perso-title { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9375rem; font-weight: 600; color: #0f172a; margin: 0 0 0.25rem; }
	.perso-hint { font-size: 0.75rem; color: #64748b; margin: 0 0 0.75rem; }
	.wallpapers { display: grid; grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr)); gap: 0.75rem; }
	.wallpaper { position: relative; height: 5.5rem; border-radius: 0.75rem; color: white; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.25rem; font-size: 0.75rem; font-weight: 500; border: 3px solid transparent; text-shadow: 0 1px 2px rgb(0 0 0 / 0.5); }
	.wallpaper.selected { border-color: #3b82f6; }
	.widget-toggles { display: flex; flex-direction: column; gap: 0.5rem; }
	.widget-toggle { display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #334155; }
	.widget-toggle.disabled { opacity: 0.5; }
</style>
