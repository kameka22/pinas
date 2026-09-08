import { writable, derived, get } from 'svelte/store';
import { api, auth } from './api';
import type { Notification } from './api';

const MAX_KEPT = 200;

const list = writable<Notification[]>([]);
export const notifications = { subscribe: list.subscribe };
export const unreadCount = derived(list, ($list) => $list.filter((n) => !n.read).length);

function isAdmin(): boolean {
	return get(auth).user?.role === 'admin';
}

/** Load the latest notifications from the backend (admin only; others see an empty center). */
export async function loadNotifications(): Promise<void> {
	if (!isAdmin()) {
		list.set([]);
		return;
	}
	try {
		list.set(await api.getNotifications(100));
	} catch {
		// keep whatever we have; a toast is already shown for 403/5xx by the client
	}
}

/** Insert or refresh a notification received over the WebSocket. */
export function pushNotification(n: Notification): void {
	if (!isAdmin()) return;
	list.update((items) => {
		const rest = items.filter((x) => x.id !== n.id);
		return [n, ...rest].slice(0, MAX_KEPT);
	});
}

export async function markNotificationRead(id: string): Promise<void> {
	list.update((items) => items.map((n) => (n.id === id ? { ...n, read: true } : n)));
	try { await api.markNotificationRead(id); } catch { await loadNotifications(); }
}

export async function markAllNotificationsRead(): Promise<void> {
	list.update((items) => items.map((n) => ({ ...n, read: true })));
	try { await api.markAllNotificationsRead(); } catch { await loadNotifications(); }
}

export async function dismissNotification(id: string): Promise<void> {
	list.update((items) => items.filter((n) => n.id !== id));
	try { await api.deleteNotification(id); } catch { await loadNotifications(); }
}

export async function clearNotifications(): Promise<void> {
	list.set([]);
	try { await api.clearNotifications(); } catch { await loadNotifications(); }
}

/** "3 min ago" style label, in the UI language */
export function timeAgo(iso: string, locale: string, now: Date = new Date()): string {
	const then = new Date(iso).getTime();
	if (Number.isNaN(then)) return '';
	const diff = Math.round((then - now.getTime()) / 1000);
	const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
	const abs = Math.abs(diff);
	if (abs < 60) return rtf.format(Math.trunc(diff), 'second');
	if (abs < 3600) return rtf.format(Math.trunc(diff / 60), 'minute');
	if (abs < 86400) return rtf.format(Math.trunc(diff / 3600), 'hour');
	if (abs < 30 * 86400) return rtf.format(Math.trunc(diff / 86400), 'day');
	return new Date(iso).toLocaleDateString(locale);
}

/** Icon + gradient per notification source / level */
export function notificationVisual(n: Pick<Notification, 'source' | 'level'>): { icon: string; gradient: string } {
	const bySource: Record<string, string> = {
		storage: 'mdi:harddisk',
		'app-center': 'mdi:apps',
		update: 'mdi:update',
		auth: 'mdi:shield-alert',
		system: 'mdi:information'
	};
	const byLevel: Record<string, string> = {
		success: 'from-green-400 to-green-500',
		warning: 'from-amber-400 to-amber-500',
		error: 'from-red-400 to-red-500',
		info: 'from-blue-400 to-blue-500'
	};
	return { icon: bySource[n.source] || 'mdi:bell', gradient: byLevel[n.level] || byLevel.info };
}
