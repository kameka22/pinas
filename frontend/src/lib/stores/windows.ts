import { writable, derived } from 'svelte/store';

export interface WindowState {
	id: string;
	title: string;
	icon: string;
	component: string;
	x: number;
	y: number;
	width: number;
	height: number;
	minimized: boolean;
	maximized: boolean;
	zIndex: number;
	// Dynamic app config passed to component
	appConfig?: Record<string, unknown>;
	// Additional metadata
	gradient?: string;
}

interface WindowsStore {
	windows: WindowState[];
	activeWindowId: string | null;
	nextZIndex: number;
}

const initialState: WindowsStore = {
	windows: [],
	activeWindowId: null,
	nextZIndex: 100
};

function createWindowsStore() {
	const { subscribe, set, update } = writable<WindowsStore>(initialState);

	return {
		subscribe,

		openWindow: (config: Omit<WindowState, 'minimized' | 'maximized' | 'zIndex'> & { appConfig?: Record<string, unknown>; gradient?: string }) => {
			update((state) => {
				// Check if window already exists
				const existing = state.windows.find((w) => w.id === config.id);
				if (existing) {
					// Bring to front
					return {
						...state,
						activeWindowId: config.id,
						windows: state.windows.map((w) =>
							w.id === config.id
								? { ...w, minimized: false, zIndex: state.nextZIndex }
								: w
						),
						nextZIndex: state.nextZIndex + 1
					};
				}

				// Create new window
				const newWindow: WindowState = {
					...config,
					minimized: false,
					maximized: false,
					zIndex: state.nextZIndex
				};

				return {
					...state,
					windows: [...state.windows, newWindow],
					activeWindowId: config.id,
					nextZIndex: state.nextZIndex + 1
				};
			});
		},

		closeWindow: (id: string) => {
			update((state) => ({
				...state,
				windows: state.windows.filter((w) => w.id !== id),
				activeWindowId:
					state.activeWindowId === id
						? state.windows.find((w) => w.id !== id)?.id ?? null
						: state.activeWindowId
			}));
		},

		minimizeWindow: (id: string) => {
			update((state) => ({
				...state,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, minimized: true } : w
				)
			}));
		},

		maximizeWindow: (id: string) => {
			update((state) => ({
				...state,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, maximized: !w.maximized } : w
				)
			}));
		},

		focusWindow: (id: string) => {
			update((state) => ({
				...state,
				activeWindowId: id,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, zIndex: state.nextZIndex, minimized: false } : w
				),
				nextZIndex: state.nextZIndex + 1
			}));
		},

		updateWindowPosition: (id: string, x: number, y: number) => {
			update((state) => ({
				...state,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, x, y } : w
				)
			}));
		},

		updateWindowSize: (id: string, width: number, height: number) => {
			update((state) => ({
				...state,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, width, height } : w
				)
			}));
		},

		restoreWindow: (id: string) => {
			update((state) => ({
				...state,
				windows: state.windows.map((w) =>
					w.id === id ? { ...w, minimized: false, zIndex: state.nextZIndex } : w
				),
				activeWindowId: id,
				nextZIndex: state.nextZIndex + 1
			}));
		}
	};
}

export const windowsStore = createWindowsStore();

// Derived stores
export const windows = derived(windowsStore, ($store) => $store.windows);
export const activeWindowId = derived(windowsStore, ($store) => $store.activeWindowId);
export const openWindows = derived(windowsStore, ($store) =>
	$store.windows.filter((w) => !w.minimized)
);

// Actions
export const {
	openWindow,
	closeWindow,
	minimizeWindow,
	maximizeWindow,
	focusWindow,
	updateWindowPosition,
	updateWindowSize,
	restoreWindow
} = windowsStore;
