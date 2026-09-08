import { writable } from 'svelte/store';

/** Global search palette (Ctrl+K) */
export const paletteVisible = writable(false);

export interface PaletteItem {
	id: string;
	kind: 'app' | 'settings' | 'user' | 'share' | 'file' | 'action';
	title: string;
	subtitle?: string;
	icon: string;
	run: () => void;
}

/** Simple fuzzy-ish score: prefix > word start > substring; 0 = no match */
export function scoreMatch(query: string, text: string): number {
	const q = query.trim().toLowerCase();
	const t = text.toLowerCase();
	if (!q) return 1;
	if (t === q) return 100;
	if (t.startsWith(q)) return 80;
	if (t.split(/[\s/_.-]+/).some((w) => w.startsWith(q))) return 60;
	const idx = t.indexOf(q);
	if (idx >= 0) return 40 - Math.min(idx, 30);
	return 0;
}
