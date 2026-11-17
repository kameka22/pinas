<script lang="ts">
	import Icon from '@iconify/svelte';
	let {
		title,
		description = '',
		icon,
	}: { title: string; description?: string; icon: string } = $props();
</script>

<header class="settings-heading">
	<span class="settings-heading-icon"><Icon {icon} width="24" /></span>
	<div>
		<h2>{title}</h2>
		{#if description}<p>{description}</p>{/if}
	</div>
</header>
