<script lang="ts">
	import Icon from '@iconify/svelte';
	import {
		currentStep,
		isTransitioning,
		nextStep,
		prevStep,
		updateConfig,
		completeSetup,
		setupError,
		clearError
	} from '$stores/onboarding';
	import { t, locale, languages, type Locale } from '$lib/i18n';

	// Local form values
	let selectedLanguage: Locale = $locale;
	let machineName = '';
	let adminUsername = '';
	let adminPassword = '';
	let confirmPassword = '';

	// Validation errors
	let passwordError = '';
	let machineNameError = '';
	let usernameError = '';

	// Submission state
	let isSubmitting = false;

	const TOTAL_STEPS = 4;

	const stepLabels = ['language', 'device', 'account', 'password'] as const;

	function getStepLabel(index: number): string {
		const key = stepLabels[index];
		return $t.onboarding.steps[key];
	}

	function selectLanguage(lang: Locale) {
		selectedLanguage = lang;
		locale.setLocale(lang);
	}

	function validateMachineName(): boolean {
		if (!machineName.trim()) {
			machineNameError = $t.onboarding.validation.machineNameRequired;
			return false;
		}
		if (machineName.length < 2) {
			machineNameError = $t.onboarding.validation.machineNameMinLength;
			return false;
		}
		if (!/^[a-zA-Z0-9-]+$/.test(machineName)) {
			machineNameError = $t.onboarding.validation.machineNameInvalid;
			return false;
		}
		machineNameError = '';
		return true;
	}

	function validateUsername(): boolean {
		if (!adminUsername.trim()) {
			usernameError = $t.onboarding.validation.usernameRequired;
			return false;
		}
		if (adminUsername.length < 3) {
			usernameError = $t.onboarding.validation.usernameMinLength;
			return false;
		}
		if (!/^[a-zA-Z0-9_]+$/.test(adminUsername)) {
			usernameError = $t.onboarding.validation.usernameInvalid;
			return false;
		}
		usernameError = '';
		return true;
	}

	function validatePassword(): boolean {
		if (!adminPassword) {
			passwordError = $t.onboarding.validation.passwordRequired;
			return false;
		}
		if (adminPassword.length < 6) {
			passwordError = $t.onboarding.validation.passwordMinLength;
			return false;
		}
		if (adminPassword !== confirmPassword) {
			passwordError = $t.onboarding.validation.passwordMismatch;
			return false;
		}
		passwordError = '';
		return true;
	}

	async function handleNext() {
		// Clear any previous errors
		clearError();

		if ($currentStep === 1) {
			// Language step - just proceed
		} else if ($currentStep === 2) {
			if (!validateMachineName()) return;
			updateConfig({ machineName });
		} else if ($currentStep === 3) {
			if (!validateUsername()) return;
			updateConfig({ adminUsername });
		} else if ($currentStep === 4) {
			if (!validatePassword()) return;
			updateConfig({ adminPassword });

			// Start submission
			isSubmitting = true;
			try {
				const success = await completeSetup();
				if (!success) {
					// Error will be displayed from the store
					isSubmitting = false;
				}
			} catch (e) {
				isSubmitting = false;
			}
			return;
		}
		nextStep();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !isSubmitting) {
			handleNext();
		}
	}
</script>

