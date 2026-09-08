import { writable } from 'svelte/store';

export type ToastKind = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
	id: number;
	kind: ToastKind;
	message: string;
	/** ms; 0 = sticky until dismissed */
	timeout: number;
}

const DEFAULT_TIMEOUT: Record<ToastKind, number> = {
	success: 4000,
	info: 5000,
	warning: 7000,
	error: 8000
};

let nextId = 1;
const { subscribe, update } = writable<Toast[]>([]);

function push(kind: ToastKind, message: string, timeout = DEFAULT_TIMEOUT[kind]): number {
	const id = nextId++;
	update((list) => [...list, { id, kind, message, timeout }]);
	if (timeout > 0 && typeof setTimeout !== 'undefined') {
		setTimeout(() => dismiss(id), timeout);
	}
	return id;
}

function dismiss(id: number) {
	update((list) => list.filter((t) => t.id !== id));
}

function clear() {
	update(() => []);
}

/** Human-readable message for anything thrown by the API client or a component */
export function errorMessage(e: unknown, fallback: string): string {
	if (e instanceof Error && e.message) return e.message;
	if (typeof e === 'string' && e) return e;
	return fallback;
}

export const toasts = {
	subscribe,
	push,
	dismiss,
	clear,
	success: (message: string, timeout?: number) => push('success', message, timeout),
	error: (message: string, timeout?: number) => push('error', message, timeout),
	info: (message: string, timeout?: number) => push('info', message, timeout),
	warning: (message: string, timeout?: number) => push('warning', message, timeout)
};
