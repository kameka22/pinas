import { writable, derived, get } from 'svelte/store';
import { api, auth } from './api';

export interface SetupConfig {
	machineName: string;
	adminUsername: string;
	adminPassword: string;
}

interface OnboardingStore {
	isLoading: boolean; // True until we've checked localStorage/API
	isSetupComplete: boolean;
	isTransitioning: boolean; // True during fade transition
	currentStep: number;
	config: SetupConfig;
	error: string | null;
}

const STORAGE_KEY = 'pinas-setup-complete';

function loadSetupState(): boolean {
	if (typeof window === 'undefined') return false;
	return localStorage.getItem(STORAGE_KEY) === 'true';
}

function saveSetupState(complete: boolean) {
	if (typeof window === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, complete ? 'true' : 'false');
}

function createOnboardingStore() {
	const { subscribe, set, update } = writable<OnboardingStore>({
		isLoading: true, // Start loading until init() is called
		isSetupComplete: false,
		isTransitioning: false,
		currentStep: 1,
		config: {
			machineName: '',
			adminUsername: '',
			adminPassword: ''
		},
		error: null
	});

	return {
		subscribe,

		init: async () => {
			// First try to check with the backend
			try {
				console.log('[Onboarding] Checking setup status from backend...');
				const status = await api.getSetupStatus();
				console.log('[Onboarding] Backend response:', status);

				update((state) => ({
					...state,
					isLoading: false,
					isSetupComplete: status.is_complete
				}));

				// Sync localStorage with backend state
				saveSetupState(status.is_complete);
				console.log('[Onboarding] Setup complete:', status.is_complete);
			} catch (error) {
				// Fallback to localStorage if backend is unavailable
				console.warn('[Onboarding] Failed to check setup status from backend:', error);
				const localState = loadSetupState();
				console.log('[Onboarding] Falling back to localStorage:', localState);

				update((state) => ({
					...state,
					isLoading: false,
					isSetupComplete: localState
				}));
			}
		},

		setStep: (step: number) => {
			update((state) => ({ ...state, currentStep: step }));
		},

		nextStep: () => {
			update((state) => ({ ...state, currentStep: state.currentStep + 1 }));
		},

		prevStep: () => {
			update((state) => ({ ...state, currentStep: Math.max(1, state.currentStep - 1) }));
		},

		updateConfig: (partial: Partial<SetupConfig>) => {
			update((state) => ({
				...state,
				config: { ...state.config, ...partial }
			}));
		},

		clearError: () => {
			update((state) => ({ ...state, error: null }));
		},

		completeSetup: async (): Promise<boolean> => {
			const currentState = get({ subscribe });
			const { config } = currentState;

			// Start transition
			update((state) => ({ ...state, isTransitioning: true, error: null }));

			try {
				// Call backend API to complete setup
				const response = await api.completeSetup({
					machine_name: config.machineName,
					admin_username: config.adminUsername,
					admin_password: config.adminPassword
				});

				// Auto-login with the returned token
				localStorage.setItem('token', response.token);
				localStorage.setItem(
					'user',
					JSON.stringify({
						id: response.user.id,
						username: response.user.username,
						role: response.user.is_admin ? 'admin' : 'user'
					})
				);

				// Update auth store
				auth.set({
					isAuthenticated: true,
					token: response.token,
					user: {
						id: response.user.id,
						username: response.user.username,
						role: response.user.is_admin ? 'admin' : 'user'
					}
				});

				// Mark setup as complete
				saveSetupState(true);

				// Wait for transition animation
				await new Promise((resolve) => setTimeout(resolve, 500));

				update((state) => ({
					...state,
					isSetupComplete: true,
					isTransitioning: false
				}));

				return true;
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : 'Setup failed';
				console.error('Setup failed:', error);

				update((state) => ({
					...state,
					isTransitioning: false,
					error: errorMessage
				}));

				return false;
			}
		},

		// Pour les tests/dev : reset l'onboarding
		resetSetup: () => {
			saveSetupState(false);
			localStorage.removeItem('token');
			localStorage.removeItem('user');

			auth.set({
				isAuthenticated: false,
				token: null,
				user: null
			});

			set({
				isLoading: false,
				isSetupComplete: false,
				isTransitioning: false,
				currentStep: 1,
				config: {
					machineName: '',
					adminUsername: '',
					adminPassword: ''
				},
				error: null
			});
		}
	};
}

export const onboardingStore = createOnboardingStore();

// Derived stores
export const isLoading = derived(onboardingStore, ($store) => $store.isLoading);
export const isSetupComplete = derived(onboardingStore, ($store) => $store.isSetupComplete);
export const isTransitioning = derived(onboardingStore, ($store) => $store.isTransitioning);
export const currentStep = derived(onboardingStore, ($store) => $store.currentStep);
export const setupConfig = derived(onboardingStore, ($store) => $store.config);
export const setupError = derived(onboardingStore, ($store) => $store.error);

// Actions
export const {
	init: initOnboarding,
	setStep,
	nextStep,
	prevStep,
	updateConfig,
	completeSetup,
	resetSetup,
	clearError
} = onboardingStore;
