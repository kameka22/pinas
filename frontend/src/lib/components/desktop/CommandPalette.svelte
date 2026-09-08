<script lang="ts">
	import Icon from '@iconify/svelte';
	import { tick } from 'svelte';
	import { t, translate } from '$lib/i18n';
	import { auth, api, type BrowsableLocation } from '$stores/api';
	import { allApps, type DesktopApp } from '$stores/desktop';
	import { openWindow } from '$stores/windows';
	import { paletteVisible, scoreMatch, type PaletteItem } from '$stores/palette';
	import { widgetsVisible } from '$stores/widgets';

	// Control Panel sections reachable from the palette (id -> i18n label key)
	const SETTINGS: { id: string; labelKey: string; icon: string; adminOnly?: boolean }[] = [
		{ id: 'users', labelKey: 'controlPanel.items.userManagement', icon: 'mdi:account-multiple', adminOnly: true },
		{ id: 'files', labelKey: 'controlPanel.items.fileService', icon: 'mdi:folder', adminOnly: true },
		{ id: 'device', labelKey: 'controlPanel.items.deviceConnection', icon: 'mdi:monitor-screenshot', adminOnly: true },
		{ id: 'terminal', labelKey: 'controlPanel.items.terminal', icon: 'mdi:console-line', adminOnly: true },
		{ id: 'printer', labelKey: 'controlPanel.items.printer', icon: 'mdi:printer', adminOnly: true },
		{ id: 'hardware', labelKey: 'controlPanel.items.hardwarePower', icon: 'mdi:chip', adminOnly: true },
		{ id: 'time', labelKey: 'controlPanel.items.timeLanguage', icon: 'mdi:earth' },
		{ id: 'network', labelKey: 'controlPanel.items.network', icon: 'mdi:wifi', adminOnly: true },
		{ id: 'security', labelKey: 'controlPanel.items.security', icon: 'mdi:shield-check', adminOnly: true },
		{ id: 'personalization', labelKey: 'controlPanel.items.personalization', icon: 'mdi:palette' },
		{ id: 'update', labelKey: 'controlPanel.items.systemUpdate', icon: 'mdi:update', adminOnly: true },
		{ id: 'about', labelKey: 'controlPanel.items.about', icon: 'mdi:information' }
	];

	let query = '';
	let input: HTMLInputElement;
	let selected = 0;
	let remote: PaletteItem[] = [];
	let searching = false;
	let searchSeq = 0;
	let locations: BrowsableLocation[] = [];

	$: isAdmin = $auth.user?.role === 'admin';

	function appLabel(app: DesktopApp): string {
		return app.labelKey ? translate($t, `apps.${app.labelKey}`) : app.label;
	}

	function launch(app: DesktopApp, appConfig?: Record<string, unknown>) {
		openWindow({
			id: appConfig ? `${app.id}-${Date.now()}` : app.id,
			title: appLabel(app),
			icon: app.icon,
			component: app.component,
			x: 120 + Math.random() * 80,
			y: 80 + Math.random() * 40,
			width: app.window?.width ?? 1000,
			height: app.window?.height ?? 650,
			gradient: app.gradient,
			appConfig
		});
		close();
	}

	$: localItems = (() => {
		const items: PaletteItem[] = [];
		for (const app of $allApps) {
			items.push({ id: `app:${app.id}`, kind: 'app', title: appLabel(app), subtitle: $t.palette.openApp, icon: app.icon, run: () => launch(app) });
		}
		const cp = $allApps.find((a) => a.id === 'control-panel');
		if (cp) {
			for (const s of SETTINGS) {
				if (s.adminOnly && !isAdmin) continue;
				items.push({ id: `settings:${s.id}`, kind: 'settings', title: translate($t, s.labelKey), subtitle: $t.apps.controlPanel, icon: s.icon, run: () => launch(cp, { section: s.id }) });
			}
		}
		items.push({ id: 'action:widgets', kind: 'action', title: $t.widgets.title, subtitle: $t.palette.action, icon: 'mdi:widgets-outline', run: () => { widgetsVisible.set(true); close(); } });
		return items;
	})();

	$: results = [
		...localItems.map((i) => ({ item: i, score: scoreMatch(query, i.title) })).filter((r) => r.score > 0).sort((a, b) => b.score - a.score).map((r) => r.item),
		...remote
	].slice(0, 30);

	$: if (selected >= results.length) selected = Math.max(0, results.length - 1);

	async function remoteSearch(q: string) {
		const seq = ++searchSeq;
		if (q.trim().length < 2) { remote = []; return; }
		searching = true;
		const items: PaletteItem[] = [];
		const fm = $allApps.find((a) => a.id === 'file-manager');
		const cp = $allApps.find((a) => a.id === 'control-panel');
		const tasks: Promise<void>[] = [];
		if (isAdmin && cp) {
			tasks.push(api.getUsers().then((users) => {
				for (const u of users) if (scoreMatch(q, u.username) > 0) items.push({ id: `user:${u.id}`, kind: 'user', title: u.username, subtitle: $t.palette.user, icon: 'mdi:account', run: () => launch(cp, { section: 'users' }) });
			}).catch(() => {}));
			tasks.push(api.getShares().then((shares) => {
				for (const s of shares) if (scoreMatch(q, s.name) > 0 || scoreMatch(q, s.path) > 0) items.push({ id: `share:${s.id}`, kind: 'share', title: s.name, subtitle: `${s.share_type.toUpperCase()} · ${s.path}`, icon: 'mdi:folder-network', run: () => launch(cp, { section: 'files' }) });
			}).catch(() => {}));
		}
		if (fm) {
			if (locations.length === 0) { try { locations = await api.getLocations(); } catch { locations = []; } }
			for (const loc of locations.slice(0, 6)) {
				tasks.push(api.searchFiles(q, loc.id, 15).then((files) => {
					for (const f of files) items.push({
						id: `file:${loc.id}:${f.path}`, kind: 'file', title: f.name, subtitle: `${loc.name} / ${f.path}`,
						icon: f.type === 'folder' ? 'mdi:folder' : 'mdi:file-outline',
						run: () => launch(fm, { locationId: loc.id, path: f.type === 'folder' ? f.path : f.path.split('/').slice(0, -1).join('/') })
					});
				}).catch(() => {}));
			}
		}
		await Promise.allSettled(tasks);
		if (seq === searchSeq) { remote = items.slice(0, 40); searching = false; }
	}

	let debounce: ReturnType<typeof setTimeout> | null = null;
	function onInput() {
		selected = 0;
		if (debounce) clearTimeout(debounce);
		debounce = setTimeout(() => remoteSearch(query), 250);
	}

	function close() {
		paletteVisible.set(false);
		query = '';
		remote = [];
		selected = 0;
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') { close(); return; }
		if (e.key === 'ArrowDown') { e.preventDefault(); selected = Math.min(selected + 1, results.length - 1); }
		if (e.key === 'ArrowUp') { e.preventDefault(); selected = Math.max(selected - 1, 0); }
		if (e.key === 'Enter' && results[selected]) { e.preventDefault(); results[selected].run(); }
	}

	function onGlobalKey(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k' && $auth.isAuthenticated) {
			e.preventDefault();
			paletteVisible.update((v) => !v);
		}
	}

	$: if ($paletteVisible) tick().then(() => input?.focus());
