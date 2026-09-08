import { writable } from 'svelte/store';
import { api } from './api';

export interface WallpaperPreset {
	id: string;
	label: string;
	css: string;
}

/** Bundled backgrounds: gradients only, nothing fetched from the internet */
export const WALLPAPERS: WallpaperPreset[] = [
	{ id: 'ocean', label: 'Ocean', css: 'linear-gradient(135deg, #0f2027 0%, #203a43 50%, #2c5364 100%)' },
	{ id: 'dusk', label: 'Dusk', css: 'linear-gradient(135deg, #1a365d 0%, #2d3748 50%, #1a202c 100%)' },
	{ id: 'aurora', label: 'Aurora', css: 'linear-gradient(135deg, #0b3d2e 0%, #134e4a 45%, #1e3a5f 100%)' },
	{ id: 'plum', label: 'Plum', css: 'linear-gradient(135deg, #2e1065 0%, #4c1d95 50%, #1e1b4b 100%)' },
	{ id: 'ember', label: 'Ember', css: 'linear-gradient(135deg, #3b0a0a 0%, #7c2d12 55%, #1c1917 100%)' },
	{ id: 'slate', label: 'Slate', css: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)' }
];
const PREF_KEY = 'ui.wallpaper';
const LOCAL_KEY = 'pinas-wallpaper';

function initial(): string {
	try { return localStorage.getItem(LOCAL_KEY) || 'ocean'; } catch { return 'ocean'; }
}

export const wallpaper = writable<string>(initial());

export function wallpaperCss(id: string): string {
	return (WALLPAPERS.find((w) => w.id === id) || WALLPAPERS[0]).css;
}

export async function loadWallpaper(): Promise<void> {
	try {
		const pref = await api.getPreference(PREF_KEY);
		if (pref?.value && WALLPAPERS.some((w) => w.id === pref.value)) setWallpaper(pref.value, false);
	} catch {
		// keep local/default
	}
}

export function setWallpaper(id: string, persist = true): void {
	if (!WALLPAPERS.some((w) => w.id === id)) return;
	wallpaper.set(id);
	try { localStorage.setItem(LOCAL_KEY, id); } catch { /* ignore */ }
	if (persist) api.setPreference(PREF_KEY, id).catch(() => {});
}
