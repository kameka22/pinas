<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api, type NetworkStatus, type NetworkInterface, type DnsConfig } from '$lib/stores/api';

	// State
	let networkStatus: NetworkStatus | null = null;
	let loading = false;
	let error: string | null = null;
	let saving = false;
	let activeTab: 'general' | 'interfaces' = 'general';

	// General tab form
	let hostname = '';
	let dnsManual = false;
	let dnsPrimary = '';
	let dnsSecondary = '';
	let hostnameEditing = false;

	// Interface edit modal
	let showEditModal = false;
	let editingInterface: NetworkInterface | null = null;
	let editMethod = 'dhcp';
	let editIp = '';
	let editSubnet = '255.255.255.0';
	let editGateway = '';
	let editDns = '';
	let editSaving = false;
	let editError: string | null = null;

	onMount(() => {
		loadNetworkStatus();
	});

	async function loadNetworkStatus() {
		loading = true;
		error = null;
		try {
			networkStatus = await api.getNetworkStatus();
			hostname = networkStatus.hostname;
			dnsManual = networkStatus.dns.manual;
			dnsPrimary = networkStatus.dns.primary;
			dnsSecondary = networkStatus.dns.secondary;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load network status';
		} finally {
			loading = false;
		}
	}

	async function saveGeneral() {
		if (!networkStatus || saving) return;

		saving = true;
		error = null;

		try {
			// Update hostname if changed
			if (hostname !== networkStatus.hostname) {
				await api.updateNetworkHostname(hostname);
			}

			// Update DNS if changed
			if (
				dnsManual !== networkStatus.dns.manual ||
				dnsPrimary !== networkStatus.dns.primary ||
				dnsSecondary !== networkStatus.dns.secondary
			) {
				await api.updateNetworkDns({
					manual: dnsManual,
					primary: dnsPrimary,
					secondary: dnsSecondary
				});
			}

			await loadNetworkStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save settings';
		} finally {
			saving = false;
		}
	}

	function openEditModal(iface: NetworkInterface) {
		editingInterface = iface;
		editMethod = iface.method;
		editIp = iface.ip_address;
		editSubnet = iface.subnet_mask || '255.255.255.0';
		editGateway = iface.gateway;
		editDns = iface.dns;
		editError = null;
		showEditModal = true;
	}

	function closeEditModal() {
		showEditModal = false;
		editingInterface = null;
	}

	async function saveInterface() {
		if (!editingInterface || editSaving) return;

		editSaving = true;
		editError = null;

		try {
			await api.updateNetworkInterface({
				name: editingInterface.name,
				method: editMethod,
				ip_address: editMethod === 'manual' ? editIp : undefined,
				subnet_mask: editMethod === 'manual' ? editSubnet : undefined,
				gateway: editMethod === 'manual' ? editGateway : undefined,
				dns: editMethod === 'manual' ? editDns : undefined
			});

			closeEditModal();
			await loadNetworkStatus();
		} catch (e) {
			editError = e instanceof Error ? e.message : 'Failed to save interface';
		} finally {
			editSaving = false;
		}
	}

	$: hasGeneralChanges =
		networkStatus &&
		(hostname !== networkStatus.hostname ||
			dnsManual !== networkStatus.dns.manual ||
			dnsPrimary !== networkStatus.dns.primary ||
			dnsSecondary !== networkStatus.dns.secondary);
</script>

