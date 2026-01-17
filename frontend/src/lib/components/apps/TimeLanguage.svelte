<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t, locale, languages, type Locale } from '$lib/i18n';

	let activeTab: 'time' | 'language' = 'language';

	// Time settings
	let timezone = 'Europe/Paris';
	let dateFormat = 'DD/MM/YYYY';
	let timeFormat = '24h';
	let ntpEnabled = true;
	let ntpServer = 'pool.ntp.org';

	// Common timezones
	const timezones = [
		{ value: 'Europe/Paris', label: 'Europe/Paris (UTC+1)' },
		{ value: 'Europe/London', label: 'Europe/London (UTC+0)' },
		{ value: 'America/New_York', label: 'America/New_York (UTC-5)' },
		{ value: 'America/Los_Angeles', label: 'America/Los_Angeles (UTC-8)' },
		{ value: 'Asia/Tokyo', label: 'Asia/Tokyo (UTC+9)' },
		{ value: 'Asia/Shanghai', label: 'Asia/Shanghai (UTC+8)' },
		{ value: 'Australia/Sydney', label: 'Australia/Sydney (UTC+11)' }
	];

	const dateFormats = [
		{ value: 'DD/MM/YYYY', label: 'DD/MM/YYYY (31/12/2025)' },
		{ value: 'MM/DD/YYYY', label: 'MM/DD/YYYY (12/31/2025)' },
		{ value: 'YYYY-MM-DD', label: 'YYYY-MM-DD (2025-12-31)' }
	];

	function handleLanguageChange(lang: Locale) {
		locale.setLocale(lang);
	}

	function getCurrentTime() {
		return new Date().toLocaleString($locale === 'fr' ? 'fr-FR' : 'en-US', {
			timeZone: timezone,
			hour12: timeFormat === '12h'
		});
	}

	$: currentTime = getCurrentTime();

	// Update time every second
	let interval: ReturnType<typeof setInterval>;
	import { onMount, onDestroy } from 'svelte';

	onMount(() => {
		interval = setInterval(() => {
			currentTime = getCurrentTime();
		}, 1000);
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});
</script>

<div class="time-language">
	<!-- Tabs -->
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
			<!-- Time Tab -->
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
				</div>

				<div class="setting-row">
					<label class="setting-label">{$t.timeLanguage.time.timezone}</label>
					<select bind:value={timezone} class="setting-select">
						{#each timezones as tz}
							<option value={tz.value}>{tz.label}</option>
						{/each}
					</select>
				</div>

				<div class="setting-row">
					<label class="setting-label">{$t.timeLanguage.time.dateFormat}</label>
					<select bind:value={dateFormat} class="setting-select">
						{#each dateFormats as df}
							<option value={df.value}>{df.label}</option>
						{/each}
					</select>
				</div>

				<div class="setting-row">
					<label class="setting-label">{$t.timeLanguage.time.timeFormat}</label>
					<div class="radio-group">
						<label class="radio-label">
							<input type="radio" bind:group={timeFormat} value="24h" />
							<span>{$t.timeLanguage.time.format24h}</span>
						</label>
						<label class="radio-label">
							<input type="radio" bind:group={timeFormat} value="12h" />
							<span>{$t.timeLanguage.time.format12h}</span>
						</label>
					</div>
				</div>

				<div class="setting-divider"></div>

				<div class="setting-row">
					<div class="setting-info">
						<label class="setting-label">{$t.timeLanguage.time.syncWithNtp}</label>
					</div>
					<label class="toggle">
						<input type="checkbox" bind:checked={ntpEnabled} />
						<span class="toggle-slider"></span>
					</label>
				</div>

				{#if ntpEnabled}
					<div class="setting-row">
						<label class="setting-label">{$t.timeLanguage.time.ntpServer}</label>
						<input type="text" bind:value={ntpServer} class="setting-input" />
					</div>

					<div class="setting-row">
						<button class="btn-secondary">
							<Icon icon="mdi:sync" class="w-4 h-4" />
							{$t.timeLanguage.time.syncNow}
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<!-- Language Tab -->
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

	<!-- Apply Button -->
	<div class="actions-bar">
		<button class="btn-primary">
			{$t.common.apply}
		</button>
	</div>
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
</style>
