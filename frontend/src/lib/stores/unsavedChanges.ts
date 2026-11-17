import { derived, writable } from 'svelte/store';

/**
 * Registry of editors holding changes that are not saved yet. Each editor
 * registers under its own token, so several editors on screen do not clear
 * each other's state. Navigation (control panel sections) asks before leaving
 * while `hasUnsavedChanges` is true.
 */
const dirtyOwners = writable<ReadonlySet<symbol>>(new Set());

export const hasUnsavedChanges = derived(dirtyOwners, (owners) => owners.size > 0);

export function setUnsaved(owner: symbol, dirty: boolean): void {
	dirtyOwners.update((owners) => {
		if (owners.has(owner) === dirty) return owners;
		const next = new Set(owners);
		if (dirty) next.add(owner);
		else next.delete(owner);
		return next;
	});
}

/** Test helper and last-resort reset (e.g. after logout). */
export function clearUnsaved(): void {
	dirtyOwners.set(new Set());
}
