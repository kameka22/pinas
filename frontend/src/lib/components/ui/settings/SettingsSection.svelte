<script lang="ts">
	import type { Snippet } from 'svelte';
	let {
		title,
		description = '',
		children,
	}: { title: string; description?: string; children: Snippet } = $props();
</script>

<section class="settings-section-card">
	<h3>{title}</h3>
	{#if description}<p class="settings-help">{description}</p>{/if}
	{@render children()}
</section>
