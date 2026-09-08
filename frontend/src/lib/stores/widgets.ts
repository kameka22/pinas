import { writable, get } from 'svelte/store';
import { api } from './api';

export type WidgetId = 'system' | 'storage' | 'services' | 'notifications' | 'docker';
export const ALL_WIDGETS: WidgetId[] = ['system', 'storage', 'services', 'notifications', 'docker'];
const PREF_KEY = 'widgets.enabled';

export const widgetsVisible = writable(false);
export const enabledWidgets = writable<WidgetId[]>([...ALL_WIDGETS]);

export async function loadWidgetPreferences(): Promise<void> {
	try {
		const pref = await api.getPreference(PREF_KEY);
		const parsed = JSON.parse(pref.value);
		if (Array.isArray(parsed)) enabledWidgets.set(parsed.filter((w): w is WidgetId => ALL_WIDGETS.includes(w)));
	} catch {
		// keep defaults
	}
}

export function toggleWidget(id: WidgetId): void {
	const next = get(enabledWidgets).includes(id)
		? get(enabledWidgets).filter((w) => w !== id)
		: [...get(enabledWidgets), id];
	enabledWidgets.set(ALL_WIDGETS.filter((w) => next.includes(w)));
	api.setPreference(PREF_KEY, JSON.stringify(get(enabledWidgets))).catch(() => {});
}
