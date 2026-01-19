<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { t, locale, loadAppTranslations } from '$lib/i18n';
	import { loadInstalledApps } from '$stores/desktop';
	import { openWindow } from '$stores/windows';

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
	}

	interface CatalogApp {
		id: string;
		name: string;
		version: string;
		category: string;
		icon?: string;
		description?: { en?: string; fr?: string } | string;
	}

	interface InstalledPackage {
		id: string;
		name: string;
		version: string;
		status: string;
	}

	let packages: AppPackage[] = [];
	let installedPackages: InstalledPackage[] = [];
	let loading = true;
	let searchQuery = '';
	let selectedCategory = 'all';
	let selectedPackage: AppPackage | null = null;
	let installError: string | null = null;

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
						category: app.category
					};
				});
			} else {
				// Fallback to static Docker entry if catalog unavailable
				packages = [{
					id: 'docker',
					name: 'Docker',
					description: $t.appCenter.packages.docker.description,
					icon: 'mdi:docker',
					iconBg: 'bg-blue-500',
					version: '24.0.7',
					size: '~150 MB',
					status: installedPackages.some((p) => p.id === 'docker' && p.status === 'installed') ? 'installed' : 'not_installed',
					category: 'containers'
				}];
			}
		} catch (error) {
			console.error('Failed to load packages:', error);
			// Fallback
			packages = [{
				id: 'docker',
				name: 'Docker',
				description: $t.appCenter.packages.docker.description,
				icon: 'mdi:docker',
				iconBg: 'bg-blue-500',
				version: '24.0.7',
				size: '~150 MB',
				status: 'not_installed',
				category: 'containers'
			}];
		}
		loading = false;
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

	$: filteredPackages = packages.filter((pkg) => {
		const matchesSearch =
			pkg.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
			pkg.description.toLowerCase().includes(searchQuery.toLowerCase());
		const matchesCategory = selectedCategory === 'all' || pkg.category === selectedCategory;
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

			// Poll for completion
			const result = await response.json();
			if (result.task_id) {
				await pollTaskStatus(result.task_id, pkg.id);
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
		}
	}

	async function pollTaskStatus(taskId: string, packageId: string) {
		const maxAttempts = 60;
		let attempts = 0;

		while (attempts < maxAttempts) {
			try {
				const response = await fetch(`/api/packages/task/${taskId}`);
				if (response.ok) {
					const task = await response.json();
					if (task.status === 'completed') {
						// Success! Reload everything
						await loadPackages();
						await loadInstalledApps();
						await loadAppTranslations(packageId);

						if (selectedPackage?.id === packageId) {
							selectedPackage = packages.find((p) => p.id === packageId) || null;
						}
						return;
					} else if (task.status === 'failed') {
						throw new Error(task.error_message || 'Installation failed');
					}
				}
			} catch (error) {
				console.error('Failed to check task status:', error);
			}

			await new Promise((resolve) => setTimeout(resolve, 2000));
			attempts++;
		}

		throw new Error('Installation timed out');
	}

	async function handleUninstall(pkg: AppPackage | null) {
		if (!pkg) return;

		try {
			const response = await fetch(`/api/packages/${pkg.id}`, {
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
	}

	function handleOpenApp(pkg: AppPackage | null) {
		if (!pkg) return;

		// Open the app window
		openWindow({
			id: pkg.id,
			title: pkg.name,
			icon: pkg.icon,
			component: pkg.id.charAt(0).toUpperCase() + pkg.id.slice(1) + 'App',
			x: 150 + Math.random() * 100,
			y: 80 + Math.random() * 50,
			width: 1000,
			height: 650
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
					class:active={selectedCategory === category.id}
					on:click={() => (selectedCategory = category.id)}
				>
					<Icon icon={category.icon} class="w-5 h-5" />
					<span>{$t.appCenter.categories[category.labelKey]}</span>
				</button>
			{/each}
		</nav>

		<div class="sidebar-footer">
			<div class="stats">
				<span class="stat-value">{packages.filter((p) => p.status === 'installed').length}</span>
				<span class="stat-label">{$t.appCenter.installedCount}</span>
			</div>
		</div>
	</aside>

	<!-- Main Content -->
	<main class="main-content">
		<!-- Header -->
		<header class="content-header">
			<div class="search-box">
				<Icon icon="mdi:magnify" class="search-icon" />
				<input
					type="text"
					placeholder={$t.appCenter.searchPlaceholder}
					bind:value={searchQuery}
				/>
			</div>
		</header>

		<!-- Package Grid or Detail View -->
		{#if loading}
			<div class="loading-state">
				<Icon icon="mdi:loading" class="w-12 h-12 text-blue-500 animate-spin" />
				<p>{$t.common.loading}</p>
			</div>
		{:else if selectedPackage}
			<!-- Detail View -->
			<div class="package-detail">
				<button class="back-button" on:click={closeDetail}>
					<Icon icon="mdi:arrow-left" class="w-5 h-5" />
					<span>{$t.common.back}</span>
				</button>

				<div class="detail-header">
					<div class="detail-icon {selectedPackage.iconBg}">
						<Icon icon={selectedPackage.icon} class="w-16 h-16 text-white" />
					</div>
					<div class="detail-info">
						<h1>{selectedPackage.name}</h1>
						<p class="detail-meta">
							{$t.appCenter.version}: {selectedPackage.version} · {selectedPackage.size}
						</p>
						<span class="status-badge {getStatusColor(selectedPackage.status)}">
							{getStatusLabel(selectedPackage.status)}
						</span>
					</div>
					<div class="detail-actions">
						{#if selectedPackage.status === 'not_installed'}
							<button class="btn-primary" on:click={() => handleInstall(selectedPackage)}>
								<Icon icon="mdi:download" class="w-5 h-5" />
								{$t.appCenter.actions.install}
							</button>
						{:else if selectedPackage.status === 'installed'}
							<button class="btn-secondary" on:click={() => handleOpenApp(selectedPackage)}>
								<Icon icon="mdi:open-in-new" class="w-5 h-5" />
								{$t.appCenter.actions.open}
							</button>
							<button class="btn-danger" on:click={() => handleUninstall(selectedPackage)}>
								<Icon icon="mdi:delete" class="w-5 h-5" />
								{$t.appCenter.actions.uninstall}
							</button>
						{:else if selectedPackage.status === 'installing'}
							<button class="btn-primary" disabled>
								<Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
								{$t.appCenter.actions.installing}
							</button>
						{/if}
						{#if installError}
							<p class="error-message">{installError}</p>
						{/if}
					</div>
				</div>

				<div class="detail-description">
					<h2>{$t.appCenter.description}</h2>
					<p>{selectedPackage.description}</p>
				</div>

				<div class="detail-features">
					<h2>{$t.appCenter.features}</h2>
					<ul>
						<li><Icon icon="mdi:check" class="w-4 h-4 text-green-500" /> {$t.appCenter.packages.docker.feature1}</li>
						<li><Icon icon="mdi:check" class="w-4 h-4 text-green-500" /> {$t.appCenter.packages.docker.feature2}</li>
						<li><Icon icon="mdi:check" class="w-4 h-4 text-green-500" /> {$t.appCenter.packages.docker.feature3}</li>
					</ul>
				</div>
			</div>
		{:else}
			<!-- Grid View -->
			<div class="package-grid">
				{#if filteredPackages.length === 0}
					<div class="empty-state">
						<Icon icon="mdi:package-variant" class="w-16 h-16 text-slate-300" />
						<p>{$t.appCenter.noPackages}</p>
					</div>
				{:else}
					{#each filteredPackages as pkg}
						<button class="package-card" on:click={() => selectPackage(pkg)}>
							<div class="package-icon {pkg.iconBg}">
								<Icon icon={pkg.icon} class="w-10 h-10 text-white" />
							</div>
							<div class="package-info">
								<h3>{pkg.name}</h3>
								<p>{pkg.description}</p>
								<div class="package-meta">
									<span class="version">{pkg.version}</span>
									<span class="status-dot {pkg.status === 'installed' ? 'installed' : ''}"></span>
								</div>
							</div>
							<Icon icon="mdi:chevron-right" class="chevron" />
						</button>
					{/each}
				{/if}
			</div>
		{/if}
	</main>
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

	.sidebar-footer {
		padding: 16px 20px;
		border-top: 1px solid #e2e8f0;
	}

	.stats {
		display: flex;
		flex-direction: column;
		align-items: center;
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
		padding: 16px 24px;
		background: white;
		border-bottom: 1px solid #e2e8f0;
	}

	.search-box {
		position: relative;
		max-width: 400px;
	}

	.search-box input {
		width: 100%;
		padding: 10px 10px 10px 40px;
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

	.back-button {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		background: white;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		color: #64748b;
		font-size: 14px;
		cursor: pointer;
		margin-bottom: 20px;
	}

	.back-button:hover {
		background: #f8fafc;
		color: #334155;
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

	.status-badge {
		display: inline-block;
		padding: 4px 10px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 500;
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
	.detail-features {
		background: white;
		border-radius: 12px;
		padding: 20px;
		margin-bottom: 16px;
	}

	.detail-description h2,
	.detail-features h2 {
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
</style>
