<script lang="ts">
	let {
		label,
		items,
		value,
		onchange,
	}: {
		label: string;
		items: { value: string; label: string }[];
		value: string;
		onchange: (value: string) => void;
	} = $props();
</script>

<nav class="settings-tabs" aria-label={label}>
	{#each items as item (item.value)}
		<button
			type="button"
			class="settings-tab"
			aria-pressed={value === item.value}
			onclick={() => onchange(item.value)}>{item.label}</button
		>
	{/each}
</nav>
