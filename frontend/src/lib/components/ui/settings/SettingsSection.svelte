<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from '@iconify/svelte';
	let {
		title,
		description = '',
		icon = '',
		actions,
		children,
	}: {
		title: string;
		description?: string;
		icon?: string;
		actions?: Snippet;
		children: Snippet;
	} = $props();
</script>

<section class="settings-section-card">
	<div class="settings-section-head">
		<div>
			<h3>{#if icon}<Icon {icon} width="20" />{/if}{title}</h3>
			{#if description}<p class="settings-help">{description}</p>{/if}
		</div>
		{#if actions}<div class="settings-row-control">{@render actions()}</div>{/if}
	</div>
	{@render children()}
</section>
