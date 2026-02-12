import { writable } from 'svelte/store';

export interface UpdateScreenState {
	active: boolean;
	taskId: string | null;
	rebootRequired: boolean;
	version: string;
	changelog: Record<string, string> | null;
	devTest: boolean;
}

export const updateScreen = writable<UpdateScreenState>({
	active: false,
	taskId: null,
	rebootRequired: false,
	version: '',
	changelog: null,
	devTest: false
});
