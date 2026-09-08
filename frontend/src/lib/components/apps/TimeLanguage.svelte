<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { t, locale, languages, type Locale } from '$lib/i18n';
	import { api, auth, type TimeStatus } from '$stores/api';
	import { toasts, errorMessage } from '$stores/toasts';

	let activeTab: 'time' | 'language' = 'time';

	// Server state
	let status: TimeStatus | null = null;
	let zones: string[] = [];
	let loading = true;
	let saving = false;
	let syncing = false;

	// Editable copy
	let timezone = 'UTC';
	let ntpEnabled = true;
	let ntpServers = '';
	let zoneFilter = '';

	// Display preferences (per user)
	let dateFormat = 'DD/MM/YYYY';
	let timeFormat = '24h';
	const dateFormats = [
		{ value: 'DD/MM/YYYY', label: 'DD/MM/YYYY (31/12/2026)' },
		{ value: 'MM/DD/YYYY', label: 'MM/DD/YYYY (12/31/2026)' },
		{ value: 'YYYY-MM-DD', label: 'YYYY-MM-DD (2026-12-31)' }
	];

	$: isAdmin = $auth.user?.role === 'admin';
	$: dirty = status !== null && (timezone !== status.timezone || ntpEnabled !== status.ntp_enabled || ntpServers.trim() !== status.ntp_servers.join(' '));
	$: filteredZones = zoneFilter ? zones.filter((z) => z.toLowerCase().includes(zoneFilter.toLowerCase())).slice(0, 200) : zones;

	function applyStatus(s: TimeStatus) {
		status = s;
		timezone = s.timezone;
		ntpEnabled = s.ntp_enabled;
		ntpServers = s.ntp_servers.join(' ');
	}

	async function load() {
		loading = true;
		try {
			const [s, z] = await Promise.all([api.getTimeStatus(), api.getTimezones()]);
			applyStatus(s);
			zones = z;
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
		}
		try {
			const [df, tf] = await Promise.allSettled([api.getPreference('ui.dateFormat'), api.getPreference('ui.timeFormat')]);
			if (df.status === 'fulfilled' && df.value?.value) dateFormat = df.value.value;
			if (tf.status === 'fulfilled' && tf.value?.value) timeFormat = tf.value.value;
		} catch {
			// no preferences yet
		}
		loading = false;
	}

	async function apply() {
		saving = true;
		try {
			const servers = ntpServers.split(/[\s,]+/).filter(Boolean);
			applyStatus(await api.updateTime({ timezone, ntp_enabled: ntpEnabled, ntp_servers: servers }));
			toasts.success($t.timeLanguage.time.saved);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		} finally {
			saving = false;
		}
	}

	async function syncNow() {
		syncing = true;
		try {
			applyStatus(await api.syncTime());
			toasts.success($t.timeLanguage.time.syncRequested);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		} finally {
			syncing = false;
		}
	}

	function savePreference(key: string, value: string) {
		api.setPreference(key, value).catch(() => {});
	}

	function handleLanguageChange(lang: Locale) {
		locale.setLocale(lang);
		savePreference('ui.locale', lang);
	}

	function getCurrentTime() {
		try {
			return new Date().toLocaleString($locale === 'fr' ? 'fr-FR' : 'en-US', {
				timeZone: timezone,
				hour12: timeFormat === '12h'
			});
		} catch {
			return new Date().toLocaleString();
		}
	}

	$: currentTime = getCurrentTime();
	let interval: ReturnType<typeof setInterval>;

	onMount(() => {
		load();
		interval = setInterval(() => { currentTime = getCurrentTime(); }, 1000);
	});
	onDestroy(() => { if (interval) clearInterval(interval); });
</script>

