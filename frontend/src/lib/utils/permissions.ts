import { api, type PermissionLevel } from '#lib/stores/api.js';
export type PermissionChoice = PermissionLevel | 'default';

/** Removing an explicit rule is different from storing an explicit denial. */
export async function setFolderPermission(
	path: string,
	subject: { user_id?: string; group_id?: string },
	previousId: string | undefined,
	permission: PermissionChoice,
): Promise<void> {
	if (permission === 'default') {
		if (previousId) await api.deletePermission(previousId);
	} else if (previousId) {
		await api.updatePermission(previousId, permission);
	} else {
		await api.createPermission({ path, ...subject, permission });
	}
}
