<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t, locale, loadAppTranslations } from '$lib/i18n';
	import { loadInstalledApps } from '$stores/desktop';
	import { openWindow } from '$stores/windows';
	import { taskProgress, type TaskProgress } from '$stores/websocket';

	interface AppPackage {
		id: string;
		name: string;
		description: string;
		icon: string;
		iconBg: string;
		version: string;
		size: string;
		status: 'not_installed' | 'installed' | 'installing' | 'update_available';
		category: string;
		dependencies: string[];
		author?: string;
		website?: string;
	}

	interface CatalogApp {
		id: string;
		name: string;
		version: string;
		category: string;
		icon?: string;
		description?: { en?: string; fr?: string } | string;
		dependencies?: string[];
		author?: string;
		website?: string;
	}

	interface InstalledPackage {
		id: string;
		name: string;
		version: string;
		status: string;
		frontend_config?: string;
	}

	interface FrontendConfig {
		component: string;
		icon: string;
		gradient: string;
		window?: {
			width?: number;
			height?: number;
			min_width?: number;
			min_height?: number;
		};
		config?: Record<string, unknown>;
	}

	let packages: AppPackage[] = [];
	let installedPackages: InstalledPackage[] = [];
	let loading = true;
	let searchQuery = '';
	let selectedCategory = 'all';
	let selectedPackage: AppPackage | null = null;
	let installError: string | null = null;

	// Installation progress tracking
	let activeTaskId: string | null = null;
	let activePackageId: string | null = null;

	// Reactive: get progress from WebSocket store for the active task
	$: activeProgress = activeTaskId ? $taskProgress[activeTaskId] ?? null : null;

	// Uninstall modal state
	let showUninstallModal = false;
	let uninstallTarget: AppPackage | null = null;
	let deleteAppData = true;

	const categories = [
		{ id: 'all', labelKey: 'all', icon: 'mdi:view-grid' },
		{ id: 'containers', labelKey: 'containers', icon: 'mdi:docker' },
		{ id: 'media', labelKey: 'media', icon: 'mdi:play-circle' },
		{ id: 'network', labelKey: 'network', icon: 'mdi:lan' },
		{ id: 'utilities', labelKey: 'utilities', icon: 'mdi:tools' }
	];

	const iconBgMap: Record<string, string> = {
		docker: 'bg-blue-500',
		containers: 'bg-blue-500',
		media: 'bg-purple-500',
		network: 'bg-indigo-500',
		utilities: 'bg-slate-500'
	};

	// Count apps per category (reactive)
	$: categoryCounts = packages.reduce((acc, pkg) => {
		acc[pkg.category] = (acc[pkg.category] || 0) + 1;
		return acc;
	}, {} as Record<string, number>);

	// Get category label for display
	function getCategoryLabel(categoryId: string): string {
		return $t.appCenter.categories[categoryId] || categoryId;
	}

	// Navigate to a category (close detail view)
	function selectCategory(categoryId: string) {
		selectedCategory = categoryId;
		selectedPackage = null;
		installError = null;
	}

	onMount(async () => {
		await loadPackages();
	});

	async function loadPackages() {
		loading = true;
		try {
			// Load installed packages
			const installedRes = await fetch('/api/packages');
			if (installedRes.ok) {
				installedPackages = await installedRes.json();
			}

			// Load catalog
			const catalogRes = await fetch('/api/packages/catalog');
			if (catalogRes.ok) {
				const catalog = await catalogRes.json();
				packages = (catalog.apps || []).map((app: CatalogApp) => {
					const installed = installedPackages.find((p) => p.id === app.id);
					return {
						id: app.id,
						name: app.name,
						description: getAppDescription(app.id, app.description),
						icon: app.icon || getDefaultIcon(app.category),
						iconBg: iconBgMap[app.id] || iconBgMap[app.category] || 'bg-slate-500',
						version: app.version,
						size: '~150 MB',
						status: installed ? (installed.status === 'installed' ? 'installed' : 'installing') : 'not_installed',
						category: app.category,
						dependencies: app.dependencies || [],
						author: app.author,
						website: app.website
					};
				});
			} else {
				packages = [];
			}
		} catch (error) {
			console.error('Failed to load packages:', error);
			packages = [];
		}
		loading = false;
	}

	// Check if a package is installed
	function isPackageInstalled(packageId: string): boolean {
		return installedPackages.some((p) => p.id === packageId && p.status === 'installed');
	}

	// Get missing dependencies for a package
	function getMissingDependencies(pkg: AppPackage): string[] {
		if (!pkg.dependencies || pkg.dependencies.length === 0) {
			return [];
		}
		return pkg.dependencies.filter((dep) => !isPackageInstalled(dep));
	}

	// Get human-readable names for dependencies
	function getDependencyNames(depIds: string[]): string[] {
		return depIds.map((depId) => {
			const depPkg = packages.find((p) => p.id === depId);
			return depPkg ? depPkg.name : depId;
		});
	}

	// Check if package can be installed (all dependencies satisfied)
	function canInstall(pkg: AppPackage): boolean {
		return getMissingDependencies(pkg).length === 0;
	}

	function getAppDescription(appId: string, catalogDescription?: { en?: string; fr?: string } | string): string {
		// Try to get localized description from app translations
		const appTranslations = ($t as any)[appId];
		if (appTranslations?.description) {
			return appTranslations.description;
		}
		// Try catalog description (localized)
		if (catalogDescription) {
			if (typeof catalogDescription === 'string') {
				return catalogDescription;
			}
			// Get current locale
			const currentLocale = $locale;
			return catalogDescription[currentLocale] || catalogDescription.en || '';
		}
		// Fallback to known descriptions
		if (appId === 'docker') {
			return $t.appCenter.packages.docker.description;
		}
		return '';
	}

	function getDefaultIcon(category: string): string {
		switch (category) {
			case 'containers': return 'mdi:docker';
			case 'media': return 'mdi:play-circle';
			case 'utilities': return 'mdi:tools';
			default: return 'mdi:package-variant';
		}
	}

	function getPackageFeatures(appId: string): string[] {
		// Try to get features from app-specific translations
		const appTrans = ($t as any).appCenter?.packages?.[appId];
		if (appTrans) {
			const features: string[] = [];
			// Look for feature1, feature2, feature3, etc.
			for (let i = 1; i <= 10; i++) {
				const featureKey = `feature${i}`;
				if (appTrans[featureKey]) {
					features.push(appTrans[featureKey]);
				}
			}
			if (features.length > 0) return features;
		}

		// Return empty array if no features found
		return [];
	}

	$: filteredPackages = packages.filter((pkg) => {
		const matchesSearch =
			pkg.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			pkg.description.toLowerCase().includes(searchQuery.toLowerCase());
		const matchesCategory =
			selectedCategory === 'all' ||
			selectedCategory === 'installed'
				? (selectedCategory === 'all' || pkg.status === 'installed')
				: pkg.category === selectedCategory;
		return matchesSearch && matchesCategory;
	});

	function selectPackage(pkg: AppPackage) {
		selectedPackage = pkg;
		installError = null;
	}

	function closeDetail() {
		selectedPackage = null;
		installError = null;
	}

	function getStatusLabel(status: AppPackage['status']): string {
		switch (status) {
			case 'installed':
				return $t.appCenter.status.installed;
			case 'installing':
				return $t.appCenter.status.installing;
			case 'update_available':
				return $t.appCenter.status.updateAvailable;
			default:
				return $t.appCenter.status.notInstalled;
		}
	}

	function getStatusColor(status: AppPackage['status']): string {
		switch (status) {
			case 'installed':
				return 'text-green-600 bg-green-50';
			case 'installing':
				return 'text-blue-600 bg-blue-50';
			case 'update_available':
				return 'text-orange-600 bg-orange-50';
			default:
				return 'text-slate-600 bg-slate-50';
		}
	}

	async function handleInstall(pkg: AppPackage | null) {
		if (!pkg) return;

		installError = null;

		// Update UI immediately
		const pkgIndex = packages.findIndex((p) => p.id === pkg.id);
		if (pkgIndex >= 0) {
			packages[pkgIndex] = { ...packages[pkgIndex], status: 'installing' };
			packages = [...packages];
		}
		if (selectedPackage?.id === pkg.id) {
			selectedPackage = { ...selectedPackage, status: 'installing' };
		}

		try {
			const response = await fetch('/api/packages/install', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ package_id: pkg.id })
			});

			if (!response.ok) {
				const error = await response.json();
				throw new Error(error.error || 'Installation failed');
			}

			const result = await response.json();
			if (result.task_id) {
				activeTaskId = result.task_id;
				activePackageId = pkg.id;
				await waitForTaskCompletion(result.task_id, pkg.id);
			}
		} catch (error) {
			console.error('Installation failed:', error);
			installError = error instanceof Error ? error.message : 'Installation failed';

			// Revert status
			if (pkgIndex >= 0) {
				packages[pkgIndex] = { ...packages[pkgIndex], status: 'not_installed' };
				packages = [...packages];
			}
			if (selectedPackage?.id === pkg.id) {
				selectedPackage = { ...selectedPackage, status: 'not_installed' };
			}
		} finally {
			activeTaskId = null;
			activePackageId = null;
		}
	}

	async function waitForTaskCompletion(taskId: string, packageId: string) {
		const maxAttempts = 120;
		let attempts = 0;

		while (attempts < maxAttempts) {
			// Check WebSocket store first (real-time)
			const wsProgress = $taskProgress[taskId];
			if (wsProgress) {
				if (wsProgress.status === 'completed') {
					await onInstallComplete(packageId);
					return;
				} else if (wsProgress.status === 'failed') {
					throw new Error(wsProgress.error_message || 'Installation failed');
				}
			}

			// Fallback: poll API every 2s
			try {
				const response = await fetch(`/api/packages/task/${taskId}`);
				if (response.ok) {
					const task = await response.json();
					if (task.status === 'completed') {
						await onInstallComplete(packageId);
						return;
					} else if (task.status === 'failed') {
						throw new Error(task.error_message || 'Installation failed');
					}
				}
			} catch (error) {
				if (error instanceof Error && error.message !== 'Installation failed') {
					console.error('Failed to check task status:', error);
				} else {
					throw error;
				}
			}

			await new Promise((resolve) => setTimeout(resolve, 2000));
			attempts++;
		}

		throw new Error('Installation timed out');
	}

	async function onInstallComplete(packageId: string) {
		await loadPackages();
		await loadInstalledApps();
		await loadAppTranslations(packageId);

		if (selectedPackage?.id === packageId) {
			selectedPackage = packages.find((p) => p.id === packageId) || null;
		}
	}

	function showUninstallConfirmation(pkg: AppPackage | null) {
		if (!pkg) return;
		uninstallTarget = pkg;
		deleteAppData = true;
		showUninstallModal = true;
	}

	function cancelUninstall() {
		showUninstallModal = false;
		uninstallTarget = null;
	}

	async function confirmUninstall() {
		if (!uninstallTarget) return;

		const pkg = uninstallTarget;
		showUninstallModal = false;

		try {
			const response = await fetch(`/api/packages/${pkg.id}?delete_data=${deleteAppData}`, {
				method: 'DELETE'
			});

			if (!response.ok) {
				const error = await response.json();
				throw new Error(error.error || 'Uninstall failed');
			}

			// Reload
			await loadPackages();
			await loadInstalledApps();

			if (selectedPackage?.id === pkg.id) {
				selectedPackage = packages.find((p) => p.id === pkg.id) || null;
			}
		} catch (error) {
			console.error('Uninstall failed:', error);
			installError = error instanceof Error ? error.message : 'Uninstall failed';
		}

		uninstallTarget = null;
	}

	function handleOpenApp(pkg: AppPackage | null) {
		if (!pkg) return;

		// Find the installed package to get frontend config
		const installed = installedPackages.find((p) => p.id === pkg.id);
		let component = pkg.id.charAt(0).toUpperCase() + pkg.id.slice(1) + 'App';
		let width = 1000;
		let height = 650;
		let appConfig: Record<string, unknown> | undefined;
		let gradient: string | undefined;

		// Use frontend config from installed package if available
		if (installed?.frontend_config) {
			try {
				const frontendConfig: FrontendConfig = JSON.parse(installed.frontend_config);
				if (frontendConfig.component) {
					component = frontendConfig.component;
				}
				if (frontendConfig.window) {
					width = frontendConfig.window.width || width;
					height = frontendConfig.window.height || height;
				}
				if (frontendConfig.gradient) {
					gradient = frontendConfig.gradient;
				}
				// Pass config to the component (e.g., port, path for IframeApp)
				if (frontendConfig.config) {
					appConfig = frontendConfig.config;
				}
			} catch (e) {
				console.warn('Failed to parse frontend config:', e);
			}
		}

		// Open the app window
		openWindow({
			id: pkg.id,
			title: pkg.name,
			icon: pkg.icon,
			component,
			x: 150 + Math.random() * 100,
			y: 80 + Math.random() * 50,
			width,
			height,
			appConfig,
			gradient
		});
	}
