<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';
	import { api } from '$lib/stores/api';
	import type { BrowsableLocation } from '$lib/stores/api';

	// Props
	export let value: string = '';
	export let placeholder: string = '/storage/shares/documents';
	export let label: string = '';
	export let hint: string = '';
	export let disabled: boolean = false;

	const dispatch = createEventDispatcher<{ change: string }>();

	// State
	let showModal = false;
	let locations: BrowsableLocation[] = [];
	let selectedLocation: BrowsableLocation | null = null;
	let currentPath = '';
	let folders: { name: string; path: string }[] = [];
	let loading = false;
	let locationsLoading = false;
	let error: string | null = null;
	let breadcrumbs: { name: string; path: string }[] = [];

	// Load available locations
	async function loadLocations() {
		locationsLoading = true;
		try {
			locations = await api.getLocations();
			// Auto-select first location if available
			if (locations.length > 0 && !selectedLocation) {
				selectLocation(locations[0]);
			}
		} catch (e) {
			console.error('Failed to load locations:', e);
			locations = [];
		} finally {
			locationsLoading = false;
		}
	}

	// Select a location and load its root folders
	function selectLocation(loc: BrowsableLocation) {
		selectedLocation = loc;
		currentPath = '';
		loadFolders('');
	}

	// Get location ID for API calls
	function getLocationId(): string | undefined {
		if (!selectedLocation) return undefined;
		// The location.id already contains the full ID (e.g., "home-abc123", "share-xyz", "volume-vol1")
		return selectedLocation.id;
	}

	// Get the full path for display
	function getFullPath(): string {
		if (!selectedLocation) return '';
		const basePath = selectedLocation.path;
		if (!currentPath) return basePath;
		return `${basePath}/${currentPath}`.replace(/\/+/g, '/');
	}

	// Load folders for current path
	async function loadFolders(path: string) {
		loading = true;
		error = null;

		try {
			const locationId = getLocationId();
			console.log('FolderPicker: Loading folders', { path, locationId });
			const files = await api.getFiles(path, locationId);
			console.log('FolderPicker: API response', files);
			folders = files
				.filter((f) => f.type === 'folder')
				.filter((f) => !f.name.startsWith('.'))
				.map((f) => ({
					name: f.name,
					path: path ? `${path}/${f.name}` : f.name
				}))
				.sort((a, b) => a.name.localeCompare(b.name));
			console.log('FolderPicker: Filtered folders', folders);

			currentPath = path;
			buildBreadcrumbs(path);
		} catch (e) {
			console.error('FolderPicker: Error loading folders', e);
			error = e instanceof Error ? e.message : 'Failed to load folders';
			folders = [];
		} finally {
			loading = false;
		}
	}

	function buildBreadcrumbs(path: string) {
		breadcrumbs = [{ name: selectedLocation?.name || 'Root', path: '' }];

		if (path) {
			const parts = path.split('/').filter(Boolean);
			let currentBreadcrumbPath = '';
			for (const part of parts) {
				currentBreadcrumbPath = currentBreadcrumbPath ? `${currentBreadcrumbPath}/${part}` : part;
				breadcrumbs.push({ name: part, path: currentBreadcrumbPath });
			}
		}
	}

	function openModal() {
		if (disabled) return;
		showModal = true;
		selectedLocation = null;
		currentPath = '';
		folders = [];
		loadLocations();
	}

	function closeModal() {
		showModal = false;
	}

	function navigateTo(path: string) {
		loadFolders(path);
	}

	function navigateUp() {
		if (!currentPath) return;
		const parentPath = currentPath.split('/').slice(0, -1).join('/');
		navigateTo(parentPath);
	}

	function selectFolder(path: string) {
		loadFolders(path);
	}

	function confirmSelection() {
		value = getFullPath();
		dispatch('change', value);
		closeModal();
	}

	function handleInputChange() {
		dispatch('change', value);
	}

	// Get icon for location type
	function getLocationIcon(loc: BrowsableLocation): string {
		if (loc.type === 'home') return 'mdi:folder-home';
		if (loc.type === 'share') return 'mdi:folder-network';
		if (loc.type === 'volume') return 'mdi:harddisk';
		return 'mdi:folder';
	}
