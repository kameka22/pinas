import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { api, auth } from './api';
import { toasts } from './toasts';

function mockFetch(status: number, body: unknown | string, headers: Record<string, string> = {}) {
	const text = typeof body === 'string' ? body : JSON.stringify(body);
	const fetchMock = vi.fn(async () =>
		new Response(status === 204 ? null : text, {
			status,
			headers: { 'content-type': 'application/json', ...headers }
		})
	);
	vi.stubGlobal('fetch', fetchMock);
	return fetchMock;
}

beforeEach(() => {
	toasts.clear();
	auth.set({ isAuthenticated: true, user: { id: 'u1', username: 'alice', role: 'admin' } });
});

describe('ApiClient', () => {
	it('sends JSON with credentials and parses the response', async () => {
		const f = mockFetch(200, { ok: 1 });
		const res = await api.post<{ ok: number }>('/x', { a: 1 });
		expect(res).toEqual({ ok: 1 });
		const [url, init] = f.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toBe('/api/x');
		expect(init.method).toBe('POST');
		expect(init.credentials).toBe('include');
		expect(init.body).toBe(JSON.stringify({ a: 1 }));
	});

	it('returns undefined for 204 / empty bodies', async () => {
		mockFetch(204, '');
		expect(await api.delete('/x')).toBeUndefined();
		mockFetch(200, '');
		expect(await api.get('/x')).toBeUndefined();
	});

	it('surfaces the unified {error, code} body as an Error with status and code', async () => {
		mockFetch(404, { error: 'User not found', code: 'USER_NOT_FOUND' });
		await expect(api.get('/users/nope')).rejects.toMatchObject({
			message: 'User not found',
			status: 404,
			code: 'USER_NOT_FOUND'
		});
	});

	it('logs the user out on 401 (revoked session)', async () => {
		mockFetch(401, { error: 'Session has been revoked', code: 'SESSION_REVOKED' });
		await expect(api.get('/system/info')).rejects.toMatchObject({ status: 401 });
		expect(get(auth).isAuthenticated).toBe(false);
	});

	it('shows a toast on 403 and keeps the session', async () => {
		mockFetch(403, { error: 'Admin access required', code: 'FORBIDDEN' });
		await expect(api.get('/storage/disks')).rejects.toMatchObject({ status: 403 });
		expect(get(auth).isAuthenticated).toBe(true);
		expect(get(toasts).some((t) => t.kind === 'error')).toBe(true);
	});

	it('requestRaw never throws and returns status + parsed body', async () => {
		mockFetch(400, { output: 'command not allowed', exit_code: 1, dev_mode: true, cwd: '/' });
		const res = await api.terminalExec('rm -rf /', '/');
		expect(res.ok).toBe(false);
		expect(res.status).toBe(400);
		expect(res.data?.output).toBe('command not allowed');
	});

	it('encodes path parameters of typed helpers', async () => {
		const f = mockFetch(200, []);
		await api.getContainerLogs('a/b c', 50);
		expect((f.mock.calls[0] as unknown as [string])[0]).toBe('/api/docker/containers/a%2Fb%20c/logs?tail=50');
		await api.uninstallPackage('jellyfin', true);
		expect((f.mock.calls[1] as unknown as [string])[0]).toBe('/api/packages/jellyfin?delete_data=true');
	});

	it('falls back to HTTP status text when the body is not JSON', async () => {
		mockFetch(502, 'Bad gateway', { 'content-type': 'text/plain' });
		await expect(api.get('/x')).rejects.toMatchObject({ message: 'HTTP 502', status: 502 });
	});
});

describe('file download URLs', () => {
	it('builds the download endpoint with location and inline flags', () => {
		expect(api.fileUrl('docs/a b.txt')).toBe('/api/files/download?path=docs%2Fa+b.txt');
		expect(api.fileUrl('x', 'share-1', true)).toBe('/api/files/download?path=x&location_id=share-1&inline=true');
	});

	it('downloadFile navigates through a hidden anchor (folders get a .zip name)', () => {
		const clicks: string[] = [];
		const orig = HTMLAnchorElement.prototype.click;
		HTMLAnchorElement.prototype.click = function () { clicks.push(`${this.download}|${this.getAttribute('href')}`); };
		api.downloadFile('docs/readme.txt');
		api.downloadFile('docs', 'share-1', true);
		HTMLAnchorElement.prototype.click = orig;
		expect(clicks).toEqual([
			'readme.txt|/api/files/download?path=docs%2Freadme.txt',
			'docs.zip|/api/files/download?path=docs&location_id=share-1'
		]);
	});
});
