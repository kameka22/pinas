import { writable } from 'svelte/store';

export interface PowerScreenState {
	active: boolean;
	action: 'restart' | 'shutdown';
}

export const powerScreen = writable<PowerScreenState>({
	active: false,
	action: 'restart'
});
