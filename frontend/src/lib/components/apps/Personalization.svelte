<script lang="ts">
	import Icon from '@iconify/svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { onMount } from 'svelte';
	import { t } from '#lib/i18n/index.js';
	import { auth } from '#lib/stores/api.js';
	import { wallpaper, WALLPAPERS, setWallpaper, loadWallpaper } from '#lib/stores/wallpaper.js';
	import { ALL_WIDGETS, enabledWidgets, toggleWidget, loadWidgetPreferences, type WidgetId } from '#lib/stores/widgets.js';

	$: isAdmin = $auth.user?.role === 'admin';
	const widgetIcons: Record<WidgetId, string> = { system: 'mdi:chip', storage: 'mdi:harddisk', services: 'mdi:cog-outline', notifications: 'mdi:bell-outline', docker: 'mdi:docker' };

	onMount(() => { loadWallpaper(); loadWidgetPreferences(); });
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.personalization} description={$t.settingsUi.descriptions.personalization} icon="mdi:palette-outline" />

	<SettingsSection title={$t.personalization.wallpaper} description={$t.personalization.wallpaperHint} icon="mdi:image-outline">
		<div class="wallpapers">
			{#each WALLPAPERS as w (w.id)}
				<button class="wallpaper" class:selected={$wallpaper === w.id} style="background: {w.css}" on:click={() => setWallpaper(w.id)} aria-pressed={$wallpaper === w.id} title={w.label}>
					{#if $wallpaper === w.id}<Icon icon="mdi:check-circle" class="w-6 h-6" />{/if}
					<span>{w.label}</span>
				</button>
			{/each}
		</div>
	</SettingsSection>

	<SettingsSection title={$t.widgets.title} description={$t.personalization.widgetsHint} icon="mdi:widgets-outline">
		{#each ALL_WIDGETS as id}
			<SettingsRow label={($t.personalization.widgets as Record<string, string>)[id]}>
				<Icon icon={widgetIcons[id]} class="w-4 h-4 text-slate-400" />
				<SettingsToggle label={($t.personalization.widgets as Record<string, string>)[id]} checked={$enabledWidgets.includes(id)} disabled={id !== 'system' && !isAdmin} onchange={() => toggleWidget(id)} />
			</SettingsRow>
		{/each}
	</SettingsSection>
	<p class="settings-help">{$t.settingsUi.immediate}</p>
</div>

<style>
	.wallpapers { display: grid; grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr)); gap: 0.75rem; }
	.wallpaper { position: relative; height: 5.5rem; border-radius: 0.75rem; color: white; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.25rem; font-size: 0.75rem; font-weight: 500; border: 3px solid transparent; text-shadow: 0 1px 2px rgb(0 0 0 / 0.5); }
	.wallpaper.selected { border-color: #2563eb; }
	.wallpaper:focus-visible { outline: 2px solid #2563eb; outline-offset: 2px; }
</style>
