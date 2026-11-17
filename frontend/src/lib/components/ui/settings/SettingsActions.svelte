<script lang="ts">
	import { t } from '#lib/i18n/index.js';
	let {
		dirty,
		saving = false,
		disabled = false,
		hint = '',
		onsave,
		oncancel,
	}: {
		dirty: boolean;
		saving?: boolean;
		disabled?: boolean;
		hint?: string;
		onsave: () => void;
		oncancel: () => void;
	} = $props();
</script>

<div class="settings-actions" aria-busy={saving}>
	<p class="settings-help" role="status">
		{dirty ? $t.settingsUi.unsaved : hint}
	</p>
	<div class="settings-actions-buttons">
		<button
			type="button"
			class="settings-button"
			onclick={oncancel}
			disabled={!dirty || saving || disabled}>{$t.common.cancel}</button
		>
		<button
			type="button"
			class="settings-button primary"
			onclick={onsave}
			disabled={!dirty || saving || disabled}
		>
			{saving ? $t.common.loading : $t.common.save}
		</button>
	</div>
</div>
