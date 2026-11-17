<script lang="ts">
	import Icon from '@iconify/svelte';
	import ConfirmModal from '#lib/components/ui/ConfirmModal.svelte';
	import SettingsHeader from '#lib/components/ui/settings/SettingsHeader.svelte';
	import SettingsSection from '#lib/components/ui/settings/SettingsSection.svelte';
	import SettingsRow from '#lib/components/ui/settings/SettingsRow.svelte';
	import SettingsToggle from '#lib/components/ui/settings/SettingsToggle.svelte';
	import SettingsState from '#lib/components/ui/settings/SettingsState.svelte';
	import '#lib/components/ui/settings/settings.css';
	import { onMount } from 'svelte';
	import { t } from '#lib/i18n/index.js';
	import { api, type PowerStatus, type ScheduledTask } from '#lib/stores/api.js';
	import { systemInfo } from '#lib/stores/system.js';
	import { toasts, errorMessage } from '#lib/stores/toasts.js';
	import { powerScreen } from '#lib/stores/power.js';

	let status: PowerStatus | null = null;
	let schedules: ScheduledTask[] = [];
	let loading = true;
	let loadError = '';
	let newKind: 'reboot' | 'shutdown' = 'reboot';
	let newTime = '03:00';
	let newDays: Record<string, boolean> = { mon: true, tue: true, wed: true, thu: true, fri: true, sat: true, sun: true };
	const DAYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;
	let confirm: 'reboot' | 'shutdown' | null = null;

	async function load() {
		loading = true;
		loadError = '';
		try {
			[status, schedules] = await Promise.all([api.getPowerStatus(), api.getPowerSchedules()]);
		} catch (e) {
			loadError = errorMessage(e, $t.common.errors.loadFailed);
		}
		loading = false;
	}

	async function changeGovernor(governor: string) {
		try {
			status = await api.setCpuGovernor(governor);
			toasts.success($t.hardwarePower.governorApplied);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		}
	}

	async function addSchedule() {
		const days = DAYS.filter((d) => newDays[d]);
		try {
			await api.createPowerSchedule({ kind: newKind, time: newTime, days });
			schedules = await api.getPowerSchedules();
			toasts.success($t.hardwarePower.scheduleAdded);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		}
	}

	async function toggleSchedule(task: ScheduledTask) {
		try {
			await api.togglePowerSchedule(task.id, !task.enabled);
			schedules = schedules.map((s) => (s.id === task.id ? { ...s, enabled: !s.enabled } : s));
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.saveFailed));
		}
	}

	async function removeSchedule(task: ScheduledTask) {
		try {
			await api.deletePowerSchedule(task.id);
			schedules = schedules.filter((s) => s.id !== task.id);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	async function powerAction(kind: 'reboot' | 'shutdown') {
		confirm = null;
		try {
			if (kind === 'reboot') await api.rebootSystem(); else await api.shutdownSystem();
			powerScreen.set({ active: true, action: kind === 'reboot' ? 'restart' : 'shutdown' });
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.generic));
		}
	}

	function dayLabel(d: string): string {
		return ($t.hardwarePower.days as Record<string, string>)[d] || d;
	}

	onMount(load);
</script>