</script>

<div class="app-center">
	<!-- Sidebar -->
	<aside class="sidebar">
		<div class="sidebar-header">
			<Icon icon="mdi:store" class="w-6 h-6 text-blue-500" />
			<span class="sidebar-title">{$t.appCenter.title}</span>
		</div>

		<nav class="sidebar-nav">
			{#each categories as category}
				<button
					class="nav-item"
					class:active={selectedCategory === category.id && !selectedPackage}
					on:click={() => selectCategory(category.id)}
				>
					<Icon icon={category.icon} class="w-5 h-5" />
					<span>{$t.appCenter.categories[category.labelKey]}</span>
					{#if category.id !== 'all'}
						<span class="nav-count">{categoryCounts[category.id] || 0}</span>
					{/if}
				</button>
			{/each}
		</nav>

		<div class="sidebar-footer">
			<button
				class="stats-button"
				class:active={selectedCategory === 'installed'}
				on:click={() => { selectCategory(selectedCategory === 'installed' ? 'all' : 'installed'); }}
			>
				<span class="stat-value">{packages.filter((p) => p.status === 'installed').length}</span>
				<span class="stat-label">{$t.appCenter.installedCount}</span>
			</button>
		</div>
	</aside>

	<!-- Main Content -->
	<main class="main-content">
		<!-- Package Grid or Detail View -->
		{#if loading}
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-12 h-12 text-blue-500 animate-spin" />
				<p>{$t.common.loading}</p>
			</div>
		{:else if selectedPackage}
			<!-- Detail View -->
			<div class="package-detail">
				<!-- Breadcrumb -->
				<div class="breadcrumb">
					<button class="breadcrumb-link" on:click={() => selectCategory(selectedPackage?.category || 'all')}>
						{getCategoryLabel(selectedPackage.category)}
					</button>
					<Icon icon="mdi:chevron-right" class="breadcrumb-sep" />
					<span class="breadcrumb-current">{selectedPackage.name}</span>
				</div>

				<div class="detail-header">
					<div class="detail-icon {selectedPackage.iconBg}">
						<Icon icon={selectedPackage.icon} class="w-16 h-16 text-white" />
					</div>
					<div class="detail-info">
						<h1>{selectedPackage.name}</h1>
						<p class="detail-meta">
							{$t.appCenter.version}: {selectedPackage.version}
							{#if selectedPackage.author}
								· {selectedPackage.author}
							{/if}
						</p>
						<div class="detail-badges">
							<span class="status-badge {getStatusColor(selectedPackage.status)}">
								{getStatusLabel(selectedPackage.status)}
							</span>
							<span class="category-badge">
								{getCategoryLabel(selectedPackage.category)}
							</span>
						</div>
					</div>
					<div class="detail-actions">
						{#if selectedPackage.status === 'not_installed'}
							{@const missingDeps = getMissingDependencies(selectedPackage)}
							{#if missingDeps.length > 0}
								<button class="btn-primary btn-disabled" disabled title={$t.appCenter.missingDependencies || 'Missing dependencies'}>
									<Icon icon="mdi:download" class="w-5 h-5" />
									{$t.appCenter.actions.install}
								</button>
								<p class="dependency-warning">
									<Icon icon="mdi:alert" class="w-4 h-4" />
									{$t.appCenter.requiresInstall || 'Requires'}: {getDependencyNames(missingDeps).join(', ')}
								</p>
							{:else}
								<button class="btn-primary" on:click={() => handleInstall(selectedPackage)}>
									<Icon icon="mdi:download" class="w-5 h-5" />
									{$t.appCenter.actions.install}
								</button>
							{/if}
						{:else if selectedPackage.status === 'installed'}
							<button class="btn-secondary" on:click={() => handleOpenApp(selectedPackage)}>
								<Icon icon="mdi:open-in-new" class="w-5 h-5" />
								{$t.appCenter.actions.open}
							</button>
							<button class="btn-danger" on:click={() => showUninstallConfirmation(selectedPackage)}>
								<Icon icon="mdi:delete" class="w-5 h-5" />
								{$t.appCenter.actions.uninstall}
							</button>
						{:else if selectedPackage.status === 'installing'}
							<div class="install-progress">
								<div class="progress-header">
									<Icon icon="mdi:loading" class="w-5 h-5 animate-spin text-blue-500" />
									<span class="progress-label">
										{#if activeProgress}
											{activeProgress.current_step || $t.appCenter.actions.installing}
										{:else}
											{$t.appCenter.actions.installing}
										{/if}
									</span>
								</div>
								<div class="progress-bar-container">
									<div
										class="progress-bar-fill"
										style="width: {activeProgress ? activeProgress.progress_percent : 0}%"
									></div>
								</div>
								<span class="progress-percent">
									{activeProgress ? activeProgress.progress_percent : 0}%
								</span>
							</div>
						{/if}
						{#if installError}
							<p class="error-message">{installError}</p>
						{/if}
					</div>
				</div>

				<div class="detail-description">
					<h2>{$t.appCenter.description}</h2>
					<p>{selectedPackage.description}</p>
					{#if selectedPackage.website}
						<a href={selectedPackage.website} target="_blank" rel="noopener noreferrer" class="website-link">
							<Icon icon="mdi:open-in-new" class="w-4 h-4" />
							{selectedPackage.website}
						</a>
					{/if}
				</div>

				{#if selectedPackage.dependencies && selectedPackage.dependencies.length > 0}
				<div class="detail-dependencies">
					<h2>{$t.appCenter.dependencies}</h2>
					<ul>
						{#each selectedPackage.dependencies as depId}
							{@const depPkg = packages.find(p => p.id === depId)}
							{@const isInstalled = isPackageInstalled(depId)}
							<li class:installed={isInstalled} class:missing={!isInstalled}>
								<Icon icon={isInstalled ? 'mdi:check-circle' : 'mdi:alert-circle'} class="w-4 h-4" />
								<span>{depPkg?.name || depId}</span>
								<span class="dep-status">{isInstalled ? $t.appCenter.status.installed : $t.appCenter.status.notInstalled}</span>
							</li>
						{/each}
					</ul>
				</div>
				{/if}

				{#if getPackageFeatures(selectedPackage.id).length > 0}
				<div class="detail-features">
					<h2>{$t.appCenter.features}</h2>
					<ul>
						{#each getPackageFeatures(selectedPackage.id) as feature}
						<li><Icon icon="mdi:check" class="w-4 h-4 text-green-500" /> {feature}</li>
						{/each}
					</ul>
				</div>
				{/if}
			</div>
		{:else}
			<!-- List View -->
			<header class="content-header">
				<div class="content-header-left">
					<h2 class="grid-title">
						{#if selectedCategory === 'installed'}
							{$t.appCenter.installedCount}
						{:else if selectedCategory === 'all'}
							{$t.appCenter.title}
						{:else}
							{getCategoryLabel(selectedCategory)}
						{/if}
					</h2>
					<span class="grid-count">{filteredPackages.length} {filteredPackages.length === 1 ? 'app' : 'apps'}</span>
				</div>
				<div class="search-box">
					<Icon icon="mdi:magnify" class="search-icon" />
					<input
						type="text"
						placeholder={$t.appCenter.searchPlaceholder}
						bind:value={searchQuery}
					/>
					{#if searchQuery}
						<button class="search-clear" on:click={() => searchQuery = ''}>
							<Icon icon="mdi:close" class="w-4 h-4" />
						</button>
					{/if}
				</div>
			</header>

			<div class="package-grid">
				{#if filteredPackages.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:package-variant" class="w-16 h-16 text-slate-300" />
						<p>{$t.appCenter.noPackages}</p>
					</div>
				{:else}
					{#each filteredPackages as pkg}
						{@const hasMissingDeps = pkg.status === 'not_installed' && getMissingDependencies(pkg).length > 0}
						<button class="package-card" class:has-missing-deps={hasMissingDeps} on:click={() => selectPackage(pkg)}>
							<div class="package-icon {pkg.iconBg}">
								<Icon icon={pkg.icon} class="w-10 h-10 text-white" />
							</div>
							<div class="package-info">
								<h3>{pkg.name}</h3>
								<p>{pkg.description}</p>
								<div class="package-meta">
									<span class="version">{pkg.version}</span>
									{#if hasMissingDeps}
										<span class="deps-indicator" title={$t.appCenter.missingDependencies}>
											<Icon icon="mdi:link-variant-off" class="w-4 h-4 text-amber-500" />
										</span>
									{/if}
									{#if pkg.status === 'installing' && activePackageId === pkg.id && activeProgress}
										<span class="mini-progress">{activeProgress.progress_percent}%</span>
									{:else}
										<span class="status-dot {pkg.status === 'installed' ? 'installed' : ''}"></span>
									{/if}
								</div>
								{#if pkg.status === 'installing' && activePackageId === pkg.id && activeProgress}
									<div class="mini-progress-bar">
										<div class="mini-progress-fill" style="width: {activeProgress.progress_percent}%"></div>
									</div>
								{/if}
							</div>
							{#if pkg.status === 'installed'}
								<button
									class="card-open-btn"
									on:click|stopPropagation={() => handleOpenApp(pkg)}
									title={$t.appCenter.actions.open}
								>
									<Icon icon="mdi:open-in-new" class="w-4 h-4" />
								</button>
							{/if}
							<Icon icon="mdi:chevron-right" class="chevron" />
						</button>
					{/each}
				{/if}
			</div>
		{/if}
	</main>

	<!-- Uninstall Confirmation Modal -->
	{#if showUninstallModal && uninstallTarget}
		<div class="modal-overlay" on:click={cancelUninstall} on:keydown={(e) => e.key === 'Escape' && cancelUninstall()} role="button" tabindex="0">
			<div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
				<div class="modal-header">
					<Icon icon="mdi:alert-circle" class="w-6 h-6 text-red-500" />
					<h3>{$t.appCenter.uninstallModal?.title || 'Uninstall Application'}</h3>
				</div>
				<div class="modal-body">
					<p>{$t.appCenter.uninstallModal?.message || 'Do you want to uninstall'} <strong>{uninstallTarget.name}</strong> ?</p>
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={deleteAppData} />
						<span>{$t.appCenter.uninstallModal?.deleteData || 'Delete application data'}</span>
					</label>
				</div>
				<div class="modal-actions">
					<button class="btn-secondary" on:click={cancelUninstall}>
						{$t.common.cancel}
					</button>
					<button class="btn-danger" on:click={confirmUninstall}>
						<Icon icon="mdi:delete" class="w-5 h-5" />
						{$t.appCenter.actions.uninstall}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.app-center {
		display: flex;
		height: 100%;
		background: #f8fafc;
	}

	/* Sidebar */
	.sidebar {
		width: 220px;
		background: white;
		border-right: 1px solid #e2e8f0;
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 16px 20px;
		border-bottom: 1px solid #e2e8f0;
	}

	.sidebar-title {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
	}

	.sidebar-nav {
		flex: 1;
		padding: 12px 8px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 12px;
		border: none;
		background: transparent;
		border-radius: 8px;
		color: #64748b;
		font-size: 14px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.nav-item:hover {
		background: #f1f5f9;
		color: #334155;
	}

	.nav-item.active {
		background: #eff6ff;
		color: #2563eb;
	}

	.nav-count {
		margin-left: auto;
		font-size: 11px;
		font-weight: 500;
		color: #94a3b8;
		background: #f1f5f9;
		padding: 1px 7px;
		border-radius: 10px;
	}

	.nav-item.active .nav-count {
		color: #3b82f6;
		background: #dbeafe;
	}

	.sidebar-footer {
		padding: 16px 20px;
		border-top: 1px solid #e2e8f0;
	}

	.stats-button {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 100%;
		padding: 8px;
		border: none;
		background: transparent;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.stats-button:hover {
		background: #f1f5f9;
	}

	.stats-button.active {
		background: #eff6ff;
	}

	.stats-button.active .stat-value {
		color: #2563eb;
	}

	.stats-button.active .stat-label {
		color: #2563eb;
	}

	.stat-value {
		font-size: 24px;
		font-weight: 700;
		color: #1e293b;
	}

	.stat-label {
		font-size: 12px;
		color: #64748b;
	}

	/* Main Content */
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.content-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 16px 24px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.content-header-left {
		display: flex;
		align-items: baseline;
		gap: 10px;
	}

	.grid-title {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
		margin: 0;
	}

	.grid-count {
		font-size: 13px;
		color: #94a3b8;
	}

	.search-box {
		position: relative;
		max-width: 400px;
	}

	.search-box input {
		width: 100%;
		padding: 10px 36px 10px 40px;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		font-size: 14px;
		background: #f8fafc;
		color: #1e293b;
	}

	.search-box input:focus {
		outline: none;
		border-color: #3b82f6;
		background: white;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		width: 20px;
		height: 20px;
		color: #94a3b8;
	}

	.search-clear {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		background: none;
		border: none;
		color: #94a3b8;
		cursor: pointer;
		padding: 4px;
		display: flex;
		align-items: center;
		border-radius: 4px;
	}

	.search-clear:hover {
		color: #64748b;
		background: #f1f5f9;
	}

	/* Package Grid */
	.package-grid {
		flex: 1;
		padding: 24px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.package-card {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
		text-align: left;
		position: relative;
		overflow: hidden;
		flex-shrink: 0;
	}

	.package-card:hover {
		border-color: #cbd5e1;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
	}

	.package-icon {
		width: 56px;
		height: 56px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.package-info {
		flex: 1;
		min-width: 0;
	}

	.package-info h3 {
		font-size: 15px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 4px;
	}

	.package-info p {
		font-size: 13px;
		color: #64748b;
		margin-bottom: 8px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.package-meta {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.version {
		font-size: 12px;
		color: #94a3b8;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #e2e8f0;
	}

	.status-dot.installed {
		background: #22c55e;
	}

	.card-open-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: 1px solid #e2e8f0;
		background: white;
		color: #64748b;
		cursor: pointer;
		transition: all 0.15s ease;
		flex-shrink: 0;
	}

	.card-open-btn:hover {
		background: #eff6ff;
		color: #3b82f6;
		border-color: #3b82f6;
	}

	.chevron {
		width: 20px;
		height: 20px;
		color: #94a3b8;
	}

	/* Empty State */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
	}

	.empty-state p {
		color: #64748b;
		font-size: 14px;
	}

	/* Detail View */
	.package-detail {
		flex: 1;
		padding: 24px;
		overflow-y: auto;
	}

	/* Breadcrumb */
	.breadcrumb {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 16px;
		font-size: 14px;
	}

	.breadcrumb-link {
		background: none;
		border: none;
		color: #3b82f6;
		cursor: pointer;
		font-size: 14px;
		padding: 0;
	}

	.breadcrumb-link:hover {
		color: #2563eb;
		text-decoration: underline;
	}

	:global(.breadcrumb-sep) {
		width: 16px;
		height: 16px;
		color: #94a3b8;
	}

	.breadcrumb-current {
		color: #64748b;
	}

	.detail-header {
		display: flex;
		gap: 20px;
		padding: 24px;
		background: white;
		border-radius: 12px;
		margin-bottom: 20px;
	}

	.detail-icon {
		width: 96px;
		height: 96px;
		border-radius: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.detail-info {
		flex: 1;
	}

	.detail-info h1 {
		font-size: 24px;
		font-weight: 700;
		color: #1e293b;
		margin-bottom: 8px;
	}

	.detail-meta {
		font-size: 14px;
		color: #64748b;
		margin-bottom: 12px;
	}

	.detail-badges {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.status-badge {
		display: inline-block;
		padding: 4px 10px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
	}

	.category-badge {
		display: inline-block;
		padding: 4px 10px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
		color: #64748b;
		background: #f1f5f9;
	}

	.detail-actions {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.btn-primary,
	.btn-secondary,
	.btn-danger {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 20px;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
		border: none;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: #f1f5f9;
		color: #334155;
	}

	.btn-secondary:hover {
		background: #e2e8f0;
	}

	.btn-danger {
		background: #fef2f2;
		color: #dc2626;
	}

	.btn-danger:hover {
		background: #fee2e2;
	}

	.detail-description,
	.detail-features,
	.detail-dependencies {
		background: white;
		border-radius: 12px;
		padding: 20px;
		margin-bottom: 16px;
	}

	.detail-description h2,
	.detail-features h2,
	.detail-dependencies h2 {
		font-size: 16px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 12px;
	}

	.detail-description p {
		font-size: 14px;
		color: #64748b;
		line-height: 1.6;
	}

	.website-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: #3b82f6;
		font-size: 13px;
		text-decoration: none;
		margin-top: 10px;
	}

	.website-link:hover {
		color: #2563eb;
		text-decoration: underline;
	}

	.detail-features ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.detail-features li {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 0;
		font-size: 14px;
		color: #334155;
		border-bottom: 1px solid #f1f5f9;
	}

	.detail-features li:last-child {
		border-bottom: none;
	}

	.detail-dependencies ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.detail-dependencies li {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 0;
		font-size: 14px;
		border-bottom: 1px solid #f1f5f9;
	}

	.detail-dependencies li:last-child {
		border-bottom: none;
	}

	.detail-dependencies li.installed {
		color: #16a34a;
	}

	.detail-dependencies li.missing {
		color: #d97706;
	}

	.detail-dependencies .dep-status {
		margin-left: auto;
		font-size: 12px;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.detail-dependencies li.installed .dep-status {
		background: #dcfce7;
		color: #16a34a;
	}

	.detail-dependencies li.missing .dep-status {
		background: #fef3c7;
		color: #d97706;
	}

	.package-card.has-missing-deps {
		opacity: 0.8;
	}

	.deps-indicator {
		display: flex;
		align-items: center;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.animate-spin {
		animation: spin 1s linear infinite;
	}

	.error-message {
		color: #dc2626;
		font-size: 13px;
		margin-top: 8px;
		text-align: center;
	}

	.dependency-warning {
		display: flex;
		align-items: center;
		gap: 6px;
		color: #d97706;
		font-size: 13px;
		padding: 8px 12px;
		background: #fef3c7;
		border-radius: 6px;
		margin-top: 8px;
	}

	.btn-disabled {
		opacity: 0.5;
		cursor: not-allowed;
		background: #94a3b8 !important;
	}

	.btn-disabled:hover {
		background: #94a3b8 !important;
	}

	/* Installation progress bar (detail view) */
	.install-progress {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 200px;
	}

	.progress-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.progress-label {
		font-size: 13px;
		color: #3b82f6;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 180px;
	}

	.progress-bar-container {
		width: 100%;
		height: 8px;
		background: #e2e8f0;
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-bar-fill {
		height: 100%;
		background: linear-gradient(90deg, #3b82f6, #2563eb);
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.progress-percent {
		font-size: 12px;
		color: #64748b;
		text-align: right;
		font-weight: 600;
	}

	/* Mini progress bar (grid card view) */
	.mini-progress {
		font-size: 11px;
		color: #3b82f6;
		font-weight: 600;
	}

	.mini-progress-bar {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 3px;
		background: #e2e8f0;
		border-radius: 0 0 12px 12px;
		overflow: hidden;
	}

	.mini-progress-fill {
		height: 100%;
		background: linear-gradient(90deg, #3b82f6, #2563eb);
		transition: width 0.3s ease;
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
	}

	.loading-state p {
		color: #64748b;
		font-size: 14px;
	}

	/* Modal styles */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 12px;
		width: 100%;
		max-width: 420px;
		box-shadow: 0 20px 50px rgba(0, 0, 0, 0.2);
	}

	.modal-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 20px 24px;
		border-bottom: 1px solid #e2e8f0;
	}

	.modal-header h3 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
		margin: 0;
	}

	.modal-body {
		padding: 20px 24px;
	}

	.modal-body p {
		font-size: 14px;
		color: #475569;
		margin-bottom: 16px;
		line-height: 1.5;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 10px;
		cursor: pointer;
		font-size: 14px;
		color: #334155;
		padding: 12px;
		background: #f8fafc;
		border-radius: 8px;
		border: 1px solid #e2e8f0;
	}

	.checkbox-label input[type="checkbox"] {
		width: 18px;
		height: 18px;
		accent-color: #3b82f6;
		cursor: pointer;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
		padding: 16px 24px;
		border-top: 1px solid #e2e8f0;
		background: #f8fafc;
		border-radius: 0 0 12px 12px;
	}
</style>
