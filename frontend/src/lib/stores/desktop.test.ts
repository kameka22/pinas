import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { auth } from './api';
import { allApps, builtInApps, getAppById, installedApps } from './desktop';

beforeEach(() => {
	installedApps.set([]);
});

describe('desktop apps visibility', () => {
	it('admins see every built-in app', () => {
		auth.set({ isAuthenticated: true, user: { id: '1', username: 'root', role: 'admin' } });
		expect(get(allApps).map((a) => a.id)).toEqual(builtInApps.map((a) => a.id));
	});

	it('regular users do not see admin-only apps', () => {
		auth.set({ isAuthenticated: true, user: { id: '2', username: 'bob', role: 'user' } });
		const ids = get(allApps).map((a) => a.id);
		expect(ids).toContain('file-manager');
		expect(ids).toContain('control-panel');
		for (const hidden of ['storage', 'app-center', 'terminal', 'process-manager', 'display', 'shares']) {
			expect(ids, hidden).not.toContain(hidden);
		}
	});

	it('installed apps are appended and visible to everyone', () => {
		auth.set({ isAuthenticated: true, user: { id: '2', username: 'bob', role: 'user' } });
		installedApps.set([{ id: 'jellyfin', label: 'Jellyfin', icon: 'mdi:play', component: 'Iframe', gradient: 'g', isInstalled: true }]);
		expect(get(allApps).some((a) => a.id === 'jellyfin')).toBe(true);
	});

	it('getAppById finds built-in apps', () => {
		expect(getAppById('file-manager')?.component).toBe('FileManager');
		expect(getAppById('nope')).toBeUndefined();
	});
});