</script>

<svelte:window on:keydown={onGlobalKey} />

{#if $paletteVisible}
	<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
	<div class="palette-overlay" on:click={close}>
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div class="palette" on:click|stopPropagation role="dialog" tabindex="-1" aria-label={$t.common.search}>
			<div class="palette-input">
				<Icon icon={searching ? 'mdi:loading' : 'mdi:magnify'} class="w-5 h-5 text-slate-400 {searching ? 'animate-spin' : ''}" />
				<input bind:this={input} bind:value={query} on:input={onInput} on:keydown={onKey} placeholder={$t.palette.placeholder} autocomplete="off" spellcheck="false" />
				<kbd>Esc</kbd>
			</div>
			<ul class="palette-results" role="listbox">
				{#if results.length === 0}
					<li class="palette-empty">{query.trim().length < 2 ? $t.palette.hint : $t.palette.noResults}</li>
				{/if}
				{#each results as item, i (item.id)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<li role="option" aria-selected={i === selected} class:selected={i === selected} on:mouseenter={() => (selected = i)} on:click={item.run}>
						<Icon icon={item.icon} class="w-5 h-5 shrink-0" />
						<div class="palette-text">
							<span class="palette-title">{item.title}</span>
							{#if item.subtitle}<span class="palette-sub">{item.subtitle}</span>{/if}
						</div>
						<span class="palette-kind">{($t.palette.kinds as Record<string, string>)[item.kind] || item.kind}</span>
					</li>
				{/each}
			</ul>
		</div>
	</div>
{/if}

<style>
	.palette-overlay { position: fixed; inset: 0; z-index: 9500; background: rgb(15 23 42 / 0.45); display: flex; justify-content: center; align-items: flex-start; padding-top: 12vh; }
	.palette { width: min(640px, 92vw); background: white; border-radius: 1rem; box-shadow: 0 30px 80px rgb(0 0 0 / 0.35); overflow: hidden; }
	.palette-input { display: flex; align-items: center; gap: 0.625rem; padding: 0.875rem 1rem; border-bottom: 1px solid #e2e8f0; }
	.palette-input input { flex: 1; border: 0; outline: 0; font-size: 1rem; background: transparent; color: #0f172a; }
	kbd { font-size: 0.6875rem; color: #64748b; border: 1px solid #cbd5e1; border-radius: 0.25rem; padding: 0.05rem 0.35rem; }
	.palette-results { list-style: none; margin: 0; padding: 0.375rem; max-height: 55vh; overflow-y: auto; }
	.palette-results li { display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; border-radius: 0.625rem; cursor: pointer; color: #334155; }
	.palette-results li.selected { background: #eef2ff; color: #1e293b; }
	.palette-text { flex: 1; display: flex; flex-direction: column; min-width: 0; }
	.palette-title { font-size: 0.875rem; }
	.palette-sub { font-size: 0.75rem; color: #64748b; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.palette-kind { font-size: 0.6875rem; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.03em; }
	.palette-empty { padding: 1.25rem; text-align: center; font-size: 0.8125rem; color: #94a3b8; cursor: default; }
</style>
