<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t, locale } from '$lib/i18n';
	import { onMount } from 'svelte';
	import { api, type FileItem as ApiFileItem } from '$lib/stores/api';

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

	interface FolderNode {
		id: string;
		name: string;
		nameKey: string;
		icon: string;
		expanded: boolean;
		children?: FolderNode[];
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
	let renameTarget: FileItem | null = null;
	let renameValue = '';

	// Sidebar folders (simplified for MVP)
	let sidebarFolders: FolderNode[] = [
		{
			id: 'root',
			name: 'Fichiers',
			nameKey: 'personalFolder',
			icon: 'mdi:folder-home',
			expanded: true
		}
	];

	// Get icon for file type
	function getFileIcon(item: ApiFileItem): string {
		if (item.type === 'folder') {
			return 'mdi:folder';
		}
		// Determine icon based on mime type
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

	// Load files from API
	async function loadFiles(path: string = '') {
		loading = true;
		error = null;
		try {
			const apiFiles = await api.getFiles(path);
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

	// Load files on mount
	onMount(() => {
		loadFiles();
	});

	// Navigation history
	let historyIndex = 0;
	let history: string[] = ['personal'];

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
		// Update history
		if (history[historyIndex] !== path) {
			history = [...history.slice(0, historyIndex + 1), path];
			historyIndex = history.length - 1;
		}
		loadFiles(path);
	}

	function toggleFolderExpand(folder: FolderNode) {
		folder.expanded = !folder.expanded;
		sidebarFolders = [...sidebarFolders];
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

	// Create new folder
	async function handleCreateFolder() {
		const name = prompt($t.fileManager.toolbar.newFolderPrompt || 'Nom du dossier:');
		if (name && name.trim()) {
			try {
				await api.createFolder(currentPath, name.trim());
				await loadFiles(currentPath);
			} catch (e) {
				alert(e instanceof Error ? e.message : 'Error creating folder');
			}
		}
	}

	// Delete file/folder
	async function handleDelete(file: FileItem) {
		const confirmMsg = file.type === 'folder'
			? `Supprimer le dossier "${file.name}" et tout son contenu ?`
			: `Supprimer "${file.name}" ?`;

		if (confirm(confirmMsg)) {
			try {
				await api.deleteFile(file.path);
				await loadFiles(currentPath);
			} catch (e) {
				alert(e instanceof Error ? e.message : 'Error deleting file');
			}
		}
	}

	// Start rename
	function startRename(file: FileItem) {
		renameTarget = file;
		renameValue = file.name;
		showActionMenu = null;
	}

	// Confirm rename
	async function confirmRename() {
		if (renameTarget && renameValue.trim() && renameValue !== renameTarget.name) {
			try {
				await api.renameFile(renameTarget.path, renameValue.trim());
				await loadFiles(currentPath);
			} catch (e) {
				alert(e instanceof Error ? e.message : 'Error renaming file');
			}
		}
		renameTarget = null;
		renameValue = '';
	}

	// Cancel rename
	function cancelRename() {
		renameTarget = null;
		renameValue = '';
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
			// Navigate into folder
			selectFolder(file.path);
		} else {
			// Open file (not implemented yet)
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

	function getCurrentFolderName(): string {
		if (!currentPath) return '/';
		return '/' + currentPath;
	}

	// Go to parent folder
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
</script>

<svelte:window on:click={closeMenus} />

<!-- Rename dialog -->
{#if renameTarget}
	<div class="rename-overlay" on:click={cancelRename}>
		<div class="rename-dialog" on:click|stopPropagation>
			<h3>Renommer</h3>
			<input
				type="text"
				bind:value={renameValue}
				on:keydown={(e) => {
					if (e.key === 'Enter') confirmRename();
					if (e.key === 'Escape') cancelRename();
				}}
				autofocus
			/>
			<div class="rename-actions">
				<button class="btn-cancel" on:click={cancelRename}>Annuler</button>
				<button class="btn-confirm" on:click={confirmRename}>Renommer</button>
			</div>
		</div>
	</div>
{/if}

<div class="file-manager">
	<!-- Sidebar -->
	<aside class="sidebar">
		<nav class="sidebar-nav">
			{#each sidebarFolders as folder}
				<div class="sidebar-folder">
					<div
						class="sidebar-item"
						class:active={currentPath === '' && folder.id === 'root'}
					>
						<button
							class="expand-btn"
							on:click|stopPropagation={() => toggleFolderExpand(folder)}
						>
							<Icon
								icon={folder.expanded ? 'mdi:chevron-down' : 'mdi:chevron-right'}
								class="w-4 h-4"
							/>
						</button>
						<button
							class="folder-name-btn"
							on:click={() => selectFolder('')}
						>
							<Icon icon={folder.icon} class="w-4 h-4 mr-2 folder-icon" />
							{folder.name}
						</button>
					</div>
					{#if folder.expanded && folder.children}
						<div class="sidebar-children">
							{#each folder.children as child}
								<button class="sidebar-item child">
									<span>{child.name}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
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
				<span>{getCurrentFolderName()}</span>
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
				<button class="action-btn" title={$t.fileManager.toolbar.newFolder} on:click={handleCreateFolder}>
					<Icon icon="mdi:folder-plus" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.upload}>
					<Icon icon="mdi:upload" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.copy}>
					<Icon icon="mdi:content-copy" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.paste}>
					<Icon icon="mdi:content-paste" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.cut}>
					<Icon icon="mdi:content-cut" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.duplicate}>
					<Icon icon="mdi:file-multiple-outline" class="w-5 h-5" />
				</button>
			</div>
			<div class="action-right">
				<button class="action-btn" title={$t.fileManager.toolbar.archive}>
					<Icon icon="mdi:archive-outline" class="w-5 h-5" />
				</button>
				<button class="action-btn" title={$t.fileManager.toolbar.settings}>
					<Icon icon="mdi:cog-outline" class="w-5 h-5" />
				</button>
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

		<!-- File list -->
		<div class="file-list-container">
			{#if loading}
				<div class="loading-state">
					<Icon icon="mdi:loading" class="w-8 h-8 animate-spin" />
					<span>Chargement...</span>
				</div>
			{:else if error}
				<div class="error-state">
					<Icon icon="mdi:alert-circle" class="w-8 h-8" />
					<span>{error}</span>
					<button class="retry-btn" on:click={refresh}>Réessayer</button>
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
										on:click|stopPropagation={() => showActionMenu = showActionMenu === file.id ? null : file.id}
									>
										<Icon icon="mdi:dots-vertical" class="w-5 h-5" />
									</button>
									{#if showActionMenu === file.id}
										<div class="action-menu">
											{#if file.type === 'folder'}
												<button class="menu-item" on:click={() => { handleDoubleClick(file); showActionMenu = null; }}>
													<Icon icon="mdi:folder-open" class="w-4 h-4" />
													{$t.common.open}
												</button>
											{/if}
											<button class="menu-item" on:click={() => startRename(file)}>
												<Icon icon="mdi:pencil" class="w-4 h-4" />
												{$t.fileManager.contextMenu.rename}
											</button>
											<div class="menu-divider"></div>
											<button class="menu-item danger" on:click={() => { handleDelete(file); showActionMenu = null; }}>
												<Icon icon="mdi:delete" class="w-4 h-4" />
												{$t.common.delete}
											</button>
										</div>
									{/if}
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
				<!-- Compact view -->
				<div class="file-compact">
					{#each sortedFiles as file}
						<button
							class="file-compact-item"
							class:selected={selectedFiles.includes(file.id)}
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
					<span>Ce dossier est vide</span>
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

	.sidebar-folder {
		display: flex;
		flex-direction: column;
	}

	.sidebar-item {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		padding: 10px 12px 10px 8px;
		font-size: 14px;
		color: #374151;
		text-align: left;
		transition: all 0.15s ease;
		border-left: 3px solid transparent;
	}

	.sidebar-item:hover {
		background: #eef2ff;
	}

	.sidebar-item.active {
		background: #eef2ff;
		border-left-color: #3b82f6;
		color: #1f2937;
	}

	.sidebar-item.child {
		padding-left: 40px;
		font-size: 13px;
		cursor: pointer;
	}

	.expand-btn {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #6b7280;
		border-radius: 4px;
		flex-shrink: 0;
	}

	.expand-btn:hover {
		background: rgba(0, 0, 0, 0.08);
	}

	.folder-name-btn {
		flex: 1;
		text-align: left;
		font-weight: 500;
		color: inherit;
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
	}

	.sidebar-children {
		display: flex;
		flex-direction: column;
	}

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

	.search-input input::placeholder {
		color: #9ca3af;
	}

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

	.action-btn:hover {
		background: #f3f4f6;
		color: #374151;
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

	.view-option:hover {
		background: #f3f4f6;
	}

	.view-option.active {
		background: #eef2ff;
		color: #3b82f6;
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

	.file-table th.sortable:hover {
		color: #374151;
	}

	.sort-icon {
		margin-left: 4px;
		color: #3b82f6;
	}

	.col-size {
		width: 100px;
	}

	.col-type {
		width: 120px;
	}

	.col-modified {
		width: 160px;
	}

	.col-actions {
		width: 48px;
	}

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

	.file-row:hover {
		background: #f9fafb;
	}

	.file-row.selected {
		background: #eef2ff;
	}

	.cell-name {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.file-icon {
		flex-shrink: 0;
	}

	.folder-icon {
		color: #f59e0b;
	}

	.trash-icon {
		color: #9ca3af;
	}

	.file-icon-default {
		color: #6b7280;
	}

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

	.file-row:hover .action-menu-btn {
		opacity: 1;
	}

	.action-menu-btn:hover {
		background: #e5e7eb;
		color: #374151;
	}

	.action-menu {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 4px;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		min-width: 160px;
		z-index: 50;
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

	.menu-item:hover {
		background: #f3f4f6;
	}

	.menu-item.danger {
		color: #ef4444;
	}

	.menu-item.danger:hover {
		background: #fef2f2;
	}

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

	.file-card:hover {
		background: #f3f4f6;
	}

	.file-card.selected {
		background: #eef2ff;
	}

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

	.file-compact-item:hover {
		background: #f3f4f6;
	}

	.file-compact-item.selected {
		background: #eef2ff;
	}

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

	.selected-info {
		color: #3b82f6;
	}

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

	.error-state {
		color: #ef4444;
	}

	.retry-btn {
		margin-top: 8px;
		padding: 8px 16px;
		background: #3b82f6;
		color: white;
		border-radius: 6px;
		font-size: 14px;
	}

	.retry-btn:hover {
		background: #2563eb;
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Rename dialog */
	.rename-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.rename-dialog {
		background: white;
		padding: 24px;
		border-radius: 12px;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
		min-width: 300px;
	}

	.rename-dialog h3 {
		margin: 0 0 16px 0;
		font-size: 18px;
		font-weight: 600;
		color: #1f2937;
	}

	.rename-dialog input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		outline: none;
	}

	.rename-dialog input:focus {
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.rename-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 16px;
	}

	.btn-cancel,
	.btn-confirm {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
	}

	.btn-cancel {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-cancel:hover {
		background: #e5e7eb;
	}

	.btn-confirm {
		background: #3b82f6;
		color: white;
	}

	.btn-confirm:hover {
		background: #2563eb;
	}
</style>