<div class="network-settings">
	<!-- Header -->
	<div class="section-header">
		<Icon icon="mdi:wifi" class="w-6 h-6" />
		<div class="section-info">
			<h2>{$t.networkSettings?.title || 'Network'}</h2>
			<p>{$t.networkSettings?.description || 'Configure network interfaces and DNS'}</p>
		</div>
	</div>

	<!-- Tabs -->
	<div class="tabs-bar">
		<button class="tab" class:active={activeTab === 'general'} on:click={() => (activeTab = 'general')}>
			{$t.networkSettings?.tabs?.general || 'General'}
		</button>
		<button class="tab" class:active={activeTab === 'interfaces'} on:click={() => (activeTab = 'interfaces')}>
			{$t.networkSettings?.tabs?.interfaces || 'Network Interface'}
		</button>
	</div>

	{#if loading}
		<div class="loading-state">
			<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
			<span>{$t.common.loading}</span>
		</div>
	{:else if error && !networkStatus}
		<div class="error-state">
			<Icon icon="mdi:alert-circle" class="w-5 h-5" />
			<span>{error}</span>
			<button class="retry-btn" on:click={loadNetworkStatus}>
				{$t.common.retry || 'Retry'}
			</button>
		</div>
	{:else if networkStatus}
		{#if activeTab === 'general'}
			<!-- General Tab -->
			<div class="tab-content">
				<!-- Hostname -->
				<div class="service-row">
					<div class="service-label">
						<span class="label">{$t.networkSettings?.hostname || 'Hostname'}</span>
						<span class="sublabel">{$t.networkSettings?.hostnameHint || 'Name of your device on the network'}</span>
					</div>
					{#if hostnameEditing}
						<div class="inline-edit">
							<input
								type="text"
								bind:value={hostname}
								class="inline-input"
								on:keydown={(e) => { if (e.key === 'Enter') hostnameEditing = false; if (e.key === 'Escape') { hostname = networkStatus?.hostname || ''; hostnameEditing = false; }}}
							/>
							<button class="icon-btn" on:click={() => (hostnameEditing = false)}>
								<Icon icon="mdi:check" class="w-4 h-4" />
							</button>
						</div>
					{:else}
						<div class="value-with-edit">
							<span class="value-text">{hostname}</span>
							<button class="icon-btn" on:click={() => (hostnameEditing = true)}>
								<Icon icon="mdi:pencil" class="w-4 h-4" />
							</button>
						</div>
					{/if}
				</div>

				<!-- Default Gateway -->
				<div class="service-row">
					<div class="service-label">
						<span class="label">{$t.networkSettings?.defaultGateway || 'Default Gateway'}</span>
					</div>
					<span class="value-text mono">{networkStatus.default_gateway || '-'}</span>
				</div>

				<!-- DNS Server -->
				<div class="service-row column">
					<div class="row-header">
						<div class="service-label">
							<span class="label">{$t.networkSettings?.dnsServer || 'DNS Server'}</span>
						</div>
					</div>

					<label class="checkbox-row">
						<input type="checkbox" bind:checked={dnsManual} />
						<span>{$t.networkSettings?.configureDnsManually || 'Configure DNS server manually'}</span>
					</label>

					{#if dnsManual}
						<div class="dns-inputs">
							<div class="form-group">
								<label for="dns-primary">{$t.networkSettings?.primaryDns || 'Primary DNS'}</label>
								<input id="dns-primary" type="text" bind:value={dnsPrimary} placeholder="8.8.8.8" />
							</div>
							<div class="form-group">
								<label for="dns-secondary">{$t.networkSettings?.secondaryDns || 'Secondary DNS'}</label>
								<input id="dns-secondary" type="text" bind:value={dnsSecondary} placeholder="8.8.4.4" />
							</div>
						</div>
					{:else}
						<div class="dns-auto">
							<span class="sublabel">{$t.networkSettings?.dnsAuto || 'DNS provided by DHCP'}: {networkStatus.dns.primary}{networkStatus.dns.secondary ? `, ${networkStatus.dns.secondary}` : ''}</span>
						</div>
					{/if}
				</div>

				<!-- Apply Button -->
				{#if hasGeneralChanges}
					<div class="actions-bar">
						{#if error}
							<div class="inline-error">
								<Icon icon="mdi:alert-circle" class="w-4 h-4" />
								{error}
							</div>
						{/if}
						<button class="btn-primary" on:click={saveGeneral} disabled={saving}>
							{#if saving}
								<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
							{/if}
							{$t.common.apply}
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<!-- Interfaces Tab -->
			<div class="tab-content">
				{#each networkStatus.interfaces as iface}
					<div class="interface-card">
						<div class="interface-header">
							<div class="interface-name">
								<Icon
									icon={iface.name.startsWith('wlan') ? 'mdi:wifi' : 'mdi:ethernet'}
									class="w-5 h-5"
								/>
								<span class="name">{iface.display_name}</span>
								<span class="status-badge" class:connected={iface.status === 'connected'}>
									{iface.status === 'connected'
										? ($t.networkSettings?.connected || 'Connected')
										: ($t.networkSettings?.disconnected || 'Disconnected')}
								</span>
							</div>
							<button class="btn-secondary" on:click={() => openEditModal(iface)}>
								<Icon icon="mdi:pencil" class="w-4 h-4" />
								{$t.common.edit}
							</button>
						</div>

						{#if iface.status === 'connected'}
							<div class="interface-details">
								<div class="detail-row">
									<span class="detail-label">{$t.networkSettings?.ipAddress || 'IP Address'}</span>
									<span class="detail-value mono">{iface.ip_address}</span>
								</div>
								<div class="detail-row">
									<span class="detail-label">{$t.networkSettings?.subnetMask || 'Subnet Mask'}</span>
									<span class="detail-value mono">{iface.subnet_mask}</span>
								</div>
								<div class="detail-row">
									<span class="detail-label">{$t.networkSettings?.gateway || 'Gateway'}</span>
									<span class="detail-value mono">{iface.gateway || '-'}</span>
								</div>
								<div class="detail-row">
									<span class="detail-label">{$t.networkSettings?.macAddress || 'MAC Address'}</span>
									<span class="detail-value mono">{iface.mac_address}</span>
								</div>
								{#if iface.speed}
									<div class="detail-row">
										<span class="detail-label">{$t.networkSettings?.speed || 'Speed'}</span>
										<span class="detail-value">{iface.speed}</span>
									</div>
								{/if}
								<div class="detail-row">
									<span class="detail-label">{$t.networkSettings?.method || 'Method'}</span>
									<span class="detail-value">{iface.method === 'dhcp' ? 'DHCP' : ($t.networkSettings?.manual || 'Manual')}</span>
								</div>
							</div>
						{/if}
					</div>
				{/each}

				{#if networkStatus.interfaces.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:ethernet-cable-off" class="w-12 h-12 text-slate-300" />
						<p>{$t.networkSettings?.noInterfaces || 'No network interfaces detected'}</p>
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>

<!-- Edit Interface Modal -->
{#if showEditModal && editingInterface}
	<div class="modal-overlay" on:click={closeEditModal}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:ethernet" class="w-5 h-5" />
					{$t.networkSettings?.editInterface || 'Edit Interface'} - {editingInterface.display_name}
				</h2>
				<button class="modal-close" on:click={closeEditModal}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if editError}
					<div class="error-message">
						<Icon icon="mdi:alert-circle" class="w-4 h-4" />
						{editError}
					</div>
				{/if}

				<!-- IPv4 Method -->
				<div class="form-group">
					<label>{$t.networkSettings?.method || 'Method'}</label>
					<div class="radio-group">
						<label class="radio-label">
							<input type="radio" bind:group={editMethod} value="dhcp" />
							<span>DHCP</span>
						</label>
						<label class="radio-label">
							<input type="radio" bind:group={editMethod} value="manual" />
							<span>{$t.networkSettings?.manual || 'Manual'}</span>
						</label>
					</div>
				</div>

				<!-- Manual IP fields -->
				<div class="form-group" class:disabled={editMethod === 'dhcp'}>
					<label for="edit-ip">{$t.networkSettings?.ipAddress || 'IP Address'}</label>
					<input id="edit-ip" type="text" bind:value={editIp} placeholder="192.168.1.100" disabled={editMethod === 'dhcp'} />
				</div>

				<div class="form-group" class:disabled={editMethod === 'dhcp'}>
					<label for="edit-subnet">{$t.networkSettings?.subnetMask || 'Subnet Mask'}</label>
					<input id="edit-subnet" type="text" bind:value={editSubnet} placeholder="255.255.255.0" disabled={editMethod === 'dhcp'} />
				</div>

				<div class="form-group" class:disabled={editMethod === 'dhcp'}>
					<label for="edit-gateway">{$t.networkSettings?.gateway || 'Gateway'}</label>
					<input id="edit-gateway" type="text" bind:value={editGateway} placeholder="192.168.1.1" disabled={editMethod === 'dhcp'} />
				</div>

				<div class="form-group" class:disabled={editMethod === 'dhcp'}>
					<label for="edit-dns">DNS</label>
					<input id="edit-dns" type="text" bind:value={editDns} placeholder="8.8.8.8" disabled={editMethod === 'dhcp'} />
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={closeEditModal}>
					{$t.common.cancel}
				</button>
				<button class="btn-primary" on:click={saveInterface} disabled={editSaving}>
					{#if editSaving}
						<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
					{/if}
					{$t.common.apply}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.network-settings {
		padding: 24px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.section-header > :global(svg) {
		color: #6b7280;
	}

	.section-info h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 4px;
	}

	.section-info p {
		font-size: 13px;
		color: #6b7280;
	}

	/* Tabs */
	.tabs-bar {
		display: flex;
		gap: 0;
		border-bottom: 1px solid #e5e7eb;
		margin-bottom: 24px;
	}

	.tab {
		padding: 12px 20px;
		font-size: 14px;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		transition: all 0.15s ease;
		background: transparent;
		border-top: none;
		border-left: none;
		border-right: none;
		cursor: pointer;
	}

	.tab:hover {
		color: #374151;
	}

	.tab.active {
		color: #2563eb;
		border-bottom-color: #2563eb;
	}

	.tab-content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* Service rows */
	.service-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
	}

	.service-row.column {
		flex-direction: column;
		align-items: stretch;
		gap: 12px;
	}

	.row-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.service-label {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.service-label .label {
		font-size: 14px;
		font-weight: 500;
		color: #374151;
	}

	.service-label .sublabel {
		font-size: 12px;
		color: #9ca3af;
	}

	.sublabel {
		font-size: 12px;
		color: #9ca3af;
	}

	.value-text {
		font-size: 14px;
		color: #374151;
		font-weight: 500;
	}

	.value-text.mono, .mono {
		font-family: monospace;
	}

	.value-with-edit {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.inline-edit {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.inline-input {
		padding: 6px 10px;
		border: 1px solid #3b82f6;
		border-radius: 6px;
		font-size: 14px;
		width: 180px;
		outline: none;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
		color: #1f2937;
		background: white;
	}

	.icon-btn {
		padding: 6px;
		color: #6b7280;
		border-radius: 6px;
		transition: all 0.15s ease;
	}

	.icon-btn:hover {
		background: #e5e7eb;
		color: #374151;
	}

	/* Checkbox */
	.checkbox-row {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: #374151;
		cursor: pointer;
	}

	.checkbox-row input[type="checkbox"] {
		width: 18px;
		height: 18px;
		accent-color: #3b82f6;
		cursor: pointer;
	}

	/* DNS inputs */
	.dns-inputs {
		display: flex;
		gap: 16px;
	}

	.dns-inputs .form-group {
		flex: 1;
	}

	.dns-auto {
		padding: 4px 0;
	}

	/* Interface cards */
	.interface-card {
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		padding: 16px;
	}

	.interface-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.interface-name {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.interface-name > :global(svg) {
		color: #6b7280;
	}

	.interface-name .name {
		font-size: 15px;
		font-weight: 500;
		color: #1f2937;
	}

	.status-badge {
		padding: 3px 10px;
		border-radius: 12px;
		font-size: 12px;
		font-weight: 500;
		background: #fef2f2;
		color: #dc2626;
	}

	.status-badge.connected {
		background: #dcfce7;
		color: #16a34a;
	}

	.interface-details {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 12px;
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid #e5e7eb;
	}

	.detail-row {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.detail-label {
		font-size: 12px;
		color: #9ca3af;
	}

	.detail-value {
		font-size: 14px;
		color: #374151;
	}

	/* Actions bar */
	.actions-bar {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
		padding-top: 8px;
	}

	.inline-error {
		display: flex;
		align-items: center;
		gap: 6px;
		color: #dc2626;
		font-size: 13px;
	}

	/* Empty state */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 40px;
		color: #9ca3af;
	}

	.empty-state p {
		font-size: 14px;
	}

	/* States */
	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px 20px;
		color: #9ca3af;
	}

	.error-state {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px;
		background: #fef2f2;
		border-radius: 8px;
		color: #dc2626;
	}

	.retry-btn {
		margin-left: auto;
		padding: 6px 12px;
		background: white;
		border: 1px solid #fecaca;
		border-radius: 6px;
		font-size: 13px;
		color: #dc2626;
	}

	.retry-btn:hover {
		background: #fef2f2;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Form */
	.form-group {
		margin-bottom: 16px;
	}

	.form-group.disabled {
		opacity: 0.5;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 6px;
	}

	.form-group input[type="text"] {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		font-family: monospace;
		color: #1f2937;
		background: white;
	}

	.form-group input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-group input:disabled {
		background: #f3f4f6;
		color: #9ca3af;
		cursor: not-allowed;
	}

	/* Radio group */
	.radio-group {
		display: flex;
		gap: 24px;
	}

	.radio-label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		color: #374151;
		cursor: pointer;
	}

	.radio-label input[type="radio"] {
		accent-color: #3b82f6;
	}

	/* Buttons */
	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: #3b82f6;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		color: white;
		transition: background 0.15s ease;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 13px;
		color: #374151;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		width: 90%;
		max-width: 480px;
		background: white;
		border-radius: 12px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
	}

	.modal-header h2 {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 16px;
		font-weight: 600;
		color: #1f2937;
	}

	.modal-close {
		color: #6b7280;
		padding: 4px;
		border-radius: 4px;
	}

	.modal-close:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.modal-body {
		padding: 20px;
	}

	.error-message {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px;
		background: #fef2f2;
		border-radius: 8px;
		color: #dc2626;
		font-size: 13px;
		margin-bottom: 16px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e5e7eb;
	}
</style>
