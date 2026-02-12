<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api, type CupsStatus, type CupsPrinter, type DetectedPrinter, type PrinterDriver, type PrintJob } from '$lib/stores/api';

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

	// Add printer modal
	let showAddModal = false;
	let addPrinterUri = '';
	let addPrinterName = '';
	let addPrinterDriver = '';
	let addPrinterLocation = '';
	let addPrinterShared = true;
	let availableDrivers: PrinterDriver[] = [];
	let driversLoading = false;
	let addError: string | null = null;
	let adding = false;

	// Remove printer modal
	let showRemoveModal = false;
	let removeTarget: CupsPrinter | null = null;

	// Jobs
	let jobs: PrintJob[] = [];
	let jobsLoading = false;

	// Test page feedback
	let testPagePrinter: string | null = null;

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
			cupsError = e instanceof Error ? e.message : 'Failed to load CUPS status';
		} finally {
			cupsLoading = false;
		}
	}

	async function toggleCups() {
		if (!cupsStatus || cupsToggling) return;

		cupsToggling = true;
		cupsError = null;

		try {
			if (cupsStatus.enabled) {
				await api.disableCups();
			} else {
				await api.enableCups();
			}
			await loadStatus();
		} catch (e) {
			cupsError = e instanceof Error ? e.message : 'Failed to toggle CUPS';
		} finally {
			cupsToggling = false;
		}
	}

	async function loadPrinters() {
		printersLoading = true;
		try {
			printers = await api.getCupsPrinters();
		} catch (e) {
			console.error('Failed to load printers:', e);
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
			console.error('Failed to load jobs:', e);
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
			console.error('Failed to detect printers:', e);
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
		addError = null;
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
			console.error('Failed to load drivers:', e);
		} finally {
			driversLoading = false;
		}
	}

	async function confirmAddPrinter() {
		if (!addPrinterName || !addPrinterDriver) return;

		adding = true;
		addError = null;

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
			addError = e instanceof Error ? e.message : 'Failed to add printer';
		} finally {
			adding = false;
		}
	}

	function openRemoveModal(printer: CupsPrinter) {
		removeTarget = printer;
		showRemoveModal = true;
	}

	async function confirmRemove() {
		if (!removeTarget) return;

		try {
			await api.removePrinter(removeTarget.name);
			showRemoveModal = false;
			removeTarget = null;
			await loadPrinters();
			await loadStatus();
		} catch (e) {
			console.error('Failed to remove printer:', e);
		}
	}

	async function toggleShared(printer: CupsPrinter) {
		try {
			await api.updatePrinter(printer.name, { shared: !printer.shared });
			await loadPrinters();
		} catch (e) {
			console.error('Failed to toggle sharing:', e);
		}
	}

	async function setDefault(printer: CupsPrinter) {
		try {
			await api.updatePrinter(printer.name, { is_default: true });
			await loadPrinters();
		} catch (e) {
			console.error('Failed to set default printer:', e);
		}
	}

	async function printTest(printer: CupsPrinter) {
		try {
			await api.printTestPage(printer.name);
			testPagePrinter = printer.name;
			setTimeout(() => { testPagePrinter = null; }, 3000);
			await loadJobs();
		} catch (e) {
			console.error('Failed to print test page:', e);
		}
	}

	async function cancelJob(jobId: number) {
		try {
			await api.cancelPrintJob(jobId);
			await loadJobs();
		} catch (e) {
			console.error('Failed to cancel job:', e);
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

	function getStateColor(state: string): string {
		switch (state) {
			case 'idle': return 'text-green-500';
			case 'processing': return 'text-blue-500';
			case 'stopped': return 'text-slate-400';
			default: return 'text-red-500';
		}
	}
</script>

<div class="printer-settings">
	<div class="section">
		<div class="section-header">
			<Icon icon="mdi:printer" class="w-6 h-6" />
			<div class="section-info">
				<h2>{$t.fileService?.cups?.title || 'Printer Sharing'}</h2>
				<p>{$t.fileService?.cups?.description || 'Share USB printers across your network'}</p>
			</div>
		</div>

		{#if cupsLoading}
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
				<span>{$t.common.loading}</span>
			</div>
		{:else if cupsError}
			<div class="error-state">
				<Icon icon="mdi:alert-circle" class="w-5 h-5" />
				<span>{cupsError}</span>
				<button class="retry-btn" on:click={loadStatus}>
					{$t.common.retry || 'Retry'}
				</button>
			</div>
		{:else if cupsStatus}
			<div class="service-content">
				<!-- Enable toggle -->
				<div class="service-row">
					<div class="service-label">
						<span class="label">{$t.fileService?.cups?.enable || 'Enable Printer Sharing'}</span>
						<span class="sublabel">{$t.fileService?.cups?.enableHint || 'Start CUPS service'}</span>
					</div>
					<button
						class="toggle-switch"
						class:active={cupsStatus.enabled}
						class:loading={cupsToggling}
						on:click={toggleCups}
						disabled={cupsToggling}
					>
						<span class="toggle-knob"></span>
					</button>
				</div>

				{#if cupsStatus.enabled}
					<!-- Status -->
					<div class="service-row">
						<div class="service-label">
							<span class="label">{$t.fileService?.cups?.status || 'Status'}</span>
						</div>
						<div class="status-badge" class:running={cupsStatus.running}>
							<Icon icon={cupsStatus.running ? 'mdi:check-circle' : 'mdi:close-circle'} class="w-4 h-4" />
							{cupsStatus.running
								? ($t.fileService?.cups?.running || 'Running')
								: ($t.fileService?.cups?.stopped || 'Stopped')}
						</div>
					</div>

					<!-- Printers section -->
					<div class="subsection">
						<div class="subsection-header">
							<h3>{$t.fileService?.cups?.printers || 'Printers'}</h3>
							<button class="btn-small" on:click={scanPrinters} disabled={detecting}>
								{#if detecting}
									<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
									{$t.fileService?.cups?.scanning || 'Scanning...'}
								{:else}
									<Icon icon="mdi:magnify-scan" class="w-4 h-4" />
									{$t.fileService?.cups?.scan || 'Scan for printers'}
								{/if}
							</button>
						</div>

						{#if printersLoading}
							<div class="loading-state small">
								<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
							</div>
						{:else if printers.length === 0}
							<div class="empty-hint">
								<Icon icon="mdi:printer-off" class="w-8 h-8 text-slate-300" />
								<p>{$t.fileService?.cups?.noPrinters || 'No printers configured'}</p>
								<span class="hint-text">{$t.fileService?.cups?.noPrintersHint || 'Connect a USB printer and scan'}</span>
							</div>
						{:else}
							{#each printers as printer}
								<div class="printer-card">
									<div class="printer-info">
										<Icon icon={getStateIcon(printer.state)} class="w-6 h-6 {getStateColor(printer.state)}" />
										<div class="printer-details">
											<div class="printer-name">
												{printer.name}
												{#if printer.is_default}
													<span class="default-badge">{$t.fileService?.cups?.default || 'Default'}</span>
												{/if}
											</div>
											<span class="printer-model">{printer.model || printer.uri}</span>
											<span class="printer-state">
												{$t.fileService?.cups?.state?.[printer.state] || printer.state}
												{#if printer.shared}
													· {$t.fileService?.cups?.shared || 'Shared'}
												{/if}
											</span>
										</div>
									</div>
									<div class="printer-actions">
										{#if testPagePrinter === printer.name}
											<span class="test-sent">
												<Icon icon="mdi:check" class="w-4 h-4" />
											</span>
										{:else}
											<button class="btn-icon" title={$t.fileService?.cups?.testPage || 'Test Page'} on:click={() => printTest(printer)}>
												<Icon icon="mdi:file-document-check" class="w-4 h-4" />
											</button>
										{/if}
										<button
											class="btn-icon"
											title={printer.shared ? ($t.fileService?.cups?.notShared || 'Unshare') : ($t.fileService?.cups?.shared || 'Share')}
											on:click={() => toggleShared(printer)}
										>
											<Icon icon={printer.shared ? 'mdi:share' : 'mdi:share-off'} class="w-4 h-4" />
										</button>
										{#if !printer.is_default}
											<button class="btn-icon" title={$t.fileService?.cups?.setDefault || 'Set as default'} on:click={() => setDefault(printer)}>
												<Icon icon="mdi:star-outline" class="w-4 h-4" />
											</button>
										{/if}
										<button class="btn-icon danger" title={$t.fileService?.cups?.removePrinter || 'Remove'} on:click={() => openRemoveModal(printer)}>
											<Icon icon="mdi:delete-outline" class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/each}
						{/if}

						<!-- Detected printers -->
						{#if detectedPrinters.length > 0}
							<div class="detected-section">
								<h4>{$t.fileService?.cups?.detected || 'Detected Printers'}</h4>
								{#each detectedPrinters as detected}
									<div class="detected-card">
										<Icon icon="mdi:usb" class="w-5 h-5 text-slate-400" />
										<div class="detected-info">
											<span class="detected-model">{detected.model}</span>
											<span class="detected-uri">{detected.uri}</span>
										</div>
										<button class="btn-small primary" on:click={() => openAddModal(detected)}>
											<Icon icon="mdi:plus" class="w-4 h-4" />
											{$t.fileService?.cups?.add || 'Add'}
										</button>
									</div>
								{/each}
							</div>
						{:else if !detecting && detectedPrinters.length === 0 && printers.length === 0}
							<!-- No detected hint is already shown in empty state -->
						{/if}
					</div>

					<!-- Print queue -->
					{#if jobs.length > 0}
						<div class="subsection">
							<div class="subsection-header">
								<h3>{$t.fileService?.cups?.queue || 'Print Queue'}</h3>
								<button class="btn-small" on:click={loadJobs}>
									<Icon icon="mdi:refresh" class="w-4 h-4" />
								</button>
							</div>
							{#each jobs as job}
								<div class="job-row">
									<Icon icon="mdi:file-document" class="w-4 h-4 text-slate-400" />
									<span class="job-title">{job.title}</span>
									<span class="job-printer">{job.printer}</span>
									<span class="job-state">{job.state}</span>
									<button class="btn-icon danger small" on:click={() => cancelJob(job.id)}>
										<Icon icon="mdi:close" class="w-3 h-3" />
									</button>
								</div>
							{/each}
						</div>
					{/if}

					<!-- Protocols info -->
					<div class="info-box">
						<Icon icon="mdi:information-outline" class="w-5 h-5" />
						<div class="info-content">
							<p><strong>{$t.fileService?.cups?.protocols || 'Sharing Protocols'}</strong></p>
							<p>{$t.fileService?.cups?.protocolsHint || 'Printers shared via IPP/AirPrint.'}</p>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<!-- Add Printer Modal -->
{#if showAddModal}
	<div class="modal-overlay" on:click={() => showAddModal = false}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:printer-plus" class="w-5 h-5" />
					{$t.fileService?.cups?.addPrinter || 'Add Printer'}
				</h2>
				<button class="modal-close" on:click={() => showAddModal = false}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body">
				{#if addError}
					<div class="error-message">
						<Icon icon="mdi:alert-circle" class="w-4 h-4" />
						{addError}
					</div>
				{/if}

				<div class="form-group">
					<label for="printer-name">{$t.fileService?.cups?.printerName || 'Printer Name'}</label>
					<input id="printer-name" type="text" bind:value={addPrinterName} placeholder="My_Printer" />
				</div>

				<div class="form-group">
					<label>{$t.fileService?.cups?.printerUri || 'Connection'}</label>
					<div class="uri-display">{addPrinterUri}</div>
				</div>

				<div class="form-group">
					<label for="printer-driver">{$t.fileService?.cups?.selectDriver || 'Select Driver'}</label>
					{#if driversLoading}
						<div class="loading-inline">
							<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
							<span>{$t.fileService?.cups?.loadingDrivers || 'Loading drivers...'}</span>
						</div>
					{:else}
						<select id="printer-driver" bind:value={addPrinterDriver}>
							{#each availableDrivers as driver}
								<option value={driver.id}>{driver.name}</option>
							{/each}
						</select>
					{/if}
				</div>

				<div class="form-group">
					<label for="printer-location">{$t.fileService?.cups?.location || 'Location'}</label>
					<input id="printer-location" type="text" bind:value={addPrinterLocation} placeholder="Office" />
				</div>

				<div class="form-group-inline">
					<label>
						<input type="checkbox" bind:checked={addPrinterShared} />
						<span>{$t.fileService?.cups?.shared || 'Shared on network'}</span>
					</label>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => showAddModal = false}>
					{$t.common.cancel}
				</button>
				<button class="btn-primary" on:click={confirmAddPrinter} disabled={adding || !addPrinterName || !addPrinterDriver}>
					{#if adding}
						<Icon icon="mdi:loading" class="w-4 h-4 animate-spin" />
						{$t.fileService?.cups?.adding || 'Adding...'}
					{:else}
						<Icon icon="mdi:plus" class="w-4 h-4" />
						{$t.fileService?.cups?.add || 'Add'}
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Remove Confirmation Modal -->
{#if showRemoveModal && removeTarget}
	<div class="modal-overlay" on:click={() => { showRemoveModal = false; removeTarget = null; }}>
		<div class="modal small" on:click|stopPropagation>
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:printer-off" class="w-5 h-5 text-red-500" />
					{$t.fileService?.cups?.confirmRemove || 'Remove this printer?'}
				</h2>
			</div>
			<div class="modal-body">
				<p><strong>{removeTarget.name}</strong></p>
				<p class="hint-text">{$t.fileService?.cups?.confirmRemoveMessage || 'The printer will no longer be shared.'}</p>
			</div>
			<div class="modal-footer">
				<button class="btn-secondary" on:click={() => { showRemoveModal = false; removeTarget = null; }}>
					{$t.common.cancel}
				</button>
				<button class="btn-danger" on:click={confirmRemove}>
					<Icon icon="mdi:delete" class="w-4 h-4" />
					{$t.fileService?.cups?.removePrinter || 'Remove'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.printer-settings {
		padding: 24px;
	}

	.section {
		margin-bottom: 32px;
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

	.service-content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.service-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
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

	/* Toggle switch */
	.toggle-switch {
		position: relative;
		width: 48px;
		height: 28px;
		background: #d1d5db;
		border-radius: 14px;
		cursor: pointer;
		transition: background 0.2s ease;
	}

	.toggle-switch.active {
		background: #3b82f6;
	}

	.toggle-switch.loading {
		opacity: 0.6;
		cursor: wait;
	}

	.toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 24px;
		height: 24px;
		background: white;
		border-radius: 12px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle-switch.active .toggle-knob {
		transform: translateX(20px);
	}

	/* Status badge */
	.status-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		background: #fef2f2;
		color: #dc2626;
	}

	.status-badge.running {
		background: #dcfce7;
		color: #16a34a;
	}

	/* Subsections */
	.subsection {
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 10px;
		padding: 16px;
	}

	.subsection-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}

	.subsection-header h3 {
		font-size: 14px;
		font-weight: 600;
		color: #374151;
	}

	/* Printer card */
	.printer-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		margin-bottom: 8px;
	}

	.printer-card:last-of-type {
		margin-bottom: 0;
	}

	.printer-info {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.printer-details {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.printer-name {
		font-size: 14px;
		font-weight: 500;
		color: #1f2937;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.default-badge {
		font-size: 10px;
		font-weight: 600;
		padding: 2px 6px;
		border-radius: 4px;
		background: #dbeafe;
		color: #2563eb;
		text-transform: uppercase;
	}

	.printer-model {
		font-size: 12px;
		color: #6b7280;
	}

	.printer-state {
		font-size: 11px;
		color: #9ca3af;
	}

	.printer-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.test-sent {
		color: #16a34a;
		padding: 6px;
	}

	/* Detected printers */
	.detected-section {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid #e5e7eb;
	}

	.detected-section h4 {
		font-size: 13px;
		font-weight: 500;
		color: #6b7280;
		margin-bottom: 8px;
	}

	.detected-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		background: white;
		border: 1px dashed #d1d5db;
		border-radius: 8px;
		margin-bottom: 6px;
	}

	.detected-info {
		flex: 1;
		display: flex;
		flex-direction: column;
	}

	.detected-model {
		font-size: 13px;
		font-weight: 500;
		color: #374151;
	}

	.detected-uri {
		font-size: 11px;
		color: #9ca3af;
		font-family: monospace;
	}

	/* Job row */
	.job-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 0;
		border-bottom: 1px solid #f3f4f6;
	}

	.job-row:last-child {
		border-bottom: none;
	}

	.job-title {
		flex: 1;
		font-size: 13px;
		color: #374151;
	}

	.job-printer {
		font-size: 12px;
		color: #9ca3af;
	}

	.job-state {
		font-size: 12px;
		color: #6b7280;
		padding: 2px 8px;
		background: #f3f4f6;
		border-radius: 4px;
	}

	/* Buttons */
	.btn-small {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		background: white;
		font-size: 13px;
		color: #374151;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-small:hover:not(:disabled) {
		background: #f9fafb;
	}

	.btn-small:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-small.primary {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.btn-small.primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		background: transparent;
		border-radius: 6px;
		color: #6b7280;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-icon:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-icon.danger:hover {
		background: #fef2f2;
		color: #dc2626;
	}

	.btn-icon.small {
		width: 24px;
		height: 24px;
	}

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
		border: none;
		cursor: pointer;
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
		padding: 10px 16px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}

	.btn-danger {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 8px;
		font-size: 14px;
		color: #dc2626;
		cursor: pointer;
	}

	.btn-danger:hover {
		background: #fee2e2;
	}

	/* Empty and loading states */
	.empty-hint {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 24px;
		text-align: center;
	}

	.empty-hint p {
		font-size: 14px;
		color: #6b7280;
	}

	.hint-text {
		font-size: 12px;
		color: #9ca3af;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px 20px;
		color: #9ca3af;
	}

	.loading-state.small {
		padding: 20px;
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
		cursor: pointer;
	}

	.retry-btn:hover {
		background: #fef2f2;
	}

	/* Info box */
	.info-box {
		display: flex;
		gap: 12px;
		padding: 16px;
		background: #eff6ff;
		border: 1px solid #bfdbfe;
		border-radius: 8px;
		color: #1d4ed8;
	}

	.info-box .info-content {
		flex: 1;
	}

	.info-box p {
		font-size: 13px;
		line-height: 1.5;
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

	.modal.small {
		max-width: 380px;
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
		border: none;
		background: none;
		cursor: pointer;
	}

	.modal-close:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.modal-body {
		padding: 20px;
	}

	.modal-body p {
		font-size: 14px;
		color: #374151;
		margin-bottom: 8px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 20px;
		border-top: 1px solid #e5e7eb;
	}

	/* Form */
	.form-group {
		margin-bottom: 16px;
	}

	.form-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #374151;
		margin-bottom: 6px;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		background: white;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.form-group-inline {
		margin-bottom: 16px;
	}

	.form-group-inline label {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: #374151;
		cursor: pointer;
	}

	.form-group-inline input[type="checkbox"] {
		width: 18px;
		height: 18px;
		accent-color: #3b82f6;
	}

	.uri-display {
		font-family: monospace;
		font-size: 12px;
		padding: 10px 12px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		color: #6b7280;
		word-break: break-all;
	}

	.loading-inline {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 0;
		color: #9ca3af;
		font-size: 13px;
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

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
