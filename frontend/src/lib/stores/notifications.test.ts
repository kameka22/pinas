import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { auth } from './api';
import { notifications, unreadCount, pushNotification, loadNotifications, markNotificationRead, dismissNotification, timeAgo, notificationVisual } from './notifications';

const n = (id: string, read = false) => ({ id, level: 'info', title: id, message: 'm', source: 'system', read, created_at: new Date().toISOString() });

function mockFetch(body: unknown, status = 200) {
	vi.stubGlobal('fetch', vi.fn(async () => new Response(status === 204 ? null : JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } })));
}

beforeEach(async () => {
	auth.set({ isAuthenticated: true, user: { id: '1', username: 'root', role: 'admin' } });
	mockFetch([]);
	await loadNotifications();
});

describe('notifications store', () => {
	it('loads from the API for admins and counts unread', async () => {
		mockFetch([n('a'), n('b', true)]);
		await loadNotifications();
		expect(get(notifications)).toHaveLength(2);
		expect(get(unreadCount)).toBe(1);
	});

	it('is empty for non-admin users even when pushed', async () => {
		auth.set({ isAuthenticated: true, user: { id: '2', username: 'bob', role: 'user' } });
		await loadNotifications();
		pushNotification(n('x'));
		expect(get(notifications)).toEqual([]);
	});

	it('push prepends and de-duplicates by id', () => {
		pushNotification(n('a'));
		pushNotification(n('b'));
		pushNotification({ ...n('a'), message: 'updated' });
		expect(get(notifications).map((x) => x.id)).toEqual(['a', 'b']);
		expect(get(notifications)[0].message).toBe('updated');
	});

	it('mark read and dismiss update locally and call the API', async () => {
		pushNotification(n('a'));
		mockFetch(null, 204);
		await markNotificationRead('a');
		expect(get(unreadCount)).toBe(0);
		await dismissNotification('a');
		expect(get(notifications)).toEqual([]);
	});

	it('timeAgo is relative and localized', () => {
		const now = new Date('2026-09-08T12:00:00Z');
		expect(timeAgo('2026-09-08T11:58:30Z', 'en', now)).toMatch(/1 minute ago/);
		expect(timeAgo('2026-09-08T09:00:00Z', 'fr', now)).toMatch(/il y a 3 heures/);
		expect(timeAgo('garbage', 'en', now)).toBe('');
	});

	it('visuals depend on source and level', () => {
		expect(notificationVisual({ source: 'storage', level: 'error' })).toEqual({ icon: 'mdi:harddisk', gradient: 'from-red-400 to-red-500' });
		expect(notificationVisual({ source: 'nope', level: 'nope' }).icon).toBe('mdi:bell');
	});
});
