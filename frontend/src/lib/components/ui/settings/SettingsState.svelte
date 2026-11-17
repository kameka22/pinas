<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '#lib/i18n/index.js';
	let {
		kind,
		message,
		onretry,
	}: {
		kind: 'loading' | 'error' | 'empty';
		message: string;
		onretry?: () => void;
	} = $props();
</script>

<div
	class="settings-state"
	class:error={kind === 'error'}
	role={kind === 'error' ? 'alert' : 'status'}
>
	<Icon
		icon={kind === 'error'
			? 'mdi:alert-circle-outline'
			: kind === 'loading'
				? 'mdi:progress-clock'
				: 'mdi:information-outline'}
		width="24"
	/>
	<p>{message}</p>
	{#if onretry}<button type="button" class="settings-button" onclick={onretry}
			>{$t.common.retry}</button
		>{/if}
</div>
