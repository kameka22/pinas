import { writable, derived, get } from 'svelte/store';
import { api } from './api';

export interface DesktopApp {
	id: string;
	label: string;
	labelKey?: string; // i18n key for translation (apps.xxx)
	icon: string;
	component: string;
	gradient: string;
	window?: {
		width: number;
		height: number;
		minWidth: number;
		minHeight: number;
	};
	isInstalled?: boolean; // true for dynamically installed apps
	appConfig?: Record<string, unknown>; // Component-specific config (e.g., port, path for IframeApp)
}

export interface AppRegistryEntry {
	id: string;
	name: string;
	icon: string;
	gradient: string;
	component: string;
	window: {
		width: number;
		height: number;
		min_width: number;
		min_height: number;
	};
	config?: Record<string, unknown>; // Component-specific config
}

// Built-in system apps (always available)
// labelKey corresponds to keys in $t.apps (e.g., 'controlPanel' -> $t.apps.controlPanel)
export const builtInApps: DesktopApp[] = [
	{
		id: 'control-panel',
		label: 'Control Panel',
		labelKey: 'controlPanel',
		icon: 'mdi:tune-variant',
		component: 'ControlPanel',
		gradient: 'from-slate-500 to-slate-600'
	},
	{
		id: 'file-manager',
		label: 'Files',
		labelKey: 'files',
		icon: 'mdi:folder',
		component: 'FileManager',
		gradient: 'from-amber-400 to-amber-500'
	},
	{
		id: 'app-center',
		label: 'App Center',
		labelKey: 'appCenter',
		icon: 'mdi:store',
		component: 'AppCenter',
		gradient: 'from-purple-500 to-pink-500'
	},
	{
		id: 'storage',
		label: 'Storage',
		labelKey: 'storage',
		icon: 'mdi:harddisk',
		component: 'StorageManager',
		gradient: 'from-slate-500 to-slate-600'
	},
	{
		id: 'shares',
		label: 'Shares',
		labelKey: 'shares',
		icon: 'mdi:folder-network',
		component: 'ShareManager',
		gradient: 'from-blue-400 to-blue-500'
	},
	{
		id: 'terminal',
		label: 'Terminal',
		labelKey: 'terminal',
		icon: 'mdi:console',
		component: 'TerminalApp',
		gradient: 'from-gray-700 to-gray-800'
	},
	{
		id: 'process-manager',
		label: 'Process Manager',
		labelKey: 'processManager',
		icon: 'mdi:chart-timeline-variant',
		component: 'ProcessManager',
		gradient: 'from-emerald-500 to-teal-600'
	},
	{
		id: 'display',
		label: 'External Display',
		labelKey: 'display',
		icon: 'mdi:monitor-screenshot',
		component: 'DisplayApp',
		gradient: 'from-indigo-500 to-purple-500'
	}
];

// Store for dynamically installed apps (loaded from API)
export const installedApps = writable<DesktopApp[]>([]);

// Combined list of all apps (built-in + installed)
export const allApps = derived(installedApps, ($installedApps) => {
	return [...builtInApps, ...$installedApps];
});

// Fetch installed apps from the backend registry
export async function loadInstalledApps(): Promise<void> {
	try {
		const response = await fetch('/api/apps/registry');
		if (!response.ok) {
			console.warn('Failed to load app registry:', response.statusText);
			return;
		}
		const registry: AppRegistryEntry[] = await response.json();

		const apps: DesktopApp[] = registry.map((entry) => ({
			id: entry.id,
			label: entry.name,
			icon: entry.icon,
			gradient: entry.gradient,
			component: entry.component,
			window: {
				width: entry.window.width,
				height: entry.window.height,
				minWidth: entry.window.min_width,
				minHeight: entry.window.min_height
			},
			isInstalled: true,
			appConfig: entry.config
		}));

		installedApps.set(apps);
	} catch (error) {
		console.warn('Failed to load installed apps:', error);
	}
}

// Check if an app is installed (either built-in or from registry)
export function isAppAvailable(appId: string): boolean {
	const $allApps = get(allApps);
	return $allApps.some((app) => app.id === appId);
}

// Get app by ID
export function getAppById(appId: string): DesktopApp | undefined {
	const $allApps = get(allApps);
	return $allApps.find((app) => app.id === appId);
}

// ==================== Desktop Icons Store ====================

const DEFAULT_DESKTOP_ICONS = ['control-panel', 'file-manager', 'app-center'];
const DESKTOP_STORAGE_KEY = 'pinas-desktop-icons';

interface DesktopStore {
	pinnedAppIds: string[];
}

