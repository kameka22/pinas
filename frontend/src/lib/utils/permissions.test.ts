import { beforeEach, expect, it, vi } from 'vitest';
import { api } from '#lib/stores/api.js';
import { setFolderPermission } from './permissions.js';
vi.mock('#lib/stores/api.js', () => ({
	api: {
		createPermission: vi.fn(),
		updatePermission: vi.fn(),
		deletePermission: vi.fn(),
	},
}));
beforeEach(() => vi.clearAllMocks());
it('stores an explicit denial instead of removing the rule', async () => {
	await setFolderPermission('/pool/data', { user_id: 'alice' }, 'rule', 'none');
	expect(api.updatePermission).toHaveBeenCalledWith('rule', 'none');
	expect(api.deletePermission).not.toHaveBeenCalled();
});
it('creates a denial when no rule exists', async () => {
	await setFolderPermission(
		'/pool/data',
		{ group_id: 'users' },
		undefined,
		'none',
	);
	expect(api.createPermission).toHaveBeenCalledWith({
		path: '/pool/data',
		group_id: 'users',
		permission: 'none',
	});
});
it('removes only an explicit rule when returning to default access', async () => {
	await setFolderPermission(
		'/pool/data',
		{ user_id: 'alice' },
		'rule',
		'default',
	);
	expect(api.deletePermission).toHaveBeenCalledWith('rule');
	expect(api.updatePermission).not.toHaveBeenCalled();
	expect(api.createPermission).not.toHaveBeenCalled();
});
it('does nothing for already inherited access', async () => {
	await setFolderPermission(
		'/pool/data',
		{ user_id: 'alice' },
		undefined,
		'default',
	);
	expect(api.deletePermission).not.toHaveBeenCalled();
	expect(api.createPermission).not.toHaveBeenCalled();
});
