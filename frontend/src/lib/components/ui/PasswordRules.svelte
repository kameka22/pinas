<script lang="ts">
	import Icon from '@iconify/svelte';
	import { t } from '$lib/i18n';

	export let password = '';
	export let confirmPassword = '';
	export let showMatch = false;

	$: hasMinLength = password.length >= 8;
	$: passwordsMatch = password === confirmPassword && confirmPassword.length > 0;
</script>

<div class="password-rules">
	<div class="rule" class:valid={hasMinLength} class:invalid={password.length > 0 && !hasMinLength}>
		<Icon icon={hasMinLength ? 'mdi:check-circle' : 'mdi:close-circle'} class="w-3.5 h-3.5" />
		<span>{$t.passwordRules?.minLength || 'At least 8 characters'}</span>
	</div>
	{#if showMatch && confirmPassword.length > 0}
		<div class="rule" class:valid={passwordsMatch} class:invalid={!passwordsMatch}>
			<Icon icon={passwordsMatch ? 'mdi:check-circle' : 'mdi:close-circle'} class="w-3.5 h-3.5" />
			<span>{$t.passwordRules?.passwordsMatch || 'Passwords match'}</span>
		</div>
	{/if}
</div>

<style>
	.password-rules {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 6px;
	}

	.rule {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: #94a3b8;
		transition: color 0.2s ease;
	}

	.rule.valid {
		color: #16a34a;
	}

	.rule.invalid {
		color: #dc2626;
	}

	.rule :global(svg) {
		flex-shrink: 0;
	}
</style>
