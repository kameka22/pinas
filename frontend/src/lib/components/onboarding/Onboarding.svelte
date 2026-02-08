<script lang="ts">
	import Icon from '@iconify/svelte';
	import {
		currentStep,
		isTransitioning,
		nextStep,
		prevStep as storePrevStep,
		updateConfig,
		completeSetup,
		setupError,
		clearError
	} from '$stores/onboarding';
	import { t, locale, languages, type Locale } from '$lib/i18n';

	// Local form values
	let selectedLanguage: Locale = $locale;
	let machineName = 'pinas';
	let adminUsername = '';
	let adminPassword = '';
	let confirmPassword = '';

	// SSH form values
	let enableSsh = false;
	let sshPassword = '';
	let sshConfirmPassword = '';

	// Validation errors
	let passwordError = '';
	let machineNameError = '';
	let usernameError = '';
	let sshPasswordError = '';

	// Submission state
	let isSubmitting = false;

	// Slide direction
	let slideDirection: 'forward' | 'backward' = 'forward';

	const TOTAL_STEPS = 7;

	const stepLabels = ['welcome', 'language', 'device', 'account', 'password', 'ssh', 'features'] as const;

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

	function validateSshPassword(): boolean {
		if (!enableSsh) return true;
		if (!sshPassword) {
			sshPasswordError = $t.onboarding.validation.passwordRequired;
			return false;
		}
		if (sshPassword.length < 4) {
			sshPasswordError = $t.onboarding.validation.sshPasswordMinLength;
			return false;
		}
		if (sshPassword !== sshConfirmPassword) {
			sshPasswordError = $t.onboarding.validation.sshPasswordMismatch;
			return false;
		}
		sshPasswordError = '';
		return true;
	}

	async function handleNext() {
		clearError();

		if ($currentStep === 1) {
			// Welcome step - just proceed
		} else if ($currentStep === 2) {
			// Language step - just proceed
		} else if ($currentStep === 3) {
			if (!validateMachineName()) return;
			updateConfig({ machineName });
		} else if ($currentStep === 4) {
			if (!validateUsername()) return;
			updateConfig({ adminUsername });
		} else if ($currentStep === 5) {
			if (!validatePassword()) return;
			updateConfig({ adminPassword });
		} else if ($currentStep === 6) {
			if (!validateSshPassword()) return;
			updateConfig({ enableSsh, sshPassword: enableSsh ? sshPassword : '' });
		} else if ($currentStep === 7) {
			// Final step - complete setup
			isSubmitting = true;
			try {
				const success = await completeSetup();
				if (!success) {
					isSubmitting = false;
				}
			} catch (e) {
				isSubmitting = false;
			}
			return;
		}
		slideDirection = 'forward';
		nextStep();
	}

	function handleBack() {
		slideDirection = 'backward';
		storePrevStep();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !isSubmitting) {
			handleNext();
		}
	}

	// Features list for the final screen
	const featureItems = [
		{ key: 'files', icon: 'mdi:folder-multiple', color: '#3b82f6' },
		{ key: 'storage', icon: 'mdi:harddisk', color: '#8b5cf6' },
		{ key: 'shares', icon: 'mdi:share-variant', color: '#10b981' },
		{ key: 'docker', icon: 'mdi:docker', color: '#06b6d4' },
		{ key: 'kodi', icon: 'mdi:kodi', color: '#f59e0b' },
		{ key: 'terminal', icon: 'mdi:console', color: '#64748b' }
	] as const;
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
			{#key $currentStep}
			<div class="step-slide" class:slide-forward={slideDirection === 'forward'} class:slide-backward={slideDirection === 'backward'}>
			{#if $currentStep === 1}
				<!-- Step 1: Welcome -->
				<div class="step-panel">
					<h2>{$t.onboarding.welcomeScreen.title}</h2>
					<p class="step-description">{$t.onboarding.welcomeScreen.thankYou}</p>
					<p class="step-description secondary">{$t.onboarding.welcomeScreen.description}</p>
				</div>

			{:else if $currentStep === 2}
				<!-- Step 2: Language Selection -->
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

			{:else if $currentStep === 3}
				<!-- Step 3: Machine Name -->
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

			{:else if $currentStep === 4}
				<!-- Step 4: Admin Username -->
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

			{:else if $currentStep === 5}
				<!-- Step 5: Password -->
				<div class="step-panel">
					<h2>{$t.onboarding.password.title}</h2>
					<p class="step-description">{$t.onboarding.password.description}</p>

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

			{:else if $currentStep === 6}
				<!-- Step 6: SSH Configuration -->
				<div class="step-panel">
					<h2>{$t.onboarding.ssh.title}</h2>
					<p class="step-description">{$t.onboarding.ssh.description}</p>

					<div class="ssh-toggle">
						<label class="toggle-row" for="enable-ssh">
							<div class="toggle-info">
								<span class="toggle-label">{$t.onboarding.ssh.enableSsh}</span>
								<span class="toggle-hint">{$t.onboarding.ssh.enableHint}</span>
							</div>
							<div class="toggle-switch" class:active={enableSsh}>
								<input
									type="checkbox"
									id="enable-ssh"
									bind:checked={enableSsh}
								/>
								<span class="toggle-slider"></span>
							</div>
						</label>
					</div>

					{#if enableSsh}
						<div class="ssh-fields">
							<div class="input-group">
								<label for="ssh-password">{$t.onboarding.ssh.passwordLabel}</label>
								<input
									id="ssh-password"
									type="password"
									placeholder={$t.onboarding.ssh.passwordPlaceholder}
									bind:value={sshPassword}
									on:keydown={handleKeydown}
									class:error={sshPasswordError}
								/>
							</div>
							<div class="input-group">
								<label for="ssh-confirm-password">{$t.onboarding.ssh.confirmPasswordLabel}</label>
								<input
									id="ssh-confirm-password"
									type="password"
									placeholder={$t.onboarding.ssh.confirmPasswordPlaceholder}
									bind:value={sshConfirmPassword}
									on:keydown={handleKeydown}
									class:error={sshPasswordError}
								/>
								{#if sshPasswordError}
									<span class="error-message">{sshPasswordError}</span>
								{/if}
							</div>
							<p class="ssh-hint">{$t.onboarding.ssh.passwordHint}</p>
						</div>
					{:else}
						<p class="skip-hint">{$t.onboarding.ssh.skipHint}</p>
					{/if}
				</div>

			{:else if $currentStep === 7}
				<!-- Step 7: Features Tour -->
				<div class="step-panel">
					{#if $setupError}
						<div class="setup-error">
							<Icon icon="mdi:alert-circle" class="w-5 h-5" />
							<span>{$setupError}</span>
						</div>
					{/if}

					<div class="features-header">
						<Icon icon="mdi:party-popper" class="w-10 h-10" style="color: #f59e0b" />
						<h2>{$t.onboarding.features.title}</h2>
					</div>
					<p class="step-description">{$t.onboarding.features.subtitle}</p>

					<div class="features-grid">
						{#each featureItems as feature}
							<div class="feature-card">
								<div class="feature-icon" style="background: {feature.color}15; color: {feature.color}">
									<Icon icon={feature.icon} class="w-6 h-6" />
								</div>
								<div class="feature-text">
									<span class="feature-title">{$t.onboarding.features.items[feature.key].title}</span>
									<span class="feature-desc">{$t.onboarding.features.items[feature.key].description}</span>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
			</div>
			{/key}
		</div>

		<!-- Footer with buttons -->
		<div class="onboarding-footer">
			{#if $currentStep > 1}
				<button class="btn-secondary" on:click={handleBack}>
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
					<Icon icon="mdi:rocket-launch" class="w-4 h-4" />
				{:else if $currentStep === 1}
					{$t.onboarding.buttons.getStarted}
					<Icon icon="mdi:arrow-right" class="w-4 h-4" />
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
		max-width: 580px;
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
		padding: 20px 16px;
		gap: 2px;
	}

	.progress-step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.step-dot {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: #e2e8f0;
		color: #94a3b8;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
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
		font-size: 9px;
		color: #94a3b8;
		font-weight: 500;
		white-space: nowrap;
	}

	.progress-step.active .step-label {
		color: #3b82f6;
	}

	.progress-line {
		width: 16px;
		height: 2px;
		background: #e2e8f0;
		margin-bottom: 18px;
		transition: all 0.3s ease;
	}

	.progress-line.active {
		background: #3b82f6;
	}

	.step-content {
		padding: 8px 32px 32px;
		overflow: hidden;
	}

	.step-slide.slide-forward {
		animation: slideFromRight 0.3s ease-out;
	}

	.step-slide.slide-backward {
		animation: slideFromLeft 0.3s ease-out;
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

	.step-description.secondary {
		margin-bottom: 0;
		color: #94a3b8;
		font-size: 13px;
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

	/* SSH toggle */
	.ssh-toggle {
		margin-bottom: 20px;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		cursor: pointer;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		text-align: left;
	}

	.toggle-label {
		font-size: 15px;
		font-weight: 500;
		color: #1e293b;
	}

	.toggle-hint {
		font-size: 12px;
		color: #94a3b8;
	}

	.toggle-switch {
		position: relative;
		width: 48px;
		height: 26px;
		flex-shrink: 0;
	}

	.toggle-switch input {
		opacity: 0;
		width: 0;
		height: 0;
		position: absolute;
	}

	.toggle-slider {
		position: absolute;
		inset: 0;
		background: #cbd5e1;
		border-radius: 26px;
		transition: all 0.3s ease;
		cursor: pointer;
	}

	.toggle-slider::before {
		content: '';
		position: absolute;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: white;
		top: 3px;
		left: 3px;
		transition: all 0.3s ease;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle-switch.active .toggle-slider {
		background: #3b82f6;
	}

	.toggle-switch.active .toggle-slider::before {
		transform: translateX(22px);
	}

	.ssh-fields {
		animation: fadeIn 0.2s ease;
	}

	.ssh-hint {
		font-size: 12px;
		color: #94a3b8;
		text-align: left;
		margin-top: -8px;
	}

	.skip-hint {
		font-size: 13px;
		color: #94a3b8;
		padding: 16px;
		background: #f8fafc;
		border-radius: 10px;
		border: 1px dashed #e2e8f0;
	}

	/* Features grid */
	.features-header {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
		margin-bottom: 4px;
	}

	.features-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.feature-card {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px;
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 12px;
		text-align: left;
		transition: all 0.2s ease;
	}

	.feature-card:hover {
		background: #f1f5f9;
		border-color: #cbd5e1;
	}

	.feature-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.feature-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.feature-title {
		font-size: 13px;
		font-weight: 600;
		color: #1e293b;
	}

	.feature-desc {
		font-size: 11px;
		color: #94a3b8;
		line-height: 1.4;
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

	@keyframes slideFromRight {
		from {
			transform: translateX(40px);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	@keyframes slideFromLeft {
		from {
			transform: translateX(-40px);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(-8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
