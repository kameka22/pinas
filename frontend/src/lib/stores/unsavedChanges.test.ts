import { describe, expect, it, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { clearUnsaved, hasUnsavedChanges, setUnsaved } from './unsavedChanges';

describe('unsavedChanges', () => {
	beforeEach(clearUnsaved);

	it('is clean by default', () => {
		expect(get(hasUnsavedChanges)).toBe(false);
	});

	it('tracks each editor independently', () => {
		const a = Symbol('a');
		const b = Symbol('b');
		setUnsaved(a, true);
		setUnsaved(b, true);
		setUnsaved(a, false);
		expect(get(hasUnsavedChanges)).toBe(true);
		setUnsaved(b, false);
		expect(get(hasUnsavedChanges)).toBe(false);
	});

	it('ignores repeated updates for the same state', () => {
		const a = Symbol('a');
		setUnsaved(a, false);
		expect(get(hasUnsavedChanges)).toBe(false);
		setUnsaved(a, true);
		setUnsaved(a, true);
		setUnsaved(a, false);
		expect(get(hasUnsavedChanges)).toBe(false);
	});
});