<div class="time-language">
	<div class="tabs-header">
		<button class="tab" class:active={activeTab === 'time'} on:click={() => (activeTab = 'time')}>
			<Icon icon="mdi:clock-outline" class="w-4 h-4" />
			{$t.timeLanguage.tabs.time}
		</button>
		<button class="tab" class:active={activeTab === 'language'} on:click={() => (activeTab = 'language')}>
			<Icon icon="mdi:translate" class="w-4 h-4" />
			{$t.timeLanguage.tabs.language}
		</button>
	</div>

	<div class="tab-content">
		{#if activeTab === 'time'}
			<div class="settings-section">
				<h3 class="section-title">
					<Icon icon="mdi:clock-outline" class="w-5 h-5" />
					{$t.timeLanguage.time.title}
				</h3>

				<div class="setting-card">
					<div class="current-time">
						<span class="time-label">{$t.timeLanguage.time.currentTime}</span>
						<span class="time-value">{currentTime}</span>
					</div>
					{#if status}
						<div class="ntp-state" class:synced={status.ntp_synchronized}>
							<Icon icon={status.ntp_synchronized ? 'mdi:check-circle' : 'mdi:clock-alert-outline'} class="w-4 h-4" />
							{status.ntp_synchronized ? $t.timeLanguage.time.synced : $t.timeLanguage.time.notSynced}
						</div>
					{/if}
				</div>

				{#if loading}
					<p class="section-description">{$t.common.loading}</p>
				{:else}
					<div class="setting-row">
						<label class="setting-label" for="tz-filter">{$t.timeLanguage.time.timezone}</label>
						<div class="tz-picker">
							<input id="tz-filter" type="text" class="setting-input" placeholder={$t.timeLanguage.time.selectTimezone} bind:value={zoneFilter} disabled={!isAdmin} />
							<select bind:value={timezone} class="setting-select" size="1" disabled={!isAdmin}>
								{#if !filteredZones.includes(timezone)}<option value={timezone}>{timezone}</option>{/if}
								{#each filteredZones as tz}
									<option value={tz}>{tz}</option>
								{/each}
							</select>
						</div>
					</div>

					<div class="setting-row">
						<label class="setting-label" for="date-format">{$t.timeLanguage.time.dateFormat}</label>
						<select id="date-format" bind:value={dateFormat} class="setting-select" on:change={() => savePreference('ui.dateFormat', dateFormat)}>
							{#each dateFormats as df}
								<option value={df.value}>{df.label}</option>
							{/each}
						</select>
					</div>

					<div class="setting-row">
						<span class="setting-label">{$t.timeLanguage.time.timeFormat}</span>
						<div class="radio-group">
							<label class="radio-label">
								<input type="radio" bind:group={timeFormat} value="24h" on:change={() => savePreference('ui.timeFormat', '24h')} />
								<span>{$t.timeLanguage.time.format24h}</span>
							</label>
							<label class="radio-label">
								<input type="radio" bind:group={timeFormat} value="12h" on:change={() => savePreference('ui.timeFormat', '12h')} />
								<span>{$t.timeLanguage.time.format12h}</span>
							</label>
						</div>
					</div>

					<div class="setting-divider"></div>

					<div class="setting-row">
						<div class="setting-info">
							<span class="setting-label">{$t.timeLanguage.time.syncWithNtp}</span>
						</div>
						<label class="toggle">
							<input type="checkbox" bind:checked={ntpEnabled} disabled={!isAdmin} />
							<span class="toggle-slider"></span>
						</label>
					</div>

					{#if ntpEnabled}
						<div class="setting-row">
							<label class="setting-label" for="ntp-servers">{$t.timeLanguage.time.ntpServer}</label>
							<input id="ntp-servers" type="text" bind:value={ntpServers} class="setting-input" placeholder="0.pool.ntp.org 1.pool.ntp.org" disabled={!isAdmin} />
						</div>
						<div class="setting-row">
							<span class="setting-hint">{$t.timeLanguage.time.serversHint}</span>
							<button class="btn-secondary" on:click={syncNow} disabled={!isAdmin || syncing || dirty}>
								<Icon icon="mdi:sync" class="w-4 h-4 {syncing ? 'animate-spin' : ''}" />
								{$t.timeLanguage.time.syncNow}
							</button>
						</div>
					{/if}
				{/if}
			</div>
		{:else}
			<div class="settings-section">
				<h3 class="section-title">
					<Icon icon="mdi:translate" class="w-5 h-5" />
					{$t.timeLanguage.language.title}
				</h3>

				<p class="section-description">{$t.timeLanguage.language.restart}</p>

				<div class="language-grid">
					{#each languages as lang}
						<button
							class="language-card"
							class:selected={$locale === lang.code}
							on:click={() => handleLanguageChange(lang.code)}
						>
							<span class="language-flag">{lang.flag}</span>
							<div class="language-info">
								<span class="language-native">{lang.nativeName}</span>
								<span class="language-name">{lang.name}</span>
							</div>
							{#if $locale === lang.code}
								<Icon icon="mdi:check-circle" class="w-5 h-5 text-blue-500 check-icon" />
							{/if}
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	{#if activeTab === 'time' && isAdmin}
		<div class="actions-bar">
			{#if status?.dev_mode}<span class="setting-hint">{$t.timeLanguage.time.devMode}</span>{/if}
			<button class="btn-primary" on:click={apply} disabled={!dirty || saving}>
				{saving ? $t.common.loading : $t.common.apply}
			</button>
		</div>
	{/if}
</div>

<style>
	.time-language {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	.tabs-header {
		display: flex;
		gap: 0;
		border-bottom: 1px solid #e5e7eb;
		padding: 0 24px;
		background: #fafafa;
	}

	.tab {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px 24px;
		font-size: 14px;
		color: #6b7280;
		border-bottom: 2px solid transparent;
		transition: all 0.15s ease;
		background: transparent;
		border-top: none;
		border-left: none;
		border-right: none;
	}

	.tab:hover {
		color: #374151;
	}

	.tab.active {
		color: #2563eb;
		border-bottom-color: #2563eb;
		background: white;
	}

	.tab-content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.settings-section {
		max-width: 600px;
	}

	.section-title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 16px;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 8px;
	}

	.section-description {
		font-size: 13px;
		color: #6b7280;
		margin-bottom: 24px;
	}

	.setting-card {
		background: #f9fafb;
		border-radius: 12px;
		padding: 20px;
		margin-bottom: 24px;
	}

	.current-time {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.time-label {
		font-size: 12px;
		color: #6b7280;
	}

	.time-value {
		font-size: 24px;
		font-weight: 600;
		color: #1f2937;
		font-variant-numeric: tabular-nums;
	}

	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 0;
	}

	.setting-label {
		font-size: 14px;
		color: #374151;
	}

	.setting-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.setting-select {
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		background: white;
		min-width: 250px;
	}

	.setting-select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.setting-input {
		padding: 8px 12px;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		font-size: 14px;
		color: #374151;
		min-width: 250px;
	}

	.setting-input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

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

	.radio-label input[type='radio'] {
		width: 16px;
		height: 16px;
		accent-color: #3b82f6;
	}

	.setting-divider {
		height: 1px;
		background: #e5e7eb;
		margin: 16px 0;
	}

	/* Toggle */
	.toggle {
		position: relative;
		display: inline-block;
		width: 44px;
		height: 24px;
		cursor: pointer;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		inset: 0;
		background: #d1d5db;
		border-radius: 24px;
		transition: all 0.2s ease;
	}

	.toggle-slider::before {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: all 0.2s ease;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.toggle input:checked + .toggle-slider {
		background: #3b82f6;
	}

	.toggle input:checked + .toggle-slider::before {
		transform: translateX(20px);
	}

	/* Language Grid */
	.language-grid {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.language-card {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px 20px;
		background: #f9fafb;
		border: 2px solid transparent;
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
		text-align: left;
	}

	.language-card:hover {
		background: #f3f4f6;
	}

	.language-card.selected {
		background: #eff6ff;
		border-color: #3b82f6;
	}

	.language-flag {
		font-size: 32px;
		line-height: 1;
	}

	.language-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}

	.language-native {
		font-size: 15px;
		font-weight: 500;
		color: #1f2937;
	}

	.language-name {
		font-size: 13px;
		color: #6b7280;
	}

	.check-icon {
		flex-shrink: 0;
	}

	/* Buttons */
	.btn-primary {
		padding: 10px 24px;
		background: #3b82f6;
		color: white;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		transition: all 0.15s ease;
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 8px;
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
		border-color: #d1d5db;
	}

	.actions-bar {
		padding: 16px 24px;
		border-top: 1px solid #e5e7eb;
		display: flex;
		justify-content: flex-end;
		background: #fafafa;
	}

	.tz-picker { display: flex; gap: 0.5rem; flex: 1; min-width: 0; }
	.tz-picker .setting-input { flex: 1; min-width: 8rem; }
	.tz-picker .setting-select { flex: 1.4; min-width: 10rem; }
	.ntp-state { display: inline-flex; align-items: center; gap: 0.375rem; margin-top: 0.5rem; font-size: 0.75rem; color: #b45309; }
	.ntp-state.synced { color: #15803d; }
	.setting-hint { font-size: 0.75rem; color: #64748b; }
	.actions-bar { display: flex; justify-content: flex-end; align-items: center; gap: 1rem; padding: 0.75rem 1.5rem; border-top: 1px solid #e2e8f0; }
</style>
