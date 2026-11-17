<script lang="ts">
 import { onMount } from 'svelte';
 import { setFolderPermission, type PermissionChoice } from '#lib/utils/permissions.js';
 import { api, type PermissionEntry } from '#lib/stores/api.js';
 import { t } from '#lib/i18n/index.js';
 import { errorMessage, toasts } from '#lib/stores/toasts.js';
 export let path: string;
 export let onclose: () => void;
 let users: Awaited<ReturnType<typeof api.getUsers>> = [];
 let groups: Awaited<ReturnType<typeof api.getGroups>> = [];
 let permissions: PermissionEntry[] = [];
 let dialog: HTMLDialogElement;
 let loading = true;
 let busy = false;
 let failed = false;
 $: principals = [
  ...users.map(user => ({ id: user.id, name: user.username, group: false, admin: user.is_admin })),
  ...groups.map(group => ({ id: group.id, name: group.name === 'administrators' ? $t.walkthrough.groupAdmins : group.name === 'users' ? $t.walkthrough.groupUsers : group.name, group: true, admin: false }))
 ];
 onMount(async () => {
  dialog.showModal();
  try {
   [users, groups] = await Promise.all([api.getUsers(), api.getGroups()]);
   permissions = (await api.getFolderPermissions(path)).permissions;
  } catch (e) {
   failed = true;
   toasts.error(errorMessage(e, $t.common.errors.generic));
  } finally { loading = false; }
 });
 function rule(id: string, group: boolean) {
  return permissions.find(p => group ? p.group_id === id : p.user_id === id);
 }
 async function change(id: string, group: boolean, value: string) {
  busy = true;
  try {
   const previous = rule(id, group);
   await setFolderPermission(path, group ? { group_id: id } : { user_id: id }, previous?.id, value as PermissionChoice);
  } catch (e) { toasts.error(errorMessage(e, $t.common.errors.updatePermission)); }
  finally {
   try { permissions = (await api.getFolderPermissions(path)).permissions; }
   catch (e) { failed = true; toasts.error(errorMessage(e, $t.common.errors.generic)); }
   busy = false;
  }
 }
</script>

<dialog bind:this={dialog} aria-labelledby="permissions-title" class="permissions-dialog" on:cancel={e => { e.preventDefault(); if (!busy) onclose(); }}>
  <h2 id="permissions-title">{$t.walkthrough.editPermissions}</h2>
  <p class="path">{path}</p>
  <p>{$t.walkthrough.permissionsHint}</p>
  {#if loading}<p>{$t.common.loading}</p>{:else if !failed}
   <div class="rules">
    {#each principals as principal (`${principal.group}-${principal.id}`)}
     <label>
      <span>{principal.name}{principal.admin ? ` (${$t.userManager.roles.administrator})` : ''}</span>
      <select disabled={busy || principal.admin} value={rule(principal.id, principal.group)?.permission ?? 'default'} on:change={e => change(principal.id, principal.group, e.currentTarget.value)}>
       <option value="default">{$t.walkthrough.defaultAccess}</option>
       <option value="none">{$t.userManager.permissions.noAccess}</option>
       <option value="read">{$t.userManager.permissions.readOnly}</option>
       <option value="write">{$t.userManager.permissions.readWrite}</option>
      </select>
     </label>
    {/each}
   </div>
  {/if}
  <button class="btn-secondary" disabled={busy} on:click={onclose}>{$t.common.close}</button>
</dialog>

<style>
 dialog::backdrop { background: #0006; }
 .permissions-dialog { border: none; margin: auto; background: white; color: #1e293b; border-radius: 1rem; padding: 1.5rem; width: 36rem; max-width: 100%; max-height: 85vh; overflow: auto; }
 h2 { font-size: 1.125rem; font-weight: 600; }
 p { margin: .75rem 0; font-size: .875rem; }
 .path { font-family: monospace; overflow-wrap: anywhere; }
 .rules { margin: 1rem 0; }
 label { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: .5rem 0; }
 select { border: 1px solid #cbd5e1; border-radius: .375rem; padding: .375rem; }
</style>
