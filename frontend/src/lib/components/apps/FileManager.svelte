<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t, locale } from '$lib/i18n';
	import { onMount } from 'svelte';
	import { api, type FileItem as ApiFileItem, type BrowsableLocation } from '$lib/stores/api';
	import { fileTasks } from '$stores/taskManager';

	// Types for display (extends API type with icon)
	interface FileItem {
		id: string;
		name: string;
		type: 'folder' | 'file' | 'trash';
		mimeType?: string;
		size?: number;
		modified: Date;
		icon: string;
		path: string;
	}

	// State
	let currentPath = '';
	let files: FileItem[] = [];
	let loading = false;
	let error: string | null = null;
	let selectedFiles: string[] = [];
	let sortColumn: 'name' | 'size' | 'type' | 'modified' = 'name';
	let sortDirection: 'asc' | 'desc' = 'asc';
	let searchQuery = '';
	let showActionMenu: string | null = null;
	let viewMode: 'list' | 'grid' | 'compact' = 'list';
	let showViewDropdown = false;
	let actionMenuPos = { x: 0, y: 0 };

	// Modal states
	let showCreateFolderModal = false;
	let showCreateFileModal = false;
	let showDeleteModal = false;
	let showRenameModal = false;
	let modalInputValue = '';
	let deleteTarget: FileItem | null = null;
	let renameTarget: FileItem | null = null;

	// Clipboard state
	let clipboard: { files: FileItem[]; mode: 'copy' | 'cut' } | null = null;

	// Upload state
	let fileInput: HTMLInputElement;

	function toggleActionMenu(fileId: string, e: MouseEvent) {
		if (showActionMenu === fileId) {
			showActionMenu = null;
		} else {
			const btn = e.currentTarget as HTMLElement;
			const rect = btn.getBoundingClientRect();
			actionMenuPos = { x: rect.right - 160, y: rect.bottom + 4 };
			showActionMenu = fileId;
		}
	}

	// Locations state
	let locations: BrowsableLocation[] = [];
	let selectedLocationId: string | null = null;
	let locationsLoading = false;
	let locationsError: string | null = null;

	// Computed: Group locations by type
	$: homeLocations = locations.filter(l => l.type === 'home');
	$: shareLocations = locations.filter(l => l.type === 'share');
	$: volumeLocations = locations.filter(l => l.type === 'volume');
	$: mediaLocations = locations.filter(l => l.type === 'media');
	$: actionMenuFile = showActionMenu ? sortedFiles.find(f => f.id === showActionMenu) || null : null;
	$: hasClipboard = clipboard !== null && clipboard.files.length > 0;
	$: hasSelection = selectedFiles.length > 0;

	// Sidebar section expansion state
	let expandedSections = {
		personal: true,
		shares: true,
		volumes: false,
		media: true
	};

	// Get icon for file type
	function getFileIcon(item: ApiFileItem): string {
		if (item.type === 'folder') {
			return 'mdi:folder';
		}
		const mime = item.mime_type || '';
		if (mime.startsWith('image/')) return 'mdi:file-image';
		if (mime.startsWith('video/')) return 'mdi:file-video';
		if (mime.startsWith('audio/')) return 'mdi:file-music';
		if (mime.includes('pdf')) return 'mdi:file-pdf-box';
		if (mime.includes('word') || mime.includes('document')) return 'mdi:file-word';
		if (mime.includes('excel') || mime.includes('spreadsheet')) return 'mdi:file-excel';
		if (mime.includes('zip') || mime.includes('archive') || mime.includes('compressed')) return 'mdi:folder-zip';
		if (mime.startsWith('text/')) return 'mdi:file-document-outline';
		return 'mdi:file-outline';
	}

	// Convert API response to display format
	function toDisplayItem(item: ApiFileItem): FileItem {
		return {
			id: item.path,
			name: item.name,
			type: item.type as 'folder' | 'file',
			mimeType: item.mime_type,
			size: item.size ?? undefined,
			modified: new Date(item.modified),
			icon: getFileIcon(item),
			path: item.path
		};
	}

	// Load locations from API
	async function loadLocations() {
		locationsLoading = true;
		locationsError = null;
		try {
			locations = await api.getLocations();
			if (!selectedLocationId && homeLocations.length > 0) {
				selectLocation(homeLocations[0]);
			} else if (!selectedLocationId && locations.length > 0) {
				selectLocation(locations[0]);
			}
		} catch (e) {
			locationsError = e instanceof Error ? e.message : 'Failed to load locations';
			locations = [];
		} finally {
			locationsLoading = false;
		}
	}

	// Select a location and load its files
	function selectLocation(location: BrowsableLocation) {
		selectedLocationId = location.id;
		currentPath = '';
		loadFiles('');
	}

	// Load files from API
	async function loadFiles(path: string = '') {
		loading = true;
		error = null;
		try {
			const apiFiles = await api.getFiles(path, selectedLocationId || undefined);
			files = apiFiles.map(toDisplayItem);
			currentPath = path;
			selectedFiles = [];
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load files';
			files = [];
		} finally {
			loading = false;
		}
	}

	// Load locations on mount
	onMount(() => {
		loadLocations();
	});

	// Navigation history
	let historyIndex = 0;
	let history: string[] = [''];

	// Computed
	$: filteredFiles = files.filter(f =>
		f.name.toLowerCase().includes(searchQuery.toLowerCase())
	);

	$: sortedFiles = [...filteredFiles].sort((a, b) => {
		let comparison = 0;
		switch (sortColumn) {
			case 'name':
				comparison = a.name.localeCompare(b.name);
				break;
			case 'size':
				comparison = (a.size || 0) - (b.size || 0);
				break;
			case 'type':
				comparison = (a.mimeType || a.type).localeCompare(b.mimeType || b.type);
				break;
			case 'modified':
				comparison = a.modified.getTime() - b.modified.getTime();
				break;
		}
		return sortDirection === 'asc' ? comparison : -comparison;
	});

	$: totalItems = sortedFiles.length;
	$: selectedCount = selectedFiles.length;
	$: canGoBack = historyIndex > 0;
	$: canGoForward = historyIndex < history.length - 1;

	// Functions
	function selectFolder(path: string) {
		if (history[historyIndex] !== path) {
			history = [...history.slice(0, historyIndex + 1), path];
			historyIndex = history.length - 1;
		}
		loadFiles(path);
	}

	function goBack() {
		if (canGoBack) {
			historyIndex--;
			loadFiles(history[historyIndex]);
		}
	}

	function goForward() {
		if (canGoForward) {
			historyIndex++;
			loadFiles(history[historyIndex]);
		}
	}

	function refresh() {
		loadFiles(currentPath);
	}

	// --- Modal-based operations ---

	// Create folder
	function openCreateFolderModal() {
		modalInputValue = '';
		showCreateFolderModal = true;
	}

	async function confirmCreateFolder() {
		if (modalInputValue.trim()) {
			try {
				const taskId = fileTasks.addTask('create_folder', modalInputValue.trim());
				await api.createFolder(currentPath, modalInputValue.trim(), selectedLocationId || undefined);
				fileTasks.updateTask(taskId, { status: 'completed', progress: 100 });
				await loadFiles(currentPath);
			} catch (e) {
				const msg = e instanceof Error ? e.message : 'Error';
				fileTasks.updateTask(fileTasks.addTask('create_folder', modalInputValue.trim()), { status: 'error', error: msg });
			}
		}
		showCreateFolderModal = false;
		modalInputValue = '';
	}

	// Create file
	function openCreateFileModal() {
		modalInputValue = '';
		showCreateFileModal = true;
	}

	async function confirmCreateFile() {
		if (modalInputValue.trim()) {
			try {
				const taskId = fileTasks.addTask('create_file', modalInputValue.trim());
				await api.createFile(currentPath, modalInputValue.trim(), selectedLocationId || undefined);
				fileTasks.updateTask(taskId, { status: 'completed', progress: 100 });
				await loadFiles(currentPath);
			} catch (e) {
				const msg = e instanceof Error ? e.message : 'Error';
				fileTasks.updateTask(fileTasks.addTask('create_file', modalInputValue.trim()), { status: 'error', error: msg });
			}
		}
		showCreateFileModal = false;
		modalInputValue = '';
	}

	// Delete
	function openDeleteModal(file: FileItem) {
		deleteTarget = file;
		showDeleteModal = true;
		showActionMenu = null;
	}

	async function confirmDelete() {
		if (deleteTarget) {
			try {
				const taskId = fileTasks.addTask('delete', deleteTarget.name);
				await api.deleteFile(deleteTarget.path, selectedLocationId || undefined);
				fileTasks.updateTask(taskId, { status: 'completed', progress: 100 });
				await loadFiles(currentPath);
			} catch (e) {
				const msg = e instanceof Error ? e.message : 'Error';
				fileTasks.updateTask(fileTasks.addTask('delete', deleteTarget?.name || ''), { status: 'error', error: msg });
			}
		}
		showDeleteModal = false;
		deleteTarget = null;
	}

	// Rename
	function openRenameModal(file: FileItem) {
		renameTarget = file;
		modalInputValue = file.name;
		showRenameModal = true;
		showActionMenu = null;
	}

	async function confirmRename() {
		if (renameTarget && modalInputValue.trim() && modalInputValue !== renameTarget.name) {
			try {
				await api.renameFile(renameTarget.path, modalInputValue.trim(), selectedLocationId || undefined);
				await loadFiles(currentPath);
			} catch (e) {
				// silently fail - could add notification
			}
		}
		showRenameModal = false;
		renameTarget = null;
		modalInputValue = '';
	}

	// Upload
	function triggerUpload() {
		fileInput?.click();
	}

	async function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const uploadFiles = input.files;
		if (!uploadFiles || uploadFiles.length === 0) return;

		for (const file of Array.from(uploadFiles)) {
			const taskId = fileTasks.addTask('upload', file.name);
			try {
				await api.uploadFile(file, currentPath, selectedLocationId || undefined);
				fileTasks.updateTask(taskId, { status: 'completed', progress: 100 });
			} catch (e) {
				const msg = e instanceof Error ? e.message : 'Upload failed';
				fileTasks.updateTask(taskId, { status: 'error', error: msg });
			}
		}
		// Reset input
		input.value = '';
		await loadFiles(currentPath);
	}

	// Download
	async function handleDownload(file: FileItem) {
		showActionMenu = null;
		try {
			await api.downloadFile(file.path, selectedLocationId || undefined);
		} catch (e) {
			// silently fail
		}
	}

	// Clipboard operations
	function handleCopy() {
		const selected = sortedFiles.filter(f => selectedFiles.includes(f.id));
		if (selected.length > 0) {
			clipboard = { files: selected, mode: 'copy' };
		}
	}

	function handleCut() {
		const selected = sortedFiles.filter(f => selectedFiles.includes(f.id));
		if (selected.length > 0) {
			clipboard = { files: selected, mode: 'cut' };
		}
	}

	async function handlePaste() {
		if (!clipboard) return;

		const sources = clipboard.files.map(f => f.path);
		const destination = currentPath || '.';

		try {
			if (clipboard.mode === 'copy') {
				await api.copyFiles(sources, destination, selectedLocationId || undefined);
			} else {
				await api.moveFiles(sources, destination, selectedLocationId || undefined);
			}
			// WebSocket will create and update task status
			if (clipboard.mode === 'cut') {
				clipboard = null;
			}
			// Refresh after a short delay to let background ops complete
			setTimeout(() => loadFiles(currentPath), 500);
		} catch (e) {
			// Error handling via WebSocket task updates
		}
	}

	async function handleDuplicate() {
		const selected = sortedFiles.filter(f => selectedFiles.includes(f.id));
		if (selected.length === 0) return;

		const sources = selected.map(f => f.path);
		const destination = currentPath || '.';

		try {
			await api.copyFiles(sources, destination, selectedLocationId || undefined);
			setTimeout(() => loadFiles(currentPath), 500);
		} catch (e) {
			// Error handling via WebSocket
		}
	}

	// Delete selected files
	function handleDeleteSelected() {
		if (selectedFiles.length === 1) {
			const file = sortedFiles.find(f => f.id === selectedFiles[0]);
			if (file) openDeleteModal(file);
		}
	}

	function toggleSort(column: typeof sortColumn) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function toggleSelect(fileId: string, event?: MouseEvent) {
		if (event?.ctrlKey || event?.metaKey) {
			if (selectedFiles.includes(fileId)) {
				selectedFiles = selectedFiles.filter(id => id !== fileId);
			} else {
				selectedFiles = [...selectedFiles, fileId];
			}
		} else {
			selectedFiles = selectedFiles.includes(fileId) && selectedFiles.length === 1
				? []
				: [fileId];
		}
	}

	function handleDoubleClick(file: FileItem) {
		if (file.type === 'folder') {
			selectFolder(file.path);
		} else {
			console.log('Open file:', file.name);
		}
	}

	function formatSize(bytes?: number): string {
		if (!bytes) return '-';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}

	function formatDate(date: Date): string {
		return date.toLocaleDateString($locale === 'fr' ? 'fr-FR' : 'en-US', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit'
		}) + ' ' + date.toLocaleTimeString($locale === 'fr' ? 'fr-FR' : 'en-US', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function getFileType(file: FileItem): string {
		if (file.type === 'folder') return $t.fileManager.types.folder;
		if (file.type === 'trash') return $t.fileManager.types.trash;
		return file.mimeType || $t.fileManager.types.file;
	}

	$: currentFolderDisplay = (() => {
		const location = locations.find(l => l.id === selectedLocationId);
		const locationName = location?.name || '';
		if (!currentPath) return locationName ? `/${locationName}` : '/';
		return `/${locationName}/${currentPath}`;
	})();

	function goToParent() {
		if (currentPath) {
			const parts = currentPath.split('/');
			parts.pop();
			selectFolder(parts.join('/'));
		}
	}

	function closeMenus() {
		showActionMenu = null;
		showViewDropdown = false;
	}

	function closeModal() {
		showCreateFolderModal = false;
		showCreateFileModal = false;
		showDeleteModal = false;
		showRenameModal = false;
		modalInputValue = '';
		deleteTarget = null;
		renameTarget = null;
	}

	function handleModalKeydown(e: KeyboardEvent, confirmFn: () => void) {
		if (e.key === 'Enter') confirmFn();
		if (e.key === 'Escape') closeModal();
	}
</script>

<svelte:window on:click={closeMenus} />

<!-- Hidden file input for uploads -->
<input
	type="file"
	bind:this={fileInput}
	on:change={handleFileUpload}
	multiple
	style="display: none;"
/>

<!-- Modals -->
{#if showCreateFolderModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={closeModal}>
		<div class="modal-dialog" on:click|stopPropagation>
			<h3>{$t.fileManager.modals?.newFolderTitle || 'New Folder'}</h3>
			<input
				type="text"
				bind:value={modalInputValue}
				placeholder={$t.fileManager.modals?.folderNamePlaceholder || 'Folder name'}
				on:keydown={(e) => handleModalKeydown(e, confirmCreateFolder)}
				autofocus
			/>
			<div class="modal-actions">
				<button class="btn-cancel" on:click={closeModal}>{$t.common.cancel}</button>
				<button class="btn-confirm" on:click={confirmCreateFolder} disabled={!modalInputValue.trim()}>{$t.common.create}</button>
			</div>
		</div>
	</div>
{/if}

{#if showCreateFileModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={closeModal}>
		<div class="modal-dialog" on:click|stopPropagation>
			<h3>{$t.fileManager.modals?.newFileTitle || 'New File'}</h3>
			<input
				type="text"
				bind:value={modalInputValue}
				placeholder={$t.fileManager.modals?.fileNamePlaceholder || 'File name'}
				on:keydown={(e) => handleModalKeydown(e, confirmCreateFile)}
				autofocus
			/>
			<div class="modal-actions">
				<button class="btn-cancel" on:click={closeModal}>{$t.common.cancel}</button>
				<button class="btn-confirm" on:click={confirmCreateFile} disabled={!modalInputValue.trim()}>{$t.common.create}</button>
			</div>
		</div>
	</div>
{/if}

{#if showRenameModal && renameTarget}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={closeModal}>
		<div class="modal-dialog" on:click|stopPropagation>
			<h3>{$t.fileManager.contextMenu.rename}</h3>
			<input
				type="text"
				bind:value={modalInputValue}
				on:keydown={(e) => handleModalKeydown(e, confirmRename)}
				autofocus
			/>
			<div class="modal-actions">
				<button class="btn-cancel" on:click={closeModal}>{$t.common.cancel}</button>
				<button class="btn-confirm" on:click={confirmRename} disabled={!modalInputValue.trim() || modalInputValue === renameTarget.name}>{$t.common.confirm}</button>
			</div>
		</div>
	</div>
{/if}

{#if showDeleteModal && deleteTarget}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" on:click={closeModal}>
		<div class="modal-dialog" on:click|stopPropagation>
			<div class="delete-modal-icon">
				<Icon icon="mdi:alert-circle" class="w-10 h-10" />
			</div>
			<h3>{$t.fileManager.modals?.deleteTitle || 'Delete'}</h3>
			<p class="modal-message">
				{#if deleteTarget.type === 'folder'}
					{$t.fileManager.modals?.deleteFolderMessage || 'Delete folder "{name}" and all its contents?'}
				{:else}
					{$t.fileManager.modals?.deleteFileMessage || 'Delete "{name}"?'}
				{/if}
			</p>
			<p class="modal-filename">"{deleteTarget.name}"</p>
			<div class="modal-actions">
				<button class="btn-cancel" on:click={closeModal}>{$t.common.cancel}</button>
				<button class="btn-danger" on:click={confirmDelete}>{$t.common.delete}</button>
			</div>
		</div>
	</div>
{/if}

<div class="file-manager">
	<!-- Sidebar -->
	<aside class="sidebar">
		<nav class="sidebar-nav">
			{#if locationsLoading}
				<div class="sidebar-loading">
					<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
				</div>
			{:else if locationsError}
				<div class="sidebar-error">
					<Icon icon="mdi:alert-circle" class="w-5 h-5" />
					<span>{locationsError}</span>
				</div>
			{:else}
				<!-- Personal section -->
				{#if homeLocations.length > 0}
					<div class="sidebar-section">
						<button
							class="section-header"
							on:click={() => expandedSections.personal = !expandedSections.personal}
						>
							<Icon
								icon={expandedSections.personal ? 'mdi:chevron-down' : 'mdi:chevron-right'}
								class="w-4 h-4"
							/>
							<span>{$t.fileManager.sections?.personal || 'Personal'}</span>
						</button>
						{#if expandedSections.personal}
							{#each homeLocations as loc}
								<button
									class="sidebar-item"
									class:active={selectedLocationId === loc.id}
									on:click={() => selectLocation(loc)}
								>
									<Icon icon={loc.icon} class="w-4 h-4 location-icon home-icon" />
									<span>{loc.name}</span>
								</button>
							{/each}
						{/if}
					</div>
				{/if}

				<!-- Shares section -->
				{#if shareLocations.length > 0}
					<div class="sidebar-section">
						<button
							class="section-header"
							on:click={() => expandedSections.shares = !expandedSections.shares}
						>
							<Icon
								icon={expandedSections.shares ? 'mdi:chevron-down' : 'mdi:chevron-right'}
								class="w-4 h-4"
							/>
							<span>{$t.fileManager.sections?.shares || 'Shared Folders'}</span>
						</button>
						{#if expandedSections.shares}
							{#each shareLocations as loc}
								<button
									class="sidebar-item"
									class:active={selectedLocationId === loc.id}
									class:disabled={!loc.enabled}
									on:click={() => loc.enabled && selectLocation(loc)}
									disabled={!loc.enabled}
								>
									<Icon icon={loc.icon} class="w-4 h-4 location-icon share-icon" />
									<span>{loc.name}</span>
									{#if !loc.enabled}
										<span class="status-badge disabled">{$t.fileManager.statuses?.disabled || 'Disabled'}</span>
									{/if}
								</button>
							{/each}
						{/if}
					</div>
				{/if}

				<!-- Volumes section -->
				{#if volumeLocations.length > 0}
					<div class="sidebar-section">
						<button
							class="section-header"
							on:click={() => expandedSections.volumes = !expandedSections.volumes}
						>
							<Icon
								icon={expandedSections.volumes ? 'mdi:chevron-down' : 'mdi:chevron-right'}
								class="w-4 h-4"
							/>
							<span>{$t.fileManager.sections?.volumes || 'Volumes'}</span>
						</button>
						{#if expandedSections.volumes}
							{#each volumeLocations as loc}
								<button
									class="sidebar-item"
									class:active={selectedLocationId === loc.id}
									class:disabled={loc.status !== 'mounted'}
									on:click={() => loc.status === 'mounted' && selectLocation(loc)}
									disabled={loc.status !== 'mounted'}
								>
									<Icon icon={loc.icon} class="w-4 h-4 location-icon volume-icon" />
									<div class="location-info">
										<span class="location-name">{loc.name}</span>
										{#if loc.pool_name}
											<span class="location-detail">{loc.pool_name}</span>
										{/if}
									</div>
									{#if loc.status === 'mounted' && loc.usage_percent !== undefined}
										<div class="usage-bar">
											<div class="usage-fill" style="width: {loc.usage_percent}%"></div>
										</div>
									{:else if loc.status !== 'mounted'}
										<span class="status-badge unmounted">{$t.fileManager.statuses?.unmounted || 'Unmounted'}</span>
									{/if}
								</button>
							{/each}
						{/if}
					</div>
				{/if}

				<!-- Removable Media section -->
				{#if mediaLocations.length > 0}
					<div class="sidebar-section">
						<button
							class="section-header"
							on:click={() => expandedSections.media = !expandedSections.media}
						>
							<Icon
								icon={expandedSections.media ? 'mdi:chevron-down' : 'mdi:chevron-right'}
								class="w-4 h-4"
							/>
							<span>{$t.fileManager.sections?.media || 'Removable Media'}</span>
						</button>
						{#if expandedSections.media}
							{#each mediaLocations as loc}
								<button
									class="sidebar-item"
									class:active={selectedLocationId === loc.id}
									on:click={() => selectLocation(loc)}
								>
									<Icon icon={loc.icon} class="w-4 h-4 location-icon media-icon" />
									<span>{loc.name}</span>
								</button>
							{/each}
						{/if}
					</div>
				{/if}

				<!-- Empty state -->
				{#if locations.length === 0}
					<div class="sidebar-empty">
						<Icon icon="mdi:folder-off-outline" class="w-8 h-8" />
						<span>{$t.fileManager.noLocations || 'No locations available'}</span>
					</div>
				{/if}
			{/if}
		</nav>
	</aside>

	<!-- Main content -->
	<div class="main-content">
		<!-- Navigation toolbar -->
		<div class="nav-toolbar">
			<div class="nav-buttons">
				<button
					class="nav-btn"
					disabled={!canGoBack}
					on:click={goBack}
					title={$t.common.back}
				>
					<Icon icon="mdi:chevron-left" class="w-5 h-5" />
				</button>
				<button
					class="nav-btn"
					disabled={!canGoForward}
					on:click={goForward}
					title={$t.common.next}
				>
					<Icon icon="mdi:chevron-right" class="w-5 h-5" />
				</button>
			</div>
			<button class="nav-btn" on:click={refresh} title={$t.fileManager.toolbar.refresh}>
				<Icon icon="mdi:refresh" class="w-5 h-5" />
			</button>
			<div class="path-input">
				<span>{currentFolderDisplay}</span>
			</div>
			<div class="search-input">
				<Icon icon="mdi:magnify" class="w-4 h-4 search-icon" />
				<input
					type="text"
					placeholder={$t.fileManager.toolbar.search}
					bind:value={searchQuery}
				/>
			</div>
		</div>

		<!-- Action toolbar -->
		<div class="action-toolbar">
			<div class="action-left">
				<button class="action-btn" title={$t.fileManager.toolbar.newFolder} on:click={openCreateFolderModal}>
					<Icon icon="mdi:folder-plus" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.modals?.newFileTitle || 'New file'} on:click={openCreateFileModal}>
					<Icon icon="mdi:file-plus" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.upload} on:click={triggerUpload}>
					<Icon icon="mdi:upload" class="w-5 h-5" />
				</button>
				<div class="toolbar-divider"></div>
				<button class="action-btn" title={$t.fileManager.toolbar.copy} disabled={!hasSelection} on:click={handleCopy}>
					<Icon icon="mdi:content-copy" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.cut} disabled={!hasSelection} on:click={handleCut}>
					<Icon icon="mdi:content-cut" class="w-5 h-5" />
				</button>
				<button class="action-btn" class:clipboard-active={hasClipboard} title={$t.fileManager.toolbar.paste} disabled={!hasClipboard} on:click={handlePaste}>
					<Icon icon="mdi:content-paste" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.duplicate} disabled={!hasSelection} on:click={handleDuplicate}>
					<Icon icon="mdi:file-multiple-outline" class="w-5 h-5" />
				</button>
			</div>
			<div class="action-right">
				<button class="action-btn" title={$t.fileManager.toolbar.sort}>
					<Icon icon="mdi:sort-variant" class="w-5 h-5" />
				</button>
				<div class="view-dropdown-container">
					<button
						class="action-btn view-btn"
						on:click|stopPropagation={() => showViewDropdown = !showViewDropdown}
					>
						<Icon icon="mdi:view-list" class="w-5 h-5" />
						<Icon icon="mdi:chevron-down" class="w-4 h-4" />
					</button>
					{#if showViewDropdown}
						<div class="view-dropdown">
							<button
								class="view-option"
								class:active={viewMode === 'list'}
								on:click={() => { viewMode = 'list'; showViewDropdown = false; }}
							>
								<Icon icon="mdi:view-list" class="w-4 h-4" />
								{$t.fileManager.viewModes.list}
							</button>
							<button
								class="view-option"
								class:active={viewMode === 'grid'}
								on:click={() => { viewMode = 'grid'; showViewDropdown = false; }}
							>
								<Icon icon="mdi:view-grid" class="w-4 h-4" />
								{$t.fileManager.viewModes.grid}
							</button>
							<button
								class="view-option"
								class:active={viewMode === 'compact'}
								on:click={() => { viewMode = 'compact'; showViewDropdown = false; }}
							>
								<Icon icon="mdi:view-headline" class="w-4 h-4" />
								{$t.fileManager.viewModes.compact}
							</button>
						</div>
					{/if}
				</div>
			</div>
		</div>

		<!-- Clipboard indicator -->
		{#if hasClipboard}
			<div class="clipboard-bar">
				<Icon icon={clipboard?.mode === 'cut' ? 'mdi:content-cut' : 'mdi:content-copy'} class="w-4 h-4" />
				<span>
					{clipboard?.files.length} {$t.fileManager.statusBar?.selected || 'selected'} — {clipboard?.mode === 'cut' ? ($t.fileManager.toolbar.cut || 'Cut') : ($t.fileManager.toolbar.copy || 'Copy')}
				</span>
				<button class="clipboard-clear" on:click={() => clipboard = null}>
					<Icon icon="mdi:close" class="w-3 h-3" />
				</button>
			</div>
		{/if}

		<!-- File list -->
		<div class="file-list-container">
			{#if loading}
				<div class="loading-state">
					<Icon icon="mdi:loading" class="w-8 h-8 animate-spin" />
					<span>{$t.common.loading}</span>
				</div>
			{:else if error}
				<div class="error-state">
					<Icon icon="mdi:alert-circle" class="w-8 h-8" />
					<span>{error}</span>
					<button class="retry-btn" on:click={refresh}>{$t.common.retry}</button>
				</div>
			{:else if viewMode === 'list'}
				<table class="file-table">
					<thead>
						<tr>
							<th
								class="sortable"
								on:click={() => toggleSort('name')}
							>
								{$t.fileManager.columns.name}
								{#if sortColumn === 'name'}
									<Icon
										icon={sortDirection === 'asc' ? 'mdi:arrow-up' : 'mdi:arrow-down'}
										class="w-3 h-3 sort-icon"
									/>
								{/if}
							</th>
							<th
								class="sortable col-size"
								on:click={() => toggleSort('size')}
							>
								{$t.fileManager.columns.size}
								{#if sortColumn === 'size'}
									<Icon
										icon={sortDirection === 'asc' ? 'mdi:arrow-up' : 'mdi:arrow-down'}
										class="w-3 h-3 sort-icon"
									/>
								{/if}
							</th>
							<th
								class="sortable col-type"
								on:click={() => toggleSort('type')}
							>
								{$t.fileManager.columns.type}
								{#if sortColumn === 'type'}
									<Icon
										icon={sortDirection === 'asc' ? 'mdi:arrow-up' : 'mdi:arrow-down'}
										class="w-3 h-3 sort-icon"
									/>
								{/if}
							</th>
							<th
								class="sortable col-modified"
								on:click={() => toggleSort('modified')}
							>
								{$t.fileManager.columns.modified}
								{#if sortColumn === 'modified'}
									<Icon
										icon={sortDirection === 'asc' ? 'mdi:arrow-up' : 'mdi:arrow-down'}
										class="w-3 h-3 sort-icon"
									/>
								{/if}
							</th>
							<th class="col-actions"></th>
						</tr>
					</thead>
					<tbody>
						{#each sortedFiles as file}
							<tr
								class="file-row"
								class:selected={selectedFiles.includes(file.id)}
								class:cut={clipboard?.mode === 'cut' && clipboard.files.some(f => f.id === file.id)}
								on:click={(e) => toggleSelect(file.id, e)}
								on:dblclick={() => handleDoubleClick(file)}
							>
								<td class="cell-name">
									<Icon
										icon={file.icon}
										class="w-5 h-5 file-icon {file.type === 'folder' ? 'folder-icon' : file.type === 'trash' ? 'trash-icon' : 'file-icon-default'}"
									/>
									<span>{file.name}</span>
								</td>
								<td class="cell-size">{formatSize(file.size)}</td>
								<td class="cell-type">{getFileType(file)}</td>
								<td class="cell-modified">{formatDate(file.modified)}</td>
								<td class="cell-actions">
									<button
										class="action-menu-btn"
										on:click|stopPropagation={(e) => toggleActionMenu(file.id, e)}
									>
										<Icon icon="mdi:dots-vertical" class="w-5 h-5" />
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{:else if viewMode === 'grid'}
				<div class="file-grid">
					{#each sortedFiles as file}
						<button
							class="file-card"
							class:selected={selectedFiles.includes(file.id)}
							class:cut={clipboard?.mode === 'cut' && clipboard.files.some(f => f.id === file.id)}
							on:click={(e) => toggleSelect(file.id, e)}
							on:dblclick={() => handleDoubleClick(file)}
						>
							<div class="file-card-icon">
								<Icon
									icon={file.icon}
									class="w-12 h-12 {file.type === 'folder' ? 'folder-icon' : file.type === 'trash' ? 'trash-icon' : 'file-icon-default'}"
								/>
							</div>
							<span class="file-card-name">{file.name}</span>
						</button>
					{/each}
				</div>
			{:else if viewMode === 'compact'}
				<div class="file-compact">
					{#each sortedFiles as file}
						<button
							class="file-compact-item"
							class:selected={selectedFiles.includes(file.id)}
							class:cut={clipboard?.mode === 'cut' && clipboard.files.some(f => f.id === file.id)}
							on:click={(e) => toggleSelect(file.id, e)}
							on:dblclick={() => handleDoubleClick(file)}
						>
							<Icon
								icon={file.icon}
								class="w-4 h-4 {file.type === 'folder' ? 'folder-icon' : file.type === 'trash' ? 'trash-icon' : 'file-icon-default'}"
							/>
							<span>{file.name}</span>
						</button>
					{/each}
				</div>
			{/if}

			{#if !loading && !error && sortedFiles.length === 0}
				<div class="empty-state">
					<Icon icon="mdi:folder-open-outline" class="w-16 h-16 text-gray-300" />
					<span>{$t.fileManager.modals?.emptyFolder || 'This folder is empty'}</span>
				</div>
			{/if}
		</div>

		<!-- Status bar -->
		<footer class="status-bar">
			<span>
				{totalItems} {totalItems === 1 ? $t.fileManager.statusBar.item : $t.fileManager.statusBar.items}
			</span>
			{#if selectedCount > 0}
				<span class="selected-info">
					{selectedCount} {$t.fileManager.statusBar.selected}
				</span>
			{/if}
		</footer>
	</div>
</div>

<!-- Floating action menu (rendered outside scroll containers to avoid clipping) -->
{#if actionMenuFile}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="action-menu-backdrop" on:click={() => showActionMenu = null}></div>
	<div class="action-menu" style="left: {actionMenuPos.x}px; top: {actionMenuPos.y}px;">
		{#if actionMenuFile.type === 'folder'}
			<button class="menu-item" on:click={() => { handleDoubleClick(actionMenuFile); showActionMenu = null; }}>
				<Icon icon="mdi:folder-open" class="w-4 h-4" />
				{$t.common.open}
			</button>
		{/if}
		{#if actionMenuFile.type === 'file'}
			<button class="menu-item" on:click={() => handleDownload(actionMenuFile)}>
				<Icon icon="mdi:download" class="w-4 h-4" />
				{$t.fileManager.contextMenu.download}
			</button>
		{/if}
		<button class="menu-item" on:click={() => openRenameModal(actionMenuFile)}>
			<Icon icon="mdi:pencil" class="w-4 h-4" />
			{$t.fileManager.contextMenu.rename}
		</button>
		<button class="menu-item" on:click={() => { clipboard = { files: [actionMenuFile], mode: 'copy' }; showActionMenu = null; }}>
			<Icon icon="mdi:content-copy" class="w-4 h-4" />
			{$t.fileManager.toolbar.copy}
		</button>
		<button class="menu-item" on:click={() => { clipboard = { files: [actionMenuFile], mode: 'cut' }; showActionMenu = null; }}>
			<Icon icon="mdi:content-cut" class="w-4 h-4" />
			{$t.fileManager.toolbar.cut}
		</button>
		<div class="menu-divider"></div>
		<button class="menu-item danger" on:click={() => openDeleteModal(actionMenuFile)}>
			<Icon icon="mdi:delete" class="w-4 h-4" />
			{$t.common.delete}
		</button>
	</div>
{/if}

<style>
	.file-manager {
		display: flex;
		height: 100%;
		background: white;
	}

	/* Sidebar */
	.sidebar {
		width: 220px;
		background: #f8f9fc;
		border-right: 1px solid #e5e7eb;
		overflow-y: auto;
	}

	.sidebar-nav {
		padding: 8px 0;
	}

	.sidebar-section {
		margin-bottom: 4px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 8px 12px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: #6b7280;
		text-align: left;
		transition: all 0.15s ease;
	}

	.section-header:hover {
		color: #374151;
		background: rgba(0, 0, 0, 0.03);
	}

	.sidebar-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 8px 12px 8px 28px;
		font-size: 13px;
		color: #374151;
		text-align: left;
		transition: all 0.15s ease;
		border-left: 3px solid transparent;
	}

	.sidebar-item:hover:not(:disabled) {
		background: #eef2ff;
	}

	.sidebar-item.active {
		background: #eef2ff;
		border-left-color: #3b82f6;
		color: #1f2937;
		font-weight: 500;
	}

	.sidebar-item.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.location-icon {
		flex-shrink: 0;
	}

	.home-icon { color: #3b82f6; }
	.share-icon { color: #10b981; }
	.volume-icon { color: #8b5cf6; }
	.media-icon { color: #f59e0b; }

	.location-info {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}

	.location-name {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.location-detail {
		font-size: 11px;
		color: #9ca3af;
	}

	.status-badge {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 4px;
		font-weight: 500;
	}

	.status-badge.disabled { background: #fef3c7; color: #92400e; }
	.status-badge.unmounted { background: #f3f4f6; color: #6b7280; }

	.usage-bar {
		width: 40px;
		height: 4px;
		background: #e5e7eb;
		border-radius: 2px;
		overflow: hidden;
		flex-shrink: 0;
	}

	.usage-fill {
		height: 100%;
		background: #3b82f6;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.sidebar-loading,
	.sidebar-error,
	.sidebar-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 24px 16px;
		color: #6b7280;
		font-size: 13px;
		text-align: center;
	}

	.sidebar-error { color: #ef4444; }

	/* Main content */
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* Navigation toolbar */
	.nav-toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.nav-buttons {
		display: flex;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		overflow: hidden;
	}

	.nav-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
		transition: all 0.15s ease;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		background: white;
	}

	.nav-buttons .nav-btn {
		border: none;
		border-radius: 0;
	}

	.nav-buttons .nav-btn:first-child {
		border-right: 1px solid #e5e7eb;
	}

	.nav-btn:hover:not(:disabled) {
		background: #f3f4f6;
		color: #374151;
	}

	.nav-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.path-input {
		flex: 1;
		padding: 8px 16px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
	}

	.search-input {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		color: #9ca3af;
	}

	.search-input input {
		padding: 8px 12px 8px 36px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		width: 200px;
		outline: none;
		transition: all 0.15s ease;
	}

	.search-input input:focus {
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.search-input input::placeholder { color: #9ca3af; }

	/* Action toolbar */
	.action-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.action-left,
	.action-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.toolbar-divider {
		width: 1px;
		height: 24px;
		background: #e5e7eb;
		margin: 0 4px;
	}

	.action-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		background: white;
		transition: all 0.15s ease;
	}

	.action-btn:hover:not(:disabled) {
		background: #f3f4f6;
		color: #374151;
	}

	.action-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.action-btn.clipboard-active {
		border-color: #3b82f6;
		color: #3b82f6;
		background: #eff6ff;
	}

	.view-btn {
		width: auto;
		padding: 0 8px;
		gap: 4px;
	}

	.view-dropdown-container {
		position: relative;
	}

	.view-dropdown {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 4px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		min-width: 140px;
		z-index: 50;
		overflow: hidden;
	}

	.view-option {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 10px 14px;
		font-size: 14px;
		color: #374151;
		text-align: left;
	}

	.view-option:hover { background: #f3f4f6; }
	.view-option.active { background: #eef2ff; color: #3b82f6; }

	/* Clipboard indicator bar */
	.clipboard-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 16px;
		background: #eff6ff;
		border-bottom: 1px solid #bfdbfe;
		font-size: 12px;
		color: #2563eb;
	}

	.clipboard-clear {
		margin-left: auto;
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		color: #2563eb;
		transition: background 0.15s;
	}

	.clipboard-clear:hover {
		background: rgba(37, 99, 235, 0.1);
	}

	/* File list */
	.file-list-container {
		flex: 1;
		overflow: auto;
	}

	.file-table {
		width: 100%;
		border-collapse: collapse;
	}

	.file-table th {
		position: sticky;
		top: 0;
		padding: 12px 16px;
		text-align: left;
		font-size: 13px;
		font-weight: 500;
		color: #6b7280;
		background: #f9fafb;
		border-bottom: 1px solid #e5e7eb;
		white-space: nowrap;
	}

	.file-table th.sortable {
		cursor: pointer;
		user-select: none;
	}

	.file-table th.sortable:hover { color: #374151; }

	.sort-icon {
		margin-left: 4px;
		color: #3b82f6;
	}

	.col-size { width: 100px; }
	.col-type { width: 120px; }
	.col-modified { width: 160px; }
	.col-actions { width: 48px; }

	.file-table td {
		padding: 12px 16px;
		font-size: 14px;
		color: #374151;
		border-bottom: 1px solid #f3f4f6;
	}

	.file-row {
		cursor: pointer;
		transition: background 0.1s ease;
	}

	.file-row:hover { background: #f9fafb; }
	.file-row.selected { background: #eef2ff; }
	.file-row.cut { opacity: 0.5; }

	.cell-name {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.file-icon { flex-shrink: 0; }
	.folder-icon { color: #f59e0b; }
	.trash-icon { color: #9ca3af; }
	.file-icon-default { color: #6b7280; }

	.cell-size,
	.cell-type,
	.cell-modified {
		color: #6b7280;
	}

	.cell-actions {
		position: relative;
	}

	.action-menu-btn {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #9ca3af;
		border-radius: 6px;
		opacity: 0;
		transition: all 0.15s ease;
	}

	.file-row:hover .action-menu-btn { opacity: 1; }

	.action-menu-btn:hover {
		background: #e5e7eb;
		color: #374151;
	}

	.action-menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: 9998;
	}

	.action-menu {
		position: fixed;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		min-width: 160px;
		z-index: 9999;
		overflow: hidden;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 14px;
		font-size: 14px;
		color: #374151;
		text-align: left;
	}

	.menu-item:hover { background: #f3f4f6; }
	.menu-item.danger { color: #ef4444; }
	.menu-item.danger:hover { background: #fef2f2; }

	.menu-divider {
		height: 1px;
		background: #e5e7eb;
		margin: 4px 0;
	}

	/* Grid view */
	.file-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
		gap: 12px;
		padding: 16px;
	}

	.file-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 16px 8px;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.file-card:hover { background: #f3f4f6; }
	.file-card.selected { background: #eef2ff; }
	.file-card.cut { opacity: 0.5; }

	.file-card-icon {
		width: 64px;
		height: 64px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.file-card-name {
		font-size: 12px;
		color: #374151;
		text-align: center;
		word-break: break-word;
		max-width: 100%;
	}

	/* Compact view */
	.file-compact {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		padding: 16px;
	}

	.file-compact-item {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 6px;
		font-size: 13px;
		color: #374151;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.file-compact-item:hover { background: #f3f4f6; }
	.file-compact-item.selected { background: #eef2ff; }
	.file-compact-item.cut { opacity: 0.5; }

	/* Status bar */
	.status-bar {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 10px 16px;
		border-top: 1px solid #e5e7eb;
		background: #fafafa;
		font-size: 13px;
		color: #6b7280;
	}

	.selected-info { color: #3b82f6; }

	/* Loading, error, empty states */
	.loading-state,
	.error-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		height: 100%;
		min-height: 200px;
		color: #6b7280;
	}

	.error-state { color: #ef4444; }

	.retry-btn {
		margin-top: 8px;
		padding: 8px 16px;
		background: #3b82f6;
		color: white;
		border-radius: 6px;
		font-size: 14px;
	}

	.retry-btn:hover { background: #2563eb; }

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Modal styles */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10000;
	}

	.modal-dialog {
		background: white;
		color: #1f2937;
		padding: 24px;
		border-radius: 12px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
		min-width: 340px;
		max-width: 420px;
		animation: modal-in 0.15s ease-out;
	}

	@keyframes modal-in {
		from { opacity: 0; transform: scale(0.95); }
		to { opacity: 1; transform: scale(1); }
	}

	.modal-dialog h3 {
		margin: 0 0 16px 0;
		font-size: 18px;
		font-weight: 600;
		color: #1f2937;
	}

	.modal-dialog input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		outline: none;
		box-sizing: border-box;
	}

	.modal-dialog input:focus {
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.modal-message {
		font-size: 14px;
		color: #4b5563;
		margin: 0 0 4px 0;
	}

	.modal-filename {
		font-size: 14px;
		font-weight: 600;
		color: #1f2937;
		margin: 0 0 16px 0;
	}

	.delete-modal-icon {
		display: flex;
		justify-content: center;
		margin-bottom: 12px;
		color: #ef4444;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 16px;
	}

	.btn-cancel,
	.btn-confirm,
	.btn-danger {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
	}

	.btn-cancel {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-cancel:hover { background: #e5e7eb; }

	.btn-confirm {
		background: #3b82f6;
		color: white;
	}

	.btn-confirm:hover:not(:disabled) { background: #2563eb; }
	.btn-confirm:disabled { opacity: 0.5; cursor: not-allowed; }

	.btn-danger {
		background: #ef4444;
		color: white;
	}

	.btn-danger:hover { background: #dc2626; }
</style>