<div class="onboarding-overlay" class:fade-out={$isTransitioning}>
	<div class="onboarding-modal">
		<!-- Header with logo -->
		<div class="onboarding-header">
			<div class="logo">
				<Icon icon="mdi:nas" class="w-10 h-10 text-blue-500" />
				<span class="logo-text">PiNAS</span>
			</div>
			<p class="subtitle">{$t.onboarding.welcome} {$t.onboarding.subtitle}</p>
		</div>

		<!-- Progress indicator -->
		<div class="progress-bar">
			{#each Array(TOTAL_STEPS) as _, i}
				<div class="progress-step" class:active={i + 1 <= $currentStep} class:current={i + 1 === $currentStep}>
					<div class="step-dot">
						{#if i + 1 < $currentStep}
							<Icon icon="mdi:check" class="w-4 h-4" />
						{:else}
							{i + 1}
						{/if}
					</div>
					<span class="step-label">{getStepLabel(i)}</span>
				</div>
				{#if i < TOTAL_STEPS - 1}
					<div class="progress-line" class:active={i + 1 < $currentStep}></div>
				{/if}
			{/each}
		</div>

		<!-- Step content -->
		<div class="step-content">
			{#if $currentStep === 1}
				<!-- Step 1: Language Selection -->
				<div class="step-panel">
					<h2>{$t.onboarding.language.title}</h2>
					<p class="step-description">{$t.onboarding.language.description}</p>

					<div class="language-grid">
						{#each languages as lang}
							<button
								class="language-option"
								class:selected={selectedLanguage === lang.code}
								on:click={() => selectLanguage(lang.code)}
							>
								<span class="language-flag">{lang.flag}</span>
								<span class="language-name">{lang.nativeName}</span>
								{#if selectedLanguage === lang.code}
									<Icon icon="mdi:check-circle" class="w-5 h-5 text-blue-500 check-icon" />
								{/if}
							</button>
						{/each}
					</div>
				</div>

			{:else if $currentStep === 2}
				<!-- Step 2: Machine Name -->
				<div class="step-panel">
					<h2>{$t.onboarding.device.title}</h2>
					<p class="step-description">{$t.onboarding.device.description}</p>
					<div class="input-group">
						<label for="machine-name">{$t.onboarding.device.fieldLabel}</label>
						<input
							id="machine-name"
							type="text"
							placeholder={$t.onboarding.device.placeholder}
							bind:value={machineName}
							on:keydown={handleKeydown}
							class:error={machineNameError}
						/>
						{#if machineNameError}
							<span class="error-message">{machineNameError}</span>
						{/if}
					</div>
				</div>

			{:else if $currentStep === 3}
				<!-- Step 3: Admin Username -->
				<div class="step-panel">
					<h2>{$t.onboarding.account.title}</h2>
					<p class="step-description">{$t.onboarding.account.description}</p>
					<div class="input-group">
						<label for="username">{$t.onboarding.account.fieldLabel}</label>
						<input
							id="username"
							type="text"
							placeholder={$t.onboarding.account.placeholder}
							bind:value={adminUsername}
							on:keydown={handleKeydown}
							class:error={usernameError}
						/>
						{#if usernameError}
							<span class="error-message">{usernameError}</span>
						{/if}
					</div>
				</div>

			{:else if $currentStep === 4}
				<!-- Step 4: Password -->
				<div class="step-panel">
					<h2>{$t.onboarding.password.title}</h2>
					<p class="step-description">{$t.onboarding.password.description}</p>

					{#if $setupError}
						<div class="setup-error">
							<Icon icon="mdi:alert-circle" class="w-5 h-5" />
							<span>{$setupError}</span>
						</div>
					{/if}

					<div class="input-group">
						<label for="password">{$t.onboarding.password.fieldLabel}</label>
						<input
							id="password"
							type="password"
							placeholder={$t.onboarding.password.placeholder}
							bind:value={adminPassword}
							on:keydown={handleKeydown}
							class:error={passwordError}
							disabled={isSubmitting}
						/>
					</div>
					<div class="input-group">
						<label for="confirm-password">{$t.onboarding.password.confirmLabel}</label>
						<input
							id="confirm-password"
							type="password"
							placeholder={$t.onboarding.password.confirmPlaceholder}
							bind:value={confirmPassword}
							on:keydown={handleKeydown}
							class:error={passwordError}
							disabled={isSubmitting}
						/>
						{#if passwordError}
							<span class="error-message">{passwordError}</span>
						{/if}
					</div>
				</div>
			{/if}
		</div>

		<!-- Footer with buttons -->
		<div class="onboarding-footer">
			{#if $currentStep > 1}
				<button class="btn-secondary" on:click={prevStep}>
					<Icon icon="mdi:arrow-left" class="w-4 h-4" />
					{$t.onboarding.buttons.back}
				</button>
			{:else}
				<div></div>
			{/if}

			<button class="btn-primary" on:click={handleNext} disabled={isSubmitting}>
				{#if isSubmitting}
					<Icon icon="mdi:loading" class="w-4 h-4 spinning" />
					Setting up...
				{:else if $currentStep === TOTAL_STEPS}
					{$t.onboarding.buttons.complete}
					<Icon icon="mdi:check" class="w-4 h-4" />
				{:else}
					{$t.onboarding.buttons.next}
					<Icon icon="mdi:arrow-right" class="w-4 h-4" />
				{/if}
			</button>
		</div>
	</div>
</div>

<style>
	.onboarding-overlay {
		position: fixed;
		inset: 0;
		background: linear-gradient(135deg, #1e3a5f 0%, #0f172a 100%);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		opacity: 1;
		transition: opacity 0.5s ease-out;
	}

	.onboarding-overlay.fade-out {
		opacity: 0;
	}

	.onboarding-modal {
		background: white;
		border-radius: 20px;
		width: 100%;
		max-width: 520px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
		overflow: hidden;
	}

	.onboarding-header {
		padding: 32px 32px 24px;
		text-align: center;
		background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
		border-bottom: 1px solid #e2e8f0;
	}

	.logo {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
		margin-bottom: 8px;
	}

	.logo-text {
		font-size: 28px;
		font-weight: 700;
		background: linear-gradient(135deg, #3b82f6, #1d4ed8);
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	.subtitle {
		color: #64748b;
		font-size: 14px;
	}

	.progress-bar {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px 24px;
		gap: 4px;
	}

	.progress-step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.step-dot {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background: #e2e8f0;
		color: #94a3b8;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		font-weight: 600;
		transition: all 0.3s ease;
	}

	.progress-step.active .step-dot {
		background: #3b82f6;
		color: white;
	}

	.progress-step.current .step-dot {
		box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.2);
	}

	.step-label {
		font-size: 10px;
		color: #94a3b8;
		font-weight: 500;
		white-space: nowrap;
	}

	.progress-step.active .step-label {
		color: #3b82f6;
	}

	.progress-line {
		width: 24px;
		height: 2px;
		background: #e2e8f0;
		margin-bottom: 20px;
		transition: all 0.3s ease;
	}

	.progress-line.active {
		background: #3b82f6;
	}

	.step-content {
		padding: 8px 32px 32px;
	}

	.step-panel {
		text-align: center;
	}

	.step-panel h2 {
		font-size: 20px;
		font-weight: 600;
		color: #1e293b;
		margin-bottom: 8px;
	}

	.step-description {
		color: #64748b;
		font-size: 14px;
		margin-bottom: 24px;
	}

	/* Language selection */
	.language-grid {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.language-option {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px 20px;
		background: #f8fafc;
		border: 2px solid #e2e8f0;
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: left;
	}

	.language-option:hover {
		background: #f1f5f9;
		border-color: #cbd5e1;
	}

	.language-option.selected {
		background: #eff6ff;
		border-color: #3b82f6;
	}

	.language-flag {
		font-size: 28px;
	}

	.language-name {
		flex: 1;
		font-size: 16px;
		font-weight: 500;
		color: #1e293b;
	}

	.check-icon {
		flex-shrink: 0;
	}

	.input-group {
		text-align: left;
		margin-bottom: 16px;
	}

	.input-group label {
		display: block;
		font-size: 13px;
		font-weight: 500;
		color: #475569;
		margin-bottom: 6px;
	}

	.input-group input {
		width: 100%;
		padding: 12px 16px;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		font-size: 15px;
		transition: all 0.2s ease;
		background: #f8fafc;
		color: #1e293b;
	}

	.input-group input::placeholder {
		color: #94a3b8;
	}

	.input-group input:focus {
		outline: none;
		border-color: #3b82f6;
		background: white;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.input-group input.error {
		border-color: #ef4444;
		background: #fef2f2;
	}

	.error-message {
		display: block;
		color: #ef4444;
		font-size: 12px;
		margin-top: 6px;
	}

	.onboarding-footer {
		display: flex;
		justify-content: space-between;
		padding: 20px 32px;
		background: #f8fafc;
		border-top: 1px solid #e2e8f0;
	}

	.btn-primary,
	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 24px;
		border-radius: 10px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		border: none;
	}

	.btn-primary {
		background: linear-gradient(135deg, #3b82f6, #2563eb);
		color: white;
	}

	.btn-primary:hover {
		background: linear-gradient(135deg, #2563eb, #1d4ed8);
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
	}

	.btn-secondary {
		background: white;
		color: #64748b;
		border: 1px solid #e2e8f0;
	}

	.btn-secondary:hover {
		background: #f1f5f9;
		color: #475569;
	}

	.btn-primary:disabled {
		opacity: 0.7;
		cursor: not-allowed;
		transform: none;
	}

	.btn-primary:disabled:hover {
		transform: none;
		box-shadow: none;
	}

	.setup-error {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		background: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 8px;
		color: #dc2626;
		font-size: 14px;
		margin-bottom: 16px;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
