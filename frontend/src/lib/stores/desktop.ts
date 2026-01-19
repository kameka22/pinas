import { writable, derived, get } from 'svelte/store';

export interface DesktopApp {
	id: string;
	label: string;
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
}

// Built-in system apps (always available)
export const builtInApps: DesktopApp[] = [
	{
		id: 'control-panel',
		label: 'Control Panel',
		icon: 'mdi:tune-variant',
		component: 'ControlPanel',
		gradient: 'from-slate-500 to-slate-600'
	},
	{
		id: 'file-manager',
		label: 'Files',
		icon: 'mdi:folder',
		component: 'FileManager',
		gradient: 'from-amber-400 to-amber-500'
	},
	{
		id: 'app-center',
		label: 'App Center',
		icon: 'mdi:store',
		component: 'AppCenter',
		gradient: 'from-purple-500 to-pink-500'
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
			isInstalled: true
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

interface DesktopStore {
	pinnedAppIds: string[];
}

const STORAGE_KEY = 'pinas-desktop-icons';

function loadFromStorage(): string[] {
	if (typeof window === 'undefined') return ['control-panel', 'file-manager'];
	const stored = localStorage.getItem(STORAGE_KEY);
	if (stored) {
		try {
			return JSON.parse(stored);
		} catch {
			return ['control-panel', 'file-manager'];
		}
	}
	return ['control-panel', 'file-manager']; // Par défaut, Control Panel et Files sont sur le bureau
}

function saveToStorage(pinnedIds: string[]) {
	if (typeof window === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(pinnedIds));
}

function createDesktopStore() {
	const { subscribe, set, update } = writable<DesktopStore>({
		pinnedAppIds: loadFromStorage()
	});

	return {
		subscribe,

		init: () => {
			set({ pinnedAppIds: loadFromStorage() });
		},

		addToDesktop: (appId: string) => {
			update((state) => {
				if (state.pinnedAppIds.includes(appId)) return state;
				const newPinned = [...state.pinnedAppIds, appId];
				saveToStorage(newPinned);
				return { pinnedAppIds: newPinned };
			});
		},

		removeFromDesktop: (appId: string) => {
			update((state) => {
				const newPinned = state.pinnedAppIds.filter((id) => id !== appId);
				saveToStorage(newPinned);
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
