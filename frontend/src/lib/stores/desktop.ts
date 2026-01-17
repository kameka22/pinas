import { writable, derived } from 'svelte/store';

export interface DesktopApp {
	id: string;
	label: string;
	icon: string;
	component: string;
	gradient: string;
}

// Liste de toutes les applications disponibles
// TODO: Réactiver les autres applications quand elles seront implémentées
export const allApps: DesktopApp[] = [
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
	}
	// {
	// 	id: 'users',
	// 	label: 'User Management',
	// 	icon: 'mdi:account-group',
	// 	component: 'UserManager',
	// 	gradient: 'from-purple-400 to-purple-500'
	// },
	// {
	// 	id: 'storage',
	// 	label: 'Storage Manager',
	// 	icon: 'mdi:harddisk',
	// 	component: 'StorageManager',
	// 	gradient: 'from-slate-500 to-slate-600'
	// },
	// {
	// 	id: 'shares',
	// 	label: 'Shared Folders',
	// 	icon: 'mdi:folder-network',
	// 	component: 'ShareManager',
	// 	gradient: 'from-blue-400 to-blue-500'
	// },
	// {
	// 	id: 'docker',
	// 	label: 'Docker',
	// 	icon: 'mdi:docker',
	// 	component: 'Docker',
	// 	gradient: 'from-blue-500 to-blue-600'
	// },
	// {
	// 	id: 'terminal',
	// 	label: 'Terminal',
	// 	icon: 'mdi:console',
	// 	component: 'Terminal',
	// 	gradient: 'from-slate-600 to-slate-700'
	// },
	// {
	// 	id: 'settings',
	// 	label: 'Settings',
	// 	icon: 'mdi:cog',
	// 	component: 'Settings',
	// 	gradient: 'from-slate-400 to-slate-500'
	// }
];

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
export const desktopApps = derived(desktopStore, ($store) =>
	allApps.filter((app) => $store.pinnedAppIds.includes(app.id))
);

// Derived store pour les IDs épinglés
export const pinnedAppIds = derived(desktopStore, ($store) => $store.pinnedAppIds);

// Actions
export const { addToDesktop, removeFromDesktop, init: initDesktop } = desktopStore;
