<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '#lib/i18n/index.js';
	import { api, type CupsStatus, type CupsPrinter, type DetectedPrinter, type PrinterDriver, type PrintJob } from '#lib/stores/api.js';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';
	import ConfirmModal from '#lib/components/ui/ConfirmModal.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';

	// Service state
	let cupsStatus: CupsStatus | null = null;
	let cupsLoading = false;
	let cupsError: string | null = null;
	let cupsToggling = false;

	// Printers
	let printers: CupsPrinter[] = [];
	let printersLoading = false;

	// Detection
	let detectedPrinters: DetectedPrinter[] = [];
	let detecting = false;
	// Printers already configured in CUPS are not offered again.
	$: newDetected = detectedPrinters.filter((d) => !printers.some((p) => p.uri === d.uri));

	// Add printer modal
	let showAddModal = false;
	let addPrinterUri = '';
	let addPrinterName = '';
	let addPrinterDriver = '';
	let addPrinterLocation = '';
	let addPrinterShared = true;
	let availableDrivers: PrinterDriver[] = [];
	let driversLoading = false;
	let adding = false;

	// Remove confirmation
	let removeTarget: CupsPrinter | null = null;

	// Jobs
	let jobs: PrintJob[] = [];
	let jobsLoading = false;

	onMount(() => {
		loadStatus();
	});

	async function loadStatus() {
		cupsLoading = true;
		cupsError = null;
		try {
			cupsStatus = await api.getCupsStatus();
			if (cupsStatus.enabled) {
				await loadPrinters();
				await loadJobs();
			}
		} catch (e) {
			cupsError = errorMessage(e, $t.common.errors.loadCupsStatus);
		} finally {
			cupsLoading = false;
		}
	}

	async function toggleCups() {
		if (!cupsStatus || cupsToggling) return;

		cupsToggling = true;

		try {
			if (cupsStatus.enabled) {
				await api.disableCups();
			} else {
				await api.enableCups();
			}
			await loadStatus();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.toggleCups));
		} finally {
			cupsToggling = false;
		}
	}

	async function loadPrinters() {
		printersLoading = true;
		try {
			printers = await api.getCupsPrinters();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
			printers = [];
		} finally {
			printersLoading = false;
		}
	}

	async function loadJobs() {
		jobsLoading = true;
		try {
			jobs = await api.getPrintJobs();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
			jobs = [];
		} finally {
			jobsLoading = false;
		}
	}

	async function scanPrinters() {
		detecting = true;
		detectedPrinters = [];
		try {
			detectedPrinters = await api.detectPrinters();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			detecting = false;
		}
	}

	function openAddModal(detected: DetectedPrinter) {
		addPrinterUri = detected.uri;
		addPrinterName = detected.model.replace(/[^a-zA-Z0-9]/g, '_');
		addPrinterDriver = '';
		addPrinterLocation = '';
		addPrinterShared = true;
		availableDrivers = [];
		showAddModal = true;
		loadDrivers(detected.uri);
	}

	async function loadDrivers(uri: string) {
		driversLoading = true;
		try {
			availableDrivers = await api.getPrinterDrivers(uri);
			if (availableDrivers.length > 0) {
				addPrinterDriver = availableDrivers[0].id;
			}
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
		} finally {
			driversLoading = false;
		}
	}

	async function confirmAddPrinter() {
		if (!addPrinterName || !addPrinterDriver) return;

		adding = true;

		try {
			await api.addPrinter({
				name: addPrinterName,
				uri: addPrinterUri,
				driver: addPrinterDriver,
				location: addPrinterLocation || undefined,
				shared: addPrinterShared
			});
			showAddModal = false;
			await loadPrinters();
			await loadStatus();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.addPrinter));
		} finally {
			adding = false;
		}
	}

	async function confirmRemove() {
		const target = removeTarget;
		removeTarget = null;
		if (!target) return;

		try {
			await api.removePrinter(target.name);
			await loadPrinters();
			await loadStatus();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function toggleShared(printer: CupsPrinter) {
		try {
			await api.updatePrinter(printer.name, { shared: !printer.shared });
			await loadPrinters();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function setDefault(printer: CupsPrinter) {
		try {
			await api.updatePrinter(printer.name, { is_default: true });
			await loadPrinters();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function printTest(printer: CupsPrinter) {
		try {
			await api.printTestPage(printer.name);
			toasts.success($t.fileService.cups.testPageSent);
			await loadJobs();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function cancelJob(jobId: number) {
		try {
			await api.cancelPrintJob(jobId);
			await loadJobs();
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	function getStateIcon(state: string): string {
		switch (state) {
			case 'idle': return 'mdi:printer-check';
			case 'processing': return 'mdi:printer';
			case 'stopped': return 'mdi:printer-off';
			default: return 'mdi:printer-alert';
		}
	}

	function stateTone(state: string): string {
		switch (state) {
			case 'idle': return 'success';
			case 'processing': return 'info';
			case 'stopped': return '';
			default: return 'danger';
		}
	}

	function stateLabel(state: string): string {
		return ($t.fileService.cups.state as Record<string, string>)[state] ?? state;
	}
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.printer} description={$t.settingsUi.descriptions.printer} icon="mdi:printer-outline" />

	{#if cupsLoading && !cupsStatus}
		<SettingsState kind="loading" message={$t.common.loading} />
	{:else if cupsError && !cupsStatus}
		<SettingsState kind="error" message={cupsError} onretry={loadStatus} />
	{:else if cupsStatus}
		<SettingsSection title={$t.fileService.cups.title} description={$t.fileService.cups.description} icon="mdi:printer">
			{#snippet actions()}
				{#if cupsStatus?.enabled}
					<span class="settings-badge" class:success={cupsStatus.running} class:danger={!cupsStatus.running}>
						{cupsStatus.running ? $t.fileService.cups.running : $t.fileService.cups.stopped}
					</span>
				{/if}
			{/snippet}
			<SettingsRow label={$t.fileService.cups.enable} hint={$t.fileService.cups.enableHint}>
				<SettingsToggle label={$t.fileService.cups.enable} checked={cupsStatus.enabled} disabled={cupsToggling} onchange={toggleCups} />
			</SettingsRow>
		</SettingsSection>

		{#if cupsStatus.enabled}
			<SettingsSection title={$t.fileService.cups.printers} icon="mdi:printer-settings">
				{#snippet actions()}
					<button class="settings-button" on:click={scanPrinters} disabled={detecting}>
						<Icon icon={detecting ? 'mdi:loading' : 'mdi:magnify-scan'} width="16" class={detecting ? 'animate-spin' : ''} />
						{detecting ? $t.fileService.cups.scanning : $t.fileService.cups.scan}
					</button>
				{/snippet}

				{#if printersLoading && printers.length === 0}
					<SettingsState kind="loading" message={$t.common.loading} />
				{:else if printers.length === 0}
					<SettingsState kind="empty" message={$t.fileService.cups.noPrinters} />
					<p class="settings-help">{$t.fileService.cups.noPrintersHint}</p>
				{:else}
					<div class="settings-table-wrap">
						<table class="settings-table">
							<thead>
								<tr>
									<th>{$t.fileService.cups.printerName}</th>
									<th>{$t.fileService.cups.status}</th>
									<th>{$t.fileService.cups.shared}</th>
									<th></th>
								</tr>
							</thead>
							<tbody>
								{#each printers as printer (printer.name)}
									<tr>
										<td>
											<div class="printer-name">
												<strong>{printer.name}</strong>
												{#if printer.is_default}
													<span class="settings-badge info">{$t.fileService.cups.default}</span>
												{/if}
											</div>
											<div class="settings-mono subtle">{printer.model || printer.uri}</div>
										</td>
										<td><span class="settings-badge {stateTone(printer.state)}">{stateLabel(printer.state)}</span></td>
										<td>
											<SettingsToggle label={`${$t.fileService.cups.shared} — ${printer.name}`} checked={printer.shared} onchange={() => toggleShared(printer)} />
										</td>
										<td class="row-actions">
											<button class="settings-button ghost" title={$t.fileService.cups.testPage} aria-label={$t.fileService.cups.testPage} on:click={() => printTest(printer)}>
												<Icon icon="mdi:file-document-check-outline" width="18" />
											</button>
											{#if !printer.is_default}
												<button class="settings-button ghost" title={$t.fileService.cups.setDefault} aria-label={$t.fileService.cups.setDefault} on:click={() => setDefault(printer)}>
													<Icon icon="mdi:star-outline" width="18" />
												</button>
											{/if}
											<button class="settings-button ghost danger-hover" title={$t.fileService.cups.removePrinter} aria-label={$t.fileService.cups.removePrinter} on:click={() => (removeTarget = printer)}>
												<Icon icon="mdi:delete-outline" width="18" />
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}

				{#if newDetected.length > 0}
					<h4 class="sub-title">{$t.fileService.cups.detected}</h4>
					<div class="settings-table-wrap">
						<table class="settings-table">
							<tbody>
								{#each newDetected as detected (detected.uri)}
									<tr>
										<td>
											<strong>{detected.model}</strong>
											<div class="settings-mono subtle">{detected.uri}</div>
										</td>
										<td class="row-actions">
											<button class="settings-button primary" on:click={() => openAddModal(detected)} disabled={showAddModal && addPrinterUri === detected.uri}>
												<Icon icon="mdi:plus" width="16" />
												{$t.fileService.cups.add}
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}

				{#if showAddModal}
					<form class="inline-form" on:submit|preventDefault={confirmAddPrinter}>
						<h4 class="sub-title">{$t.fileService.cups.addPrinter}</h4>
						<div class="settings-grid">
							<div class="settings-field">
								<label for="printer-name">{$t.fileService.cups.printerName}</label>
								<input id="printer-name" type="text" bind:value={addPrinterName} placeholder="My_Printer" />
							</div>
							<div class="settings-field">
								<label for="printer-location">{$t.fileService.cups.location}</label>
								<input id="printer-location" type="text" bind:value={addPrinterLocation} />
							</div>
							<div class="settings-field">
								<label for="printer-driver">{$t.fileService.cups.selectDriver}</label>
								{#if driversLoading}
									<span class="settings-help">{$t.fileService.cups.loadingDrivers}</span>
								{:else}
									<select id="printer-driver" bind:value={addPrinterDriver}>
										{#each availableDrivers as driver (driver.id)}
											<option value={driver.id}>{driver.name}</option>
										{/each}
									</select>
								{/if}
							</div>
							<div class="settings-field">
								<span class="field-label">{$t.fileService.cups.printerUri}</span>
								<span class="settings-mono uri">{addPrinterUri}</span>
							</div>
						</div>
						<SettingsRow label={$t.fileService.cups.shared}>
							<SettingsToggle label={$t.fileService.cups.shared} bind:checked={addPrinterShared} />
						</SettingsRow>
						<div class="settings-actions-buttons">
							<button type="button" class="settings-button" on:click={() => (showAddModal = false)} disabled={adding}>{$t.common.cancel}</button>
							<button type="submit" class="settings-button primary" disabled={adding || !addPrinterName || !addPrinterDriver}>
								{adding ? $t.fileService.cups.adding : $t.fileService.cups.add}
							</button>
						</div>
					</form>
				{/if}
			</SettingsSection>

			{#if jobs.length > 0}
				<SettingsSection title={$t.fileService.cups.queue} icon="mdi:tray-full">
					{#snippet actions()}
						<button class="settings-button ghost" on:click={loadJobs} disabled={jobsLoading} title={$t.common.refresh} aria-label={$t.common.refresh}>
							<Icon icon="mdi:refresh" width="18" />
						</button>
					{/snippet}
					<div class="settings-table-wrap">
						<table class="settings-table">
							<tbody>
								{#each jobs as job (job.id)}
									<tr>
										<td><strong>{job.title}</strong></td>
										<td>{job.printer}</td>
										<td><span class="settings-badge {stateTone(job.state)}">{stateLabel(job.state)}</span></td>
										<td class="row-actions">
											<button class="settings-button ghost danger-hover" on:click={() => cancelJob(job.id)} title={$t.fileService.cups.cancelJob} aria-label={$t.fileService.cups.cancelJob}>
												<Icon icon="mdi:close" width="18" />
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</SettingsSection>
			{/if}

			<div class="settings-notice">
				<Icon icon="mdi:information-outline" width="18" />
				<span><strong>{$t.fileService.cups.protocols}</strong> — {$t.fileService.cups.protocolsHint}</span>
			</div>
		{/if}
	{/if}
</div>

<ConfirmModal
	visible={removeTarget !== null}
	title={$t.fileService.cups.confirmRemove}
	message={`${removeTarget?.name ?? ''} — ${$t.fileService.cups.confirmRemoveMessage}`}
	confirmLabel={$t.fileService.cups.removePrinter}
	cancelLabel={$t.common.cancel}
	danger
	on:confirm={confirmRemove}
	on:cancel={() => (removeTarget = null)}
/>

<style>
	.printer-name { display: flex; align-items: center; gap: 8px; }
	.subtle { color: #64748b; font-size: 12px; }
	.row-actions { display: flex; align-items: center; justify-content: flex-end; gap: 4px; }
	.sub-title { margin: 16px 0 8px; font-size: 13px; font-weight: 600; color: #334155; }
	.inline-form { display: flex; flex-direction: column; gap: 12px; margin-top: 16px; padding: 16px; border: 1px solid #e2e8f0; border-radius: 10px; background: #f8fafc; }
	.inline-form .sub-title { margin: 0; }
	.field-label { font-size: 13px; font-weight: 500; color: #334155; }
	.uri { word-break: break-all; font-size: 12px; color: #475569; }
</style>
