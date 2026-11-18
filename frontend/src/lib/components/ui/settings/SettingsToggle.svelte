<script lang="ts">
	let {
		checked = $bindable(false),
		label,
		disabled = false,
		onchange,
	}: {
		checked?: boolean;
		label: string;
		disabled?: boolean;
		onchange?: (checked: boolean) => void;
	} = $props();

	// With `onchange`, the parent owns the state (typically after a server call):
	// the switch snaps back to `checked` and only moves when the parent updates it,
	// so a failed action never leaves it showing the wrong state.
	function handleChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const next = input.checked;
		if (onchange) {
			input.checked = checked;
			onchange(next);
		} else {
			checked = next;
		}
	}
</script>

<span class="settings-toggle">
	<input
		type="checkbox"
		role="switch"
		aria-label={label}
		{checked}
		{disabled}
		onchange={handleChange}
	/>
	<span></span>
</span>
