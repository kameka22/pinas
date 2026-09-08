import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { windows, activeWindowId, openWindows, openWindow, closeWindow, minimizeWindow, restoreWindow, focusWindow, maximizeWindow } from './windows';

function open(id: string) {
	openWindow({ id, title: id, icon: 'mdi:apps', component: 'Dashboard', x: 10, y: 10, width: 400, height: 300 });
}

beforeEach(() => {
	for (const w of get(windows)) closeWindow(w.id);
});

describe('windows store', () => {
	it('opens, focuses and closes windows', () => {
		open('a');
		open('b');
		expect(get(windows).map((w) => w.id)).toEqual(['a', 'b']);
		expect(get(activeWindowId)).toBe('b');
		closeWindow('b');
		expect(get(windows).map((w) => w.id)).toEqual(['a']);
	});

	it('re-opening an existing id focuses it instead of duplicating', () => {
		open('a');
		open('b');
		open('a');
		expect(get(windows)).toHaveLength(2);
		expect(get(activeWindowId)).toBe('a');
	});

	it('focus raises the z-index above the others', () => {
		open('a');
		open('b');
		focusWindow('a');
		const [a, b] = ['a', 'b'].map((id) => get(windows).find((w) => w.id === id)!);
		expect(a.zIndex).toBeGreaterThan(b.zIndex);
	});

	it('minimize hides from openWindows and restore brings it back', () => {
		open('a');
		minimizeWindow('a');
		expect(get(openWindows).map((w) => w.id)).not.toContain('a');
		expect(get(windows).find((w) => w.id === 'a')?.minimized).toBe(true);
		restoreWindow('a');
		expect(get(openWindows).map((w) => w.id)).toContain('a');
	});

	it('maximize toggles the flag', () => {
		open('a');
		maximizeWindow('a');
		expect(get(windows).find((w) => w.id === 'a')?.maximized).toBe(true);
		maximizeWindow('a');
		expect(get(windows).find((w) => w.id === 'a')?.maximized).toBe(false);
	});
});