<div class="settings-page">
	<SettingsHeader title={$t.controlPanel.items.hardwarePower} description={$t.settingsUi.descriptions.hardware} icon="mdi:chip" />

	{#if loading && !status}
		<SettingsState kind="loading" message={$t.common.loading} />
	{:else if loadError && !status}
		<SettingsState kind="error" message={loadError} onretry={load} />
	{:else if status}
		<SettingsSection title={$t.hardwarePower.cpu} icon="mdi:chip">
			<div class="settings-stats">
				<div class="settings-stat"><span>{$t.hardwarePower.frequency}</span><strong>{status.cpu_cur_mhz ?? '—'} MHz</strong><span class="settings-help">{status.cpu_min_mhz ?? '—'} – {status.cpu_max_mhz ?? '—'} MHz</span></div>
				<div class="settings-stat"><span>{$t.hardwarePower.temperature}</span><strong>{$systemInfo?.cpu.temperature != null ? `${Math.round($systemInfo.cpu.temperature)} °C` : '—'}</strong></div>
				<div class="settings-stat"><span>{$t.hardwarePower.load}</span><strong>{$systemInfo ? `${$systemInfo.cpu.usage.toFixed(0)} %` : '—'}</strong></div>
			</div>
			<SettingsRow label={$t.hardwarePower.governor} hint={status.available_governors.length === 0 ? $t.hardwarePower.governorUnavailable : $t.hardwarePower.governorHint} forId="governor">
				<select id="governor" value={status.governor ?? ''} on:change={(e) => changeGovernor((e.currentTarget as HTMLSelectElement).value)} disabled={status.available_governors.length === 0}>
					{#each status.available_governors as g}
						<option value={g}>{g}</option>
					{/each}
				</select>
			</SettingsRow>
			{#if status.dev_mode}<p class="settings-help">{$t.timeLanguage.time.devMode}</p>{/if}
		</SettingsSection>

		<SettingsSection title={$t.hardwarePower.schedules} description={$t.hardwarePower.schedulesHint} icon="mdi:calendar-clock">
			{#if schedules.length === 0}
				<SettingsState kind="empty" message={$t.hardwarePower.noSchedules} />
			{/if}
			{#each schedules as task (task.id)}
				<SettingsRow label={`${task.kind === 'reboot' ? $t.hardwarePower.reboot : $t.hardwarePower.shutdown} · ${task.time}`} hint={task.days.split(',').map(dayLabel).join(' ')}>
					<SettingsToggle label={task.kind === 'reboot' ? $t.hardwarePower.reboot : $t.hardwarePower.shutdown} checked={task.enabled} onchange={() => toggleSchedule(task)} />
					<button class="settings-button ghost danger-hover" on:click={() => removeSchedule(task)} title={$t.common.delete} aria-label={$t.common.delete}><Icon icon="mdi:delete-outline" width="18" /></button>
				</SettingsRow>
			{/each}
			<div class="hw-new">
				<select aria-label={$t.hardwarePower.schedules} bind:value={newKind}>
					<option value="reboot">{$t.hardwarePower.reboot}</option>
					<option value="shutdown">{$t.hardwarePower.shutdown}</option>
				</select>
				<input type="time" aria-label={$t.hardwarePower.schedules} bind:value={newTime} />
				<div class="hw-days">
					{#each DAYS as d}
						<label class="hw-day" class:on={newDays[d]}><input type="checkbox" bind:checked={newDays[d]} />{dayLabel(d)}</label>
					{/each}
				</div>
				<button class="settings-button primary" on:click={addSchedule} disabled={!DAYS.some((d) => newDays[d])}><Icon icon="mdi:plus" width="16" />{$t.common.add}</button>
			</div>
		</SettingsSection>

		<SettingsSection title={$t.hardwarePower.actions} description={$t.hardwarePower.diskPowerHint} icon="mdi:power-settings">
			<div class="settings-row-control">
				<button class="settings-button" on:click={() => (confirm = 'reboot')}><Icon icon="mdi:restart" width="16" />{$t.hardwarePower.rebootNow}</button>
				<button class="settings-button danger" on:click={() => (confirm = 'shutdown')}><Icon icon="mdi:power" width="16" />{$t.hardwarePower.shutdownNow}</button>
			</div>
		</SettingsSection>
	{/if}
</div>

<ConfirmModal
	visible={confirm !== null}
	title={confirm === 'shutdown' ? $t.hardwarePower.shutdownNow : $t.hardwarePower.rebootNow}
	message={confirm === 'reboot' ? $t.topBar.userMenu.confirmRestart : $t.topBar.userMenu.confirmShutdown}
	confirmLabel={$t.common.confirm}
	cancelLabel={$t.common.cancel}
	danger
	on:confirm={() => confirm && powerAction(confirm)}
	on:cancel={() => (confirm = null)}
/>

<style>
	.hw-new { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-top: 12px; padding-top: 16px; border-top: 1px dashed #e2e8f0; }
	.hw-days { display: flex; flex-wrap: wrap; gap: 4px; }
	.hw-day { font-size: 12px; padding: 6px 8px; border-radius: 6px; background: #f1f5f9; color: #475569; cursor: pointer; }
	.hw-day input { position: absolute; opacity: 0; width: 1px; height: 1px; }
	.hw-day:focus-within { outline: 2px solid #2563eb; outline-offset: 2px; }
	.hw-day.on { background: #dbeafe; color: #1d4ed8; }
</style>
