<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { api, type PowerStatus, type ScheduledTask } from '$stores/api';
	import { systemInfo } from '$stores/system';
	import { toasts, errorMessage } from '$stores/toasts';
	import { powerScreen } from '$stores/power';

	let status: PowerStatus | null = null;
	let schedules: ScheduledTask[] = [];
	let loading = true;
	let newKind: 'reboot' | 'shutdown' = 'reboot';
	let newTime = '03:00';
	let newDays: Record<string, boolean> = { mon: true, tue: true, wed: true, thu: true, fri: true, sat: true, sun: true };
	const DAYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;
	let confirm: 'reboot' | 'shutdown' | null = null;

	async function load() {
		loading = true;
		try {
			[status, schedules] = await Promise.all([api.getPowerStatus(), api.getPowerSchedules()]);
		} catch (e) {
			toasts.error(errorMessage(e, $t.common.errors.loadFailed));
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

<div class="hw-power">
	<section class="hw-section">
		<h3 class="hw-title"><Icon icon="mdi:chip" class="w-5 h-5" />{$t.hardwarePower.cpu}</h3>
		{#if loading}
			<p class="hw-hint">{$t.common.loading}</p>
		{:else if status}
			<div class="hw-grid">
				<div class="hw-stat"><span class="hw-label">{$t.hardwarePower.frequency}</span><span class="hw-value">{status.cpu_cur_mhz ?? '—'} MHz</span><span class="hw-sub">{status.cpu_min_mhz ?? '—'} – {status.cpu_max_mhz ?? '—'} MHz</span></div>
				<div class="hw-stat"><span class="hw-label">{$t.hardwarePower.temperature}</span><span class="hw-value">{$systemInfo?.cpu.temperature != null ? `${Math.round($systemInfo.cpu.temperature)} °C` : '—'}</span></div>
				<div class="hw-stat"><span class="hw-label">{$t.hardwarePower.load}</span><span class="hw-value">{$systemInfo ? `${$systemInfo.cpu.usage.toFixed(0)} %` : '—'}</span></div>
			</div>
			<div class="hw-row">
				<label class="hw-label" for="governor">{$t.hardwarePower.governor}</label>
				<select id="governor" class="hw-select" value={status.governor ?? ''} on:change={(e) => changeGovernor((e.currentTarget as HTMLSelectElement).value)} disabled={status.available_governors.length === 0}>
					{#each status.available_governors as g}
						<option value={g}>{g}</option>
					{/each}
				</select>
			</div>
			<p class="hw-hint">{status.available_governors.length === 0 ? $t.hardwarePower.governorUnavailable : $t.hardwarePower.governorHint}</p>
			{#if status.dev_mode}<p class="hw-hint">{$t.timeLanguage.time.devMode}</p>{/if}
		{/if}
	</section>

	<section class="hw-section">
		<h3 class="hw-title"><Icon icon="mdi:calendar-clock" class="w-5 h-5" />{$t.hardwarePower.schedules}</h3>
		<p class="hw-hint">{$t.hardwarePower.schedulesHint}</p>
		{#if schedules.length === 0}
			<p class="hw-hint">{$t.hardwarePower.noSchedules}</p>
		{/if}
		{#each schedules as task (task.id)}
			<div class="hw-task" class:disabled={!task.enabled}>
				<Icon icon={task.kind === 'reboot' ? 'mdi:restart' : 'mdi:power'} class="w-5 h-5" />
				<span class="hw-task-main"><strong>{task.kind === 'reboot' ? $t.hardwarePower.reboot : $t.hardwarePower.shutdown}</strong> · {task.time}</span>
				<span class="hw-task-days">{task.days.split(',').map(dayLabel).join(' ')}</span>
				<label class="toggle"><input type="checkbox" checked={task.enabled} on:change={() => toggleSchedule(task)} /><span class="toggle-slider"></span></label>
				<button class="hw-icon-btn danger" on:click={() => removeSchedule(task)} title={$t.common.delete}><Icon icon="mdi:delete-outline" class="w-4 h-4" /></button>
			</div>
		{/each}
		<div class="hw-new">
			<select class="hw-select" bind:value={newKind}>
				<option value="reboot">{$t.hardwarePower.reboot}</option>
				<option value="shutdown">{$t.hardwarePower.shutdown}</option>
			</select>
			<input type="time" class="hw-input" bind:value={newTime} />
			<div class="hw-days">
				{#each DAYS as d}
					<label class="hw-day" class:on={newDays[d]}><input type="checkbox" bind:checked={newDays[d]} />{dayLabel(d)}</label>
				{/each}
			</div>
			<button class="btn-primary" on:click={addSchedule} disabled={!DAYS.some((d) => newDays[d])}>{$t.common.add}</button>
		</div>
	</section>

	<section class="hw-section">
		<h3 class="hw-title"><Icon icon="mdi:power-settings" class="w-5 h-5" />{$t.hardwarePower.actions}</h3>
		<div class="hw-actions">
			<button class="btn-secondary" on:click={() => (confirm = 'reboot')}><Icon icon="mdi:restart" class="w-4 h-4" />{$t.hardwarePower.rebootNow}</button>
			<button class="btn-danger" on:click={() => (confirm = 'shutdown')}><Icon icon="mdi:power" class="w-4 h-4" />{$t.hardwarePower.shutdownNow}</button>
		</div>
		<p class="hw-hint">{$t.hardwarePower.diskPowerHint}</p>
	</section>
</div>

{#if confirm}
	<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
	<div class="hw-overlay" on:click={() => (confirm = null)}>
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div class="hw-dialog" on:click|stopPropagation>
			<p>{confirm === 'reboot' ? $t.topBar.userMenu.confirmRestart : $t.topBar.userMenu.confirmShutdown}</p>
			<div class="hw-actions">
				<button class="btn-secondary" on:click={() => (confirm = null)}>{$t.common.cancel}</button>
				<button class="btn-danger" on:click={() => confirm && powerAction(confirm)}>{$t.common.confirm}</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.hw-power { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem; overflow-y: auto; height: 100%; }
	.hw-section { background: white; border: 1px solid #e2e8f0; border-radius: 0.75rem; padding: 1rem 1.25rem; }
	.hw-title { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9375rem; font-weight: 600; color: #0f172a; margin: 0 0 0.75rem; }
	.hw-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr)); gap: 0.75rem; margin-bottom: 1rem; }
	.hw-stat { display: flex; flex-direction: column; gap: 0.125rem; padding: 0.75rem; background: #f8fafc; border-radius: 0.5rem; }
	.hw-label { font-size: 0.75rem; color: #64748b; }
	.hw-value { font-size: 1.125rem; font-weight: 600; color: #0f172a; }
	.hw-sub { font-size: 0.6875rem; color: #94a3b8; }
	.hw-row { display: flex; align-items: center; gap: 1rem; }
	.hw-select, .hw-input { border: 1px solid #cbd5e1; border-radius: 0.5rem; padding: 0.375rem 0.625rem; font-size: 0.8125rem; background: white; }
	.hw-hint { font-size: 0.75rem; color: #64748b; margin: 0.5rem 0 0; }
	.hw-task { display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0; border-top: 1px solid #f1f5f9; font-size: 0.8125rem; }
	.hw-task.disabled { opacity: 0.55; }
	.hw-task-main { flex: 1; }
	.hw-task-days { color: #64748b; font-size: 0.75rem; }
	.hw-new { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px dashed #e2e8f0; }
	.hw-days { display: flex; gap: 0.25rem; }
	.hw-day { font-size: 0.6875rem; padding: 0.25rem 0.4rem; border-radius: 0.375rem; background: #f1f5f9; color: #64748b; cursor: pointer; }
	.hw-day input { display: none; }
	.hw-day.on { background: #dbeafe; color: #1d4ed8; }
	.hw-actions { display: flex; gap: 0.5rem; }
	.hw-icon-btn { padding: 0.25rem; border-radius: 0.375rem; color: #64748b; }
	.hw-icon-btn.danger:hover { color: #dc2626; background: #fef2f2; }
	.btn-primary, .btn-secondary, .btn-danger { display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.4rem 0.875rem; border-radius: 0.5rem; font-size: 0.8125rem; font-weight: 500; }
	.btn-primary { background: #3b82f6; color: white; }
	.btn-primary:disabled { opacity: 0.5; }
	.btn-secondary { background: #f1f5f9; color: #0f172a; }
	.btn-danger { background: #dc2626; color: white; }
	.toggle { position: relative; width: 2.25rem; height: 1.25rem; display: inline-block; }
	.toggle input { opacity: 0; width: 0; height: 0; }
	.toggle-slider { position: absolute; inset: 0; background: #cbd5e1; border-radius: 999px; transition: 0.2s; }
	.toggle-slider::before { content: ''; position: absolute; width: 1rem; height: 1rem; left: 0.125rem; top: 0.125rem; background: white; border-radius: 50%; transition: 0.2s; }
	.toggle input:checked + .toggle-slider { background: #3b82f6; }
	.toggle input:checked + .toggle-slider::before { transform: translateX(1rem); }
	.hw-overlay { position: fixed; inset: 0; background: rgb(15 23 42 / 0.5); display: flex; align-items: center; justify-content: center; z-index: 8000; }
	.hw-dialog { background: white; border-radius: 0.75rem; padding: 1.25rem 1.5rem; display: flex; flex-direction: column; gap: 1rem; min-width: 20rem; font-size: 0.875rem; }
</style>
