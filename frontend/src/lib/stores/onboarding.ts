import { writable, derived } from 'svelte/store';

export interface SetupConfig {
	machineName: string;
	adminUsername: string;
	adminPassword: string;
}

interface OnboardingStore {
	isLoading: boolean; // True until we've checked localStorage
	isSetupComplete: boolean;
	isTransitioning: boolean; // True during fade transition
	currentStep: number;
	config: SetupConfig;
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
		}
	});

	return {
		subscribe,

		init: () => {
			update((state) => ({
				...state,
				isLoading: false,
				isSetupComplete: loadSetupState()
			}));
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

		completeSetup: () => {
			// Start transition
			update((state) => ({ ...state, isTransitioning: true }));

			// After fade out, mark as complete
			setTimeout(() => {
				saveSetupState(true);
				update((state) => ({
					...state,
					isSetupComplete: true,
					isTransitioning: false
				}));
			}, 500); // Match CSS transition duration
		},

		// Pour les tests/dev : reset l'onboarding
		resetSetup: () => {
			saveSetupState(false);
			set({
				isLoading: false,
				isSetupComplete: false,
				isTransitioning: false,
				currentStep: 1,
				config: {
					machineName: '',
					adminUsername: '',
					adminPassword: ''
				}
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

// Actions
export const {
	init: initOnboarding,
	setStep,
	nextStep,
	prevStep,
	updateConfig,
	completeSetup,
	resetSetup
} = onboardingStore;