</script>

<div class="folder-picker">
	{#if label}
		<label class="picker-label">{label}</label>
	{/if}

	<div class="picker-input-group">
		<input
			type="text"
			class="picker-input"
			bind:value
			on:input={handleInputChange}
			{placeholder}
			{disabled}
		/>
		<button
			type="button"
			class="picker-browse-btn"
			on:click={openModal}
			{disabled}
			title={$t.folderPicker?.browse || 'Browse'}
		>
			<Icon icon="mdi:folder-search" class="w-4 h-4" />
		</button>
	</div>

	{#if hint}
		<span class="picker-hint">{hint}</span>
	{/if}
</div>

<!-- Folder Browser Modal -->
{#if showModal}
	<div class="modal-overlay" on:click={closeModal}>
		<div class="modal folder-modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>
					<Icon icon="mdi:folder-open" class="w-5 h-5" />
					{$t.folderPicker?.title || 'Select Folder'}
				</h2>
				<button class="modal-close" on:click={closeModal}>
					<Icon icon="mdi:close" class="w-5 h-5" />
				</button>
			</div>

			<div class="modal-body folder-browser">
				{#if locationsLoading}
					<div class="loading-state">
						<Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
						<span>{$t.common.loading}</span>
					</div>
				{:else if locations.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:folder-off-outline" class="w-8 h-8" />
						<span>{$t.folderPicker?.noLocations || 'No locations available'}</span>
					</div>
				{:else}
					<!-- Location selector -->
					<div class="location-selector">
						<label>{$t.folderPicker?.location || 'Location'}:</label>
						<div class="location-chips">
							{#each locations as loc}
								<button
									class="location-chip"
									class:active={selectedLocation?.id === loc.id}
									on:click={() => selectLocation(loc)}
								>
									<Icon icon={getLocationIcon(loc)} class="w-4 h-4" />
									<span>{loc.name}</span>
								</button>
							{/each}
						</div>
					</div>

					{#if selectedLocation}
						<!-- Breadcrumb navigation -->
						<div class="breadcrumbs">
							{#each breadcrumbs as crumb, i}
								{#if i > 0}
									<Icon icon="mdi:chevron-right" class="w-4 h-4 breadcrumb-separator" />
								{/if}
								<button
									class="breadcrumb"
									class:active={crumb.path === currentPath}
									on:click={() => navigateTo(crumb.path)}
								>
									{#if i === 0}
										<Icon icon={getLocationIcon(selectedLocation)} class="w-4 h-4" />
									{/if}
									{crumb.name}
								</button>
							{/each}
						</div>

						<!-- Current path display -->
						<div class="current-path">
							<Icon icon="mdi:folder" class="w-4 h-4 folder-icon" />
							<span>{getFullPath()}</span>
						</div>

						<!-- Folder list -->
						<div class="folder-list">
							{#if loading}
								<div class="loading-state">
									<Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
								</div>
							{:else if error}
								<div class="error-state">
									<Icon icon="mdi:alert-circle" class="w-6 h-6" />
									<span>{error}</span>
								</div>
							{:else}
								<!-- Navigate up -->
								{#if currentPath}
									<button class="folder-item" on:click={navigateUp}>
										<Icon icon="mdi:folder-arrow-up" class="w-5 h-5 folder-icon-up" />
										<span>..</span>
									</button>
								{/if}

								{#if folders.length === 0 && !currentPath}
									<div class="empty-folder-state">
										<span>{$t.folderPicker?.emptyFolder || 'This folder is empty'}</span>
									</div>
								{:else if folders.length === 0}
									<div class="empty-folder-state">
										<span>{$t.folderPicker?.noSubfolders || 'No subfolders'}</span>
									</div>
								{:else}
									{#each folders as folder}
										<button
											class="folder-item"
											on:click={() => selectFolder(folder.path)}
											on:dblclick={confirmSelection}
										>
											<Icon icon="mdi:folder" class="w-5 h-5 folder-icon" />
											<span>{folder.name}</span>
										</button>
									{/each}
								{/if}
							{/if}
						</div>
					{/if}
				{/if}
			</div>

			<div class="modal-footer">
				<div class="selected-path">
					<span class="selected-label">{$t.folderPicker?.selected || 'Selected'}:</span>
					<code>{getFullPath() || '-'}</code>
				</div>
				<div class="footer-buttons">
					<button class="btn-secondary" on:click={closeModal}>
						{$t.common.cancel}
					</button>
					<button class="btn-primary" on:click={confirmSelection} disabled={!selectedLocation}>
						{$t.folderPicker?.select || 'Select'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.folder-picker {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.picker-label {
		font-size: 14px;
		font-weight: 500;
		color: #374151;
	}

	.picker-input-group {
		display: flex;
		gap: 0;
	}

	.picker-input {
		flex: 1;
		padding: 10px 12px;
		border: 1px solid #e5e7eb;
		border-right: none;
		border-radius: 8px 0 0 8px;
		font-size: 14px;
		font-family: monospace;
	}

	.picker-input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.picker-input:disabled {
		background: #f3f4f6;
		color: #9ca3af;
	}

	.picker-browse-btn {
		padding: 10px 14px;
		background: #f3f4f6;
		border: 1px solid #e5e7eb;
		border-radius: 0 8px 8px 0;
		color: #6b7280;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.picker-browse-btn:hover:not(:disabled) {
		background: #e5e7eb;
		color: #374151;
	}

	.picker-browse-btn:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.picker-hint {
		font-size: 12px;
		color: #9ca3af;
	}

	/* Modal styles */
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
		background: white;
		border-radius: 12px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
		display: flex;
		flex-direction: column;
	}

	.folder-modal {
		width: 90%;
		max-width: 600px;
		max-height: 80vh;
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
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.folder-browser {
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* Location selector */
	.location-selector {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.location-selector label {
		font-size: 13px;
		font-weight: 500;
		color: #6b7280;
	}

	.location-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.location-chip {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		background: #f3f4f6;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 13px;
		color: #374151;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.location-chip:hover {
		background: #e5e7eb;
	}

	.location-chip.active {
		background: #eff6ff;
		border-color: #3b82f6;
		color: #3b82f6;
	}

	/* Breadcrumbs */
	.breadcrumbs {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 4px;
	}

	.breadcrumb {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		font-size: 13px;
		color: #6b7280;
		border-radius: 4px;
		transition: all 0.15s ease;
	}

	.breadcrumb:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.breadcrumb.active {
		background: #eff6ff;
		color: #3b82f6;
		font-weight: 500;
	}

	.breadcrumb-separator {
		color: #d1d5db;
	}

	/* Current path */
	.current-path {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-family: monospace;
		font-size: 13px;
		color: #374151;
		overflow: hidden;
	}

	.current-path span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.current-path .folder-icon {
		color: #f59e0b;
		flex-shrink: 0;
	}

	/* Folder list */
	.folder-list {
		flex: 1;
		overflow-y: auto;
		max-height: 250px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
	}

	.folder-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		font-size: 14px;
		color: #374151;
		text-align: left;
		border-bottom: 1px solid #f3f4f6;
		transition: background 0.15s ease;
	}

	.folder-item:last-child {
		border-bottom: none;
	}

	.folder-item:hover {
		background: #f9fafb;
	}

	.folder-item .folder-icon {
		color: #f59e0b;
	}

	.folder-item .folder-icon-up {
		color: #6b7280;
	}

	/* States */
	.loading-state,
	.error-state,
	.empty-state,
	.empty-folder-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 40px 20px;
		color: #9ca3af;
		gap: 8px;
	}

	.empty-folder-state {
		padding: 20px;
	}

	.error-state {
		color: #ef4444;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Footer */
	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-top: 1px solid #e5e7eb;
		gap: 16px;
	}

	.selected-path {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: 1;
		min-width: 0;
	}

	.selected-label {
		font-size: 13px;
		color: #6b7280;
		flex-shrink: 0;
	}

	.selected-path code {
		font-size: 13px;
		background: #f3f4f6;
		padding: 4px 8px;
		border-radius: 4px;
		color: #374151;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.footer-buttons {
		display: flex;
		gap: 12px;
		flex-shrink: 0;
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
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
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-secondary {
		padding: 8px 16px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: #f9fafb;
	}
</style>
