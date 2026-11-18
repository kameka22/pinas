<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '#lib/i18n/index.js';
	import { api } from '#lib/stores/api.js';
	import type { SambaStatus, SmbGlobalConfig, ShareInfo, NfsStatus, NfsShareConfig } from '#lib/stores/api.js';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';
	import FolderPicker from '#lib/components/ui/FolderPicker.svelte';
	import ConfirmModal from '#lib/components/ui/ConfirmModal.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsTabs from '#lib/components/ui/settings/SettingsTabs.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsActions from '#lib/components/ui/settings/SettingsActions.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { openWindow } from '#lib/stores/windows.js';
	import { getAppById } from '#lib/stores/desktop.js';

	// Tabs
	type TabId = 'smb' | 'nfs';
	let activeTab: TabId = 'smb';

	const tabs: { value: TabId; label: string }[] = [
		{ value: 'smb', label: 'SMB/CIFS' },
		{ value: 'nfs', label: 'NFS' }
	];

	// NFS state
	let nfsStatus: NfsStatus | null = null;
	let nfsExports: ShareInfo[] = [];
	let nfsLoading = false;
	let nfsError = '';
	let exportToDelete: ShareInfo | null = null;
	let nfsBusy = false;
	let showNfsForm = false;
	let nfsForm: { name: string; path: string; clients: string; read_only: boolean; sync: boolean; squash: NfsShareConfig['squash'] } = {
		name: '', path: '', clients: '192.168.1.0/24', read_only: false, sync: true, squash: 'root_squash'
	};

	async function loadNfsData() {
		nfsLoading = true;
		nfsError = '';
		try {
			const [status, shares] = await Promise.all([api.getNfsStatus(), api.getShares()]);
			nfsStatus = status;
			nfsExports = shares.filter((s) => s.share_type === 'nfs');
		} catch (e) {
			nfsError = errorMessage(e, $t.common.errors.loadFailed);
		} finally {
			nfsLoading = false;
		}
	}

	async function toggleNfs() {
		if (!nfsStatus) return;
		nfsBusy = true;
		try {
			nfsStatus = nfsStatus.enabled ? await api.disableNfs() : await api.enableNfs();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			nfsBusy = false;
		}
	}

	async function createNfsExport() {
		nfsBusy = true;
		try {
			await api.createShare({
				name: nfsForm.name.trim(),
				path: nfsForm.path.trim(),
				share_type: 'nfs',
				config: {
					clients: nfsForm.clients.split(/[\s,]+/).filter(Boolean),
					read_only: nfsForm.read_only,
					sync: nfsForm.sync,
					squash: nfsForm.squash,
					subtree_check: false
				}
			});
			showNfsForm = false;
			nfsForm = { ...nfsForm, name: '', path: '' };
			toasts.success($t.fileService.nfs.exportCreated);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		} finally {
			nfsBusy = false;
		}
	}

	async function toggleExport(share: ShareInfo) {
		try {
			await api.toggleShare(share.id, !share.enabled);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		}
	}

	async function deleteExport(share: ShareInfo) {
		exportToDelete = null;
		try {
			await api.deleteShare(share.id);
			await loadNfsData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	// SMB state
	let sambaStatus: SambaStatus | null = null;
	let smbConfig: SmbGlobalConfig = {
		workgroup: 'WORKGROUP',
		server_string: 'PiNAS',
		min_protocol: 'SMB2',
		max_protocol: 'SMB3'
	};
	let activeShares: ShareInfo[] = [];
	let smbLoading = true;
	let smbError: string | null = null;
	let toggleLoading = false;
	let configLoading = false;
	let savedSmbConfig: SmbGlobalConfig | null = null;
	$: smbDirty = savedSmbConfig !== null && (Object.keys(smbConfig) as (keyof SmbGlobalConfig)[]).some((k) => smbConfig[k] !== savedSmbConfig?.[k]);

	function selectTab(tabId: TabId) {
		if (tabId === activeTab) return;
		activeTab = tabId;
		if (tabId === 'smb') loadSmbData();
		if (tabId === 'nfs') loadNfsData();
	}

	onMount(() => {
		loadSmbData();
	});

	async function loadSmbData() {
		smbLoading = true;
		smbError = null;
		try {
			const [status, config, shares] = await Promise.all([
				api.getSambaStatus(),
				api.getSmbConfig(),
				api.getShares()
			]);
			sambaStatus = status;
			smbConfig = { ...config };
			savedSmbConfig = { ...config };
			activeShares = shares.filter(s => s.share_type === 'smb' && s.enabled);
		} catch (e) {
			smbError = errorMessage(e, $t.common.errors.loadFailed);
		} finally {
			smbLoading = false;
		}
	}

	async function toggleSamba() {
		if (!sambaStatus) return;
		toggleLoading = true;
		try {
			if (sambaStatus.enabled) {
				await api.disableSamba();
			} else {
				await api.enableSamba();
			}
			await loadSmbData();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			toggleLoading = false;
		}
	}

	async function applyConfig() {
		configLoading = true;
		try {
			await api.updateSmbConfig(smbConfig);
			savedSmbConfig = { ...smbConfig };
			toasts.success($t.fileService.smb.applySuccess);
		} catch (e) {
			toasts.error(errorMessage(e, $t.fileService.smb.applyError));
		} finally {
			configLoading = false;
		}
	}

	function openShareManager() {
		const app = getAppById('shares');
		if (app) {
			openWindow({
				id: app.id,
				title: app.label,
				icon: app.icon,
				component: app.component,
				x: 200 + Math.random() * 100,
				y: 100 + Math.random() * 50,
				width: 900,
				height: 600,
				gradient: app.gradient
			});
		}
	}
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.fileService} description={$t.settingsUi.descriptions.files} icon="mdi:folder-network-outline" />
	<SettingsTabs label={$t.controlPanel.items.fileService} items={tabs} value={activeTab} onchange={(value) => selectTab(value === 'nfs' ? 'nfs' : 'smb')} />

	{#if activeTab === 'smb'}
		{#if smbLoading && !sambaStatus}
			<SettingsState kind="loading" message={$t.common.loading} />
		{:else if smbError && !sambaStatus}
			<SettingsState kind="error" message={smbError} onretry={loadSmbData} />
		{:else if sambaStatus}
			<SettingsSection title={$t.fileService.smb.title} description={$t.fileService.smb.description} icon="mdi:microsoft-windows">
				{#snippet actions()}
					<span class="settings-badge" class:success={sambaStatus?.running}>
						{sambaStatus?.running ? $t.fileService.smb.running : $t.fileService.smb.stopped}
					</span>
				{/snippet}
				<SettingsRow label={$t.fileService.smb.enable}>
					<SettingsToggle label={$t.fileService.smb.enable} checked={sambaStatus.enabled} disabled={toggleLoading} onchange={toggleSamba} />
				</SettingsRow>
				{#if sambaStatus.version}
					<SettingsRow label={$t.fileService.smb.version}><span class="settings-mono">{sambaStatus.version}</span></SettingsRow>
				{/if}
				<SettingsRow label={$t.fileService.smb.connectedUsers}><span>{sambaStatus.connected_users}</span></SettingsRow>
			</SettingsSection>

			{#if sambaStatus.enabled}
				<SettingsSection title={$t.fileService.smb.globalConfig} icon="mdi:tune-variant">
					<div class="settings-grid">
						<div class="settings-field">
							<label for="smb-workgroup">{$t.fileService.smb.workgroup}</label>
							<input id="smb-workgroup" type="text" bind:value={smbConfig.workgroup} />
						</div>
						<div class="settings-field">
							<label for="smb-description">{$t.fileService.smb.serverDescription}</label>
							<input id="smb-description" type="text" bind:value={smbConfig.server_string} />
						</div>
						<div class="settings-field">
							<label for="smb-min">{$t.fileService.smb.minProtocol}</label>
							<select id="smb-min" bind:value={smbConfig.min_protocol}>
								<option value="SMB2">SMB2</option>
								<option value="SMB2_10">SMB2_10</option>
								<option value="SMB3">SMB3</option>
							</select>
						</div>
						<div class="settings-field">
							<label for="smb-max">{$t.fileService.smb.maxProtocol}</label>
							<select id="smb-max" bind:value={smbConfig.max_protocol}>
								<option value="SMB2">SMB2</option>
								<option value="SMB3">SMB3</option>
								<option value="SMB3_11">SMB3_11</option>
							</select>
						</div>
					</div>
					<SettingsActions dirty={smbDirty} saving={configLoading} onsave={applyConfig} oncancel={() => { if (savedSmbConfig) smbConfig = { ...savedSmbConfig }; }} />
				</SettingsSection>

				<SettingsSection title={`${$t.fileService.smb.activeShares} (${activeShares.length})`} icon="mdi:folder-open-outline">
					{#snippet actions()}
						<button class="settings-button" on:click={openShareManager}>
							{$t.fileService.smb.manageShares}
							<Icon icon="mdi:arrow-right" width="16" />
						</button>
					{/snippet}
					{#if activeShares.length === 0}
						<SettingsState kind="empty" message={$t.fileService.noActiveShares} />
					{:else}
						<div class="settings-table-wrap">
							<table class="settings-table">
								<tbody>
									{#each activeShares as share}
										<tr><td><strong>{share.name}</strong></td><td class="settings-mono">{share.path}</td></tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</SettingsSection>
			{/if}
		{/if}
	{:else if activeTab === 'nfs'}
		{#if nfsLoading && !nfsStatus}
			<SettingsState kind="loading" message={$t.common.loading} />
		{:else if nfsError && !nfsStatus}
			<SettingsState kind="error" message={nfsError} onretry={loadNfsData} />
		{:else if nfsStatus}
			<SettingsSection title="NFS" description={$t.fileService.nfs.description} icon="mdi:folder-network">
				{#snippet actions()}
					<span class="settings-badge" class:success={nfsStatus?.running}>
						{nfsStatus?.running ? $t.fileService.smb.running : $t.fileService.smb.stopped}
					</span>
				{/snippet}
				<SettingsRow label={$t.fileService.nfs.enable} hint={$t.fileService.nfs.exportsCount.replace('{n}', String(nfsStatus.export_count))}>
					<SettingsToggle label={$t.fileService.nfs.enable} checked={nfsStatus.enabled} disabled={nfsBusy || !nfsStatus.nfsd_available} onchange={toggleNfs} />
				</SettingsRow>
				{#if !nfsStatus.nfsd_available}
					<div class="settings-notice warning"><Icon icon="mdi:alert-outline" width="18" /><span>{$t.fileService.nfs.notAvailable}</span></div>
				{/if}
			</SettingsSection>

			<SettingsSection title={$t.fileService.nfs.exports} icon="mdi:folder-network-outline">
				{#snippet actions()}
					{#if !showNfsForm}
						<button class="settings-button primary" on:click={() => (showNfsForm = true)}>
							<Icon icon="mdi:plus" width="16" />
							{$t.fileService.nfs.addExport}
						</button>
					{/if}
				{/snippet}

				{#if showNfsForm}
					<form class="inline-form" on:submit|preventDefault={createNfsExport}>
						<div class="settings-grid">
							<div class="settings-field">
								<label for="nfs-name">{$t.fileService.nfs.name}</label>
								<input id="nfs-name" type="text" bind:value={nfsForm.name} placeholder="media" />
							</div>
							<div class="settings-field">
								<label for="nfs-clients">{$t.fileService.nfs.clients}</label>
								<input id="nfs-clients" type="text" bind:value={nfsForm.clients} placeholder="192.168.1.0/24 10.0.0.5 *" />
							</div>
						</div>
						<p class="settings-help">{$t.fileService.nfs.clientsHint}</p>
						<FolderPicker bind:value={nfsForm.path} label={$t.fileService.nfs.path} placeholder="/storage/shares/media" />
						<div class="settings-row-control">
							<label class="check"><input type="checkbox" bind:checked={nfsForm.read_only} />{$t.fileService.nfs.readOnly}</label>
							<label class="check"><input type="checkbox" bind:checked={nfsForm.sync} />{$t.fileService.nfs.sync}</label>
							<label class="check">{$t.fileService.nfs.squash}
								<select bind:value={nfsForm.squash}>
									<option value="root_squash">root_squash</option>
									<option value="no_root_squash">no_root_squash</option>
									<option value="all_squash">all_squash</option>
								</select>
							</label>
						</div>
						<div class="settings-actions-buttons">
							<button type="button" class="settings-button" on:click={() => (showNfsForm = false)} disabled={nfsBusy}>{$t.common.cancel}</button>
							<button type="submit" class="settings-button primary" disabled={nfsBusy || !nfsForm.name.trim() || !nfsForm.path.trim() || !nfsForm.clients.trim()}>{$t.common.create}</button>
						</div>
					</form>
				{/if}

				{#if nfsExports.length === 0}
					<SettingsState kind="empty" message={$t.fileService.nfs.noExports} />
				{:else}
					<div class="settings-table-wrap">
						<table class="settings-table">
							<thead><tr><th>{$t.fileService.nfs.name}</th><th>{$t.fileService.nfs.path}</th><th>{$t.fileService.nfs.clients}</th><th></th></tr></thead>
							<tbody>
								{#each nfsExports as share (share.id)}
									<tr class:muted={!share.enabled}>
										<td><strong>{share.name}</strong></td>
										<td class="settings-mono">{share.path}</td>
										<td>{(share.nfs?.clients || ['*']).join(', ')} · {share.nfs?.read_only ? 'ro' : 'rw'} · {share.nfs?.squash}</td>
										<td class="row-actions">
											<SettingsToggle label={share.name} checked={share.enabled} onchange={() => toggleExport(share)} />
											<button class="settings-button ghost danger-hover" on:click={() => (exportToDelete = share)} title={$t.common.delete} aria-label={$t.common.delete}><Icon icon="mdi:delete-outline" width="18" /></button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
				<p class="settings-help">{$t.fileService.nfs.mountHint} <span class="settings-mono">mount -t nfs {window.location.hostname}:/storage/shares/&lt;name&gt; /mnt</span></p>
			</SettingsSection>
		{/if}
	{/if}
</div>

<ConfirmModal
	visible={exportToDelete !== null}
	title={$t.fileService.nfs.deleteTitle}
	message={$t.fileService.nfs.deleteConfirm.replace('{name}', exportToDelete?.name ?? '')}
	confirmLabel={$t.common.delete}
	cancelLabel={$t.common.cancel}
	danger
	on:confirm={() => exportToDelete && deleteExport(exportToDelete)}
	on:cancel={() => (exportToDelete = null)}
/>

<style>
	.inline-form { display: flex; flex-direction: column; gap: 12px; padding: 16px; margin-bottom: 16px; border: 1px solid #e2e8f0; border-radius: 10px; background: #f8fafc; }
	.check { display: inline-flex; align-items: center; gap: 6px; font-size: 13px; color: #334155; }
	.row-actions { display: flex; align-items: center; justify-content: flex-end; gap: 4px; }
	tr.muted td { color: #94a3b8; }
</style>

