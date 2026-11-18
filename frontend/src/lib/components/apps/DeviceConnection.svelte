<script lang="ts">
	import Icon from '@iconify/svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { onMount } from 'svelte';
	import { t } from '#lib/i18n/index.js';
	import { api, type NetworkStatus, type ServiceStatus } from '#lib/stores/api.js';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';

	let network: NetworkStatus | null = null;
	let avahi: ServiceStatus | null = null;
	let ports: { port: number; protocol: string; label: string; active: boolean }[] = [];
	let hostname = '';
	let avahiBusy = false;
	let loading = true;
	let loadError = '';

	$: scheme = window.location.protocol.replace(':', '');
	$: uiPort = window.location.port || (scheme === 'https' ? '443' : '80');
	$: addresses = network ? network.interfaces.filter((i) => i.ip_address).map((i) => ({ name: i.display_name || i.name, ip: i.ip_address })) : [];

	async function load() {
		loading = true;
		loadError = '';
		try {
			network = await api.getNetworkStatus();
			hostname = network.hostname;
		} catch (e) {
			loadError = errorMessage(e, $t.common.errors.loadNetwork);
		}
		try { avahi = await api.getServiceStatus('avahi-daemon'); } catch { avahi = null; }
		const [samba, ssh, cups, containers] = await Promise.allSettled([api.getSambaStatus(), api.getSshStatus(), api.getCupsStatus(), api.getDockerStatus().then(status => status.running ? api.getContainers(false) : [])]);
		const list: typeof ports = [
			{ port: Number(uiPort), protocol: 'tcp', label: 'PiNAS UI', active: true },
			{ port: 445, protocol: 'tcp', label: 'SMB', active: samba.status === 'fulfilled' && samba.value.running },
			{ port: 139, protocol: 'tcp', label: 'NetBIOS', active: samba.status === 'fulfilled' && samba.value.running },
			{ port: ssh.status === 'fulfilled' ? ssh.value.port : 22, protocol: 'tcp', label: 'SSH', active: ssh.status === 'fulfilled' && ssh.value.running },
			{ port: 631, protocol: 'tcp', label: 'CUPS / IPP', active: cups.status === 'fulfilled' && cups.value.running }
		];
		if (containers.status === 'fulfilled') {
			for (const c of containers.value) {
				for (const p of c.ports) {
					if (p.host) list.push({ port: p.host, protocol: p.protocol, label: `Docker · ${c.name}`, active: c.state === 'running' });
				}
			}
		}
		ports = list.sort((a, b) => a.port - b.port);
		loading = false;
	}

	async function toggleAvahi() {
		if (!avahi) return;
		avahiBusy = true;
		try {
			if (avahi.running) {
				await api.serviceAction('avahi-daemon', 'stop');
				await api.serviceAction('avahi-daemon', 'disable');
			} else {
				await api.serviceAction('avahi-daemon', 'enable');
				await api.serviceAction('avahi-daemon', 'start');
			}
			avahi = await api.getServiceStatus('avahi-daemon');
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			avahiBusy = false;
		}
	}

	function copy(text: string) {
		navigator.clipboard?.writeText(text).then(() => toasts.success($t.common.copy), () => {});
	}

	onMount(load);
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.deviceConnection} description={$t.settingsUi.descriptions.device} icon="mdi:monitor-screenshot" />

	{#if loading && !network}
		<SettingsState kind="loading" message={$t.common.loading} />
	{:else}
		{#if loadError}<SettingsState kind="error" message={loadError} onretry={load} />{/if}

		<SettingsSection title={$t.deviceConnection.identity} icon="mdi:server-network">
			<SettingsRow label={$t.deviceConnection.hostname}>
				<span class="settings-mono">{hostname || '—'}</span>
			</SettingsRow>
			<SettingsRow label={$t.deviceConnection.mdns} hint={avahi ? (avahi.running ? `${hostname}.local` : $t.deviceConnection.mdnsOff) : $t.deviceConnection.mdnsUnavailable}>
				{#if avahi}
					<SettingsToggle label={$t.deviceConnection.mdns} checked={avahi.running} disabled={avahiBusy} onchange={toggleAvahi} />
				{/if}
			</SettingsRow>
		</SettingsSection>

		<SettingsSection title={$t.deviceConnection.addresses} icon="mdi:link-variant">
			{#if addresses.length === 0 && !avahi?.running}
				<SettingsState kind="empty" message={$t.common.noData} />
			{:else}
				{#if avahi?.running}
					<SettingsRow label="mDNS">
						<a class="settings-mono link" href="{scheme}://{hostname}.local:{uiPort}" target="_blank" rel="noreferrer">{scheme}://{hostname}.local:{uiPort}</a>
						<button class="settings-button ghost" on:click={() => copy(`${scheme}://${hostname}.local:${uiPort}`)} title={$t.common.copy} aria-label={$t.common.copy}><Icon icon="mdi:content-copy" width="16" /></button>
					</SettingsRow>
				{/if}
				{#each addresses as a}
					<SettingsRow label={a.name}>
						<a class="settings-mono link" href="{scheme}://{a.ip}:{uiPort}" target="_blank" rel="noreferrer">{scheme}://{a.ip}:{uiPort}</a>
						<button class="settings-button ghost" on:click={() => copy(`${scheme}://${a.ip}:${uiPort}`)} title={$t.common.copy} aria-label={$t.common.copy}><Icon icon="mdi:content-copy" width="16" /></button>
					</SettingsRow>
				{/each}
				{#if addresses.length > 0}
					<p class="settings-help">{$t.deviceConnection.smbHint} <span class="settings-mono">{String.fromCharCode(92, 92) + addresses[0].ip}</span> · <span class="settings-mono">smb://{addresses[0].ip}</span></p>
				{/if}
			{/if}
		</SettingsSection>

		<SettingsSection title={$t.deviceConnection.ports} icon="mdi:ethernet">
			<div class="settings-table-wrap">
				<table class="settings-table">
					<thead><tr><th>{$t.deviceConnection.port}</th><th>{$t.deviceConnection.service}</th><th>{$t.common.status}</th></tr></thead>
					<tbody>
						{#each ports as p}
							<tr><td class="settings-mono">{p.port}/{p.protocol}</td><td>{p.label}</td><td><span class="settings-badge" class:success={p.active}>{p.active ? $t.dashboard.running : $t.dashboard.stopped}</span></td></tr>
						{/each}
					</tbody>
				</table>
			</div>
		</SettingsSection>
	{/if}
</div>

<style>
	.link { color: #1d4ed8; overflow-wrap: anywhere; }
</style>