function loadDesktopFromLocalStorage(): string[] {
	if (typeof window === 'undefined') return DEFAULT_DESKTOP_ICONS;
	const stored = localStorage.getItem(DESKTOP_STORAGE_KEY);
	if (stored) {
		try {
			return JSON.parse(stored);
		} catch {
			return DEFAULT_DESKTOP_ICONS;
		}
	}
	return DEFAULT_DESKTOP_ICONS;
}

function saveDesktopToLocalStorage(pinnedIds: string[]) {
	if (typeof window === 'undefined') return;
	localStorage.setItem(DESKTOP_STORAGE_KEY, JSON.stringify(pinnedIds));
}

function persistDesktopToApi(pinnedIds: string[]) {
	api.setPreference('desktop_icons', JSON.stringify(pinnedIds)).catch((e) => {
		console.warn('Failed to persist desktop icons to API:', e);
	});
}

function createDesktopStore() {
	const { subscribe, set, update } = writable<DesktopStore>({
		pinnedAppIds: loadDesktopFromLocalStorage()
	});

	return {
		subscribe,

		init: async () => {
			try {
				const pref = await api.getPreference('desktop_icons');
				const ids: string[] = JSON.parse(pref.value);
				set({ pinnedAppIds: ids });
				saveDesktopToLocalStorage(ids);
			} catch {
				// API not available or pref not found — use localStorage fallback
				set({ pinnedAppIds: loadDesktopFromLocalStorage() });
			}
		},

		addToDesktop: (appId: string) => {
			update((state) => {
				if (state.pinnedAppIds.includes(appId)) return state;
				const newPinned = [...state.pinnedAppIds, appId];
				saveDesktopToLocalStorage(newPinned);
				persistDesktopToApi(newPinned);
				return { pinnedAppIds: newPinned };
			});
		},

		removeFromDesktop: (appId: string) => {
			update((state) => {
				const newPinned = state.pinnedAppIds.filter((id) => id !== appId);
				saveDesktopToLocalStorage(newPinned);
				persistDesktopToApi(newPinned);
				return { pinnedAppIds: newPinned };
			});
		},

		isOnDesktop: (appId: string, pinnedIds: string[]): boolean => {
			return pinnedIds.includes(appId);
		}
	};
}

export const desktopStore = createDesktopStore();

// Derived store pour les applications épinglées sur le bureau
// Must derive from both desktopStore and allApps since allApps is now a derived store
export const desktopApps = derived(
	[desktopStore, allApps],
	([$store, $allApps]) => $allApps.filter((app) => $store.pinnedAppIds.includes(app.id))
);

// Derived store pour les IDs épinglés
export const pinnedAppIds = derived(desktopStore, ($store) => $store.pinnedAppIds);

// Actions
export const { addToDesktop, removeFromDesktop, init: initDesktop } = desktopStore;

// ==================== Dock Store ====================

const DEFAULT_DOCK_ITEMS = ['file-manager', 'app-center', 'control-panel'];
const DOCK_STORAGE_KEY = 'pinas-dock-items';

function loadDockFromLocalStorage(): string[] {
	if (typeof window === 'undefined') return DEFAULT_DOCK_ITEMS;
	const stored = localStorage.getItem(DOCK_STORAGE_KEY);
	if (stored) {
		try {
			return JSON.parse(stored);
		} catch {
			return DEFAULT_DOCK_ITEMS;
		}
	}
	return DEFAULT_DOCK_ITEMS;
}

function saveDockToLocalStorage(ids: string[]) {
	if (typeof window === 'undefined') return;
	localStorage.setItem(DOCK_STORAGE_KEY, JSON.stringify(ids));
}

function persistDockToApi(ids: string[]) {
	api.setPreference('dock_items', JSON.stringify(ids)).catch((e) => {
		console.warn('Failed to persist dock items to API:', e);
	});
}

export const dockPinnedIds = writable<string[]>(loadDockFromLocalStorage());

export async function initDock() {
	try {
		const pref = await api.getPreference('dock_items');
		const ids: string[] = JSON.parse(pref.value);
		dockPinnedIds.set(ids);
		saveDockToLocalStorage(ids);
	} catch {
		// API not available or pref not found — use localStorage fallback
		dockPinnedIds.set(loadDockFromLocalStorage());
	}
}

export function addToDock(appId: string) {
	dockPinnedIds.update((ids) => {
		if (ids.includes(appId)) return ids;
		const newIds = [...ids, appId];
		saveDockToLocalStorage(newIds);
		persistDockToApi(newIds);
		return newIds;
	});
}

export function removeFromDock(appId: string) {
	dockPinnedIds.update((ids) => {
		const newIds = ids.filter((id) => id !== appId);
		saveDockToLocalStorage(newIds);
		persistDockToApi(newIds);
		return newIds;
	});
}
