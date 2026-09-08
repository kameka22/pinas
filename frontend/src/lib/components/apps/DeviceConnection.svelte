<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { api, type NetworkStatus, type ServiceStatus } from '$stores/api';
	import { toasts, errorMessage } from '$stores/toasts';

	let network: NetworkStatus | null = null;
	let avahi: ServiceStatus | null = null;
	let ports: { port: number; protocol: string; label: string; active: boolean }[] = [];
	let hostname = '';
	let savingHostname = false;
	let avahiBusy = false;

	$: scheme = window.location.protocol.replace(':', '');
	$: uiPort = window.location.port || (scheme === 'https' ? '443' : '80');
	$: addresses = network ? network.interfaces.filter((i) => i.ip_address).map((i) => ({ name: i.display_name || i.name, ip: i.ip_address })) : [];

	async function load() {
		try {
			network = await api.getNetworkStatus();
			hostname = network.hostname;
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadNetwork));
		}
		try { avahi = await api.getServiceStatus('avahi-daemon'); } catch { avahi = null; }
		const [samba, ssh, cups, containers] = await Promise.allSettled([api.getSambaStatus(), api.getSshStatus(), api.getCupsStatus(), api.getContainers(false)]);
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
	}

	async function saveHostname() {
		savingHostname = true;
		try {
			await api.updateNetworkHostname(hostname.trim());
			toasts.success($t.deviceConnection.hostnameSaved);
			await load();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		} finally {
			savingHostname = false;
		}
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

<div class="dc">
	<section class="dc-section">
		<h3 class="dc-title"><Icon icon="mdi:server-network" class="w-5 h-5" />{$t.deviceConnection.identity}</h3>
		<div class="dc-row">
			<label for="hostname">{$t.deviceConnection.hostname}</label>
			<input id="hostname" class="dc-input" bind:value={hostname} pattern="[a-zA-Z0-9-]+" />
			<button class="btn-primary" on:click={saveHostname} disabled={savingHostname || !hostname.trim() || hostname === network?.hostname}>{$t.common.save}</button>
		</div>
		<div class="dc-row">
			<span>{$t.deviceConnection.mdns}</span>
			{#if avahi}
				<label class="toggle"><input type="checkbox" checked={avahi.running} disabled={avahiBusy} on:change={toggleAvahi} /><span class="toggle-slider"></span></label>
				<span class="dc-hint">{avahi.running ? `${hostname}.local` : $t.deviceConnection.mdnsOff}</span>
			{:else}
				<span class="dc-hint">{$t.deviceConnection.mdnsUnavailable}</span>
			{/if}
		</div>
	</section>

	<section class="dc-section">
		<h3 class="dc-title"><Icon icon="mdi:link-variant" class="w-5 h-5" />{$t.deviceConnection.addresses}</h3>
		{#if addresses.length === 0}<p class="dc-hint">{$t.common.noData}</p>{/if}
		<ul class="dc-list">
			{#if avahi?.running}
				<li><span class="dc-if">mDNS</span><a class="mono" href="{scheme}://{hostname}.local:{uiPort}" target="_blank" rel="noreferrer">{scheme}://{hostname}.local:{uiPort}</a><button class="dc-copy" on:click={() => copy(`${scheme}://${hostname}.local:${uiPort}`)} title={$t.common.copy}><Icon icon="mdi:content-copy" class="w-4 h-4" /></button></li>
			{/if}
			{#each addresses as a}
				<li><span class="dc-if">{a.name}</span><a class="mono" href="{scheme}://{a.ip}:{uiPort}" target="_blank" rel="noreferrer">{scheme}://{a.ip}:{uiPort}</a><button class="dc-copy" on:click={() => copy(`${scheme}://${a.ip}:${uiPort}`)} title={$t.common.copy}><Icon icon="mdi:content-copy" class="w-4 h-4" /></button></li>
			{/each}
		</ul>
		{#if addresses.length > 0}
			<p class="dc-hint">{$t.deviceConnection.smbHint} <span class="mono">\\\\{addresses[0].ip}</span> · <span class="mono">smb://{addresses[0].ip}</span></p>
		{/if}
	</section>

	<section class="dc-section">
		<h3 class="dc-title"><Icon icon="mdi:ethernet" class="w-5 h-5" />{$t.deviceConnection.ports}</h3>
		<table class="dc-table">
			<thead><tr><th>{$t.deviceConnection.port}</th><th>{$t.deviceConnection.service}</th><th>{$t.common.status}</th></tr></thead>
			<tbody>
				{#each ports as p}
					<tr><td class="mono">{p.port}/{p.protocol}</td><td>{p.label}</td><td><span class="dc-badge" class:on={p.active}>{p.active ? $t.dashboard.running : $t.dashboard.stopped}</span></td></tr>
				{/each}
			</tbody>
		</table>
	</section>
</div>

<style>
	.dc { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; overflow-y: auto; height: 100%; }
	.dc-section { background: white; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1rem 1.25rem; }
	.dc-title { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9375rem; font-weight: 600; color: #0f172a; margin: 0 0 0.75rem; }
	.dc-row { display: flex; align-items: center; gap: 0.75rem; font-size: 0.8125rem; margin-bottom: 0.625rem; flex-wrap: wrap; }
	.dc-input { border: 1px solid #cbd5e1; border-radius: 0.5rem; padding: 0.375rem 0.625rem; font-size: 0.8125rem; min-width: 12rem; }
	.dc-hint { font-size: 0.75rem; color: #64748b; margin: 0.25rem 0 0; }
	.dc-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.375rem; font-size: 0.8125rem; }
	.dc-list li { display: flex; align-items: center; gap: 0.75rem; }
	.dc-if { min-width: 5rem; color: #64748b; font-size: 0.75rem; }
	.dc-copy { padding: 0.125rem; color: #94a3b8; border-radius: 0.25rem; }
	.dc-copy:hover { color: #0f172a; background: #f1f5f9; }
	.dc-table { width: 100%; font-size: 0.75rem; border-collapse: collapse; }
	.dc-table th { text-align: left; color: #64748b; font-weight: 500; padding: 0.375rem 0.5rem; border-bottom: 1px solid #e2e8f0; }
	.dc-table td { padding: 0.375rem 0.5rem; border-bottom: 1px solid #f1f5f9; }
	.dc-badge { font-size: 0.6875rem; padding: 0.125rem 0.5rem; border-radius: 999px; background: #f1f5f9; color: #64748b; }
	.dc-badge.on { background: #dcfce7; color: #15803d; }
	.mono { font-family: ui-monospace, monospace; }
	a.mono { color: #1d4ed8; }
	.btn-primary { padding: 0.4rem 0.875rem; border-radius: 0.5rem; font-size: 0.8125rem; font-weight: 500; background: #3b82f6; color: white; }
	.btn-primary:disabled { opacity: 0.5; }
	.toggle { position: relative; width: 2.25rem; height: 1.25rem; display: inline-block; }
	.toggle input { opacity: 0; width: 0; height: 0; }
	.toggle-slider { position: absolute; inset: 0; background: #cbd5e1; border-radius: 999px; transition: 0.2s; }
	.toggle-slider::before { content: ''; position: absolute; width: 1rem; height: 1rem; left: 0.125rem; top: 0.125rem; background: white; border-radius: 50%; transition: 0.2s; }
	.toggle input:checked + .toggle-slider { background: #3b82f6; }
	.toggle input:checked + .toggle-slider::before { transform: translateX(1rem); }
</style>
