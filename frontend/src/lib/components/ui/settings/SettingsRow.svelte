<script lang="ts">
	import type { Snippet } from 'svelte';
	let {
		label,
		hint = '',
		forId = '',
		children,
	}: { label: string; hint?: string; forId?: string; children?: Snippet } = $props();
</script>

<div class="settings-row">
	<div class="settings-row-label">
		{#if forId}<label for={forId}>{label}</label>{:else}<span>{label}</span>{/if}
		{#if hint}<p class="settings-help">{hint}</p>{/if}
	</div>
	{#if children}<div class="settings-row-control">{@render children()}</div>{/if}
</div>
