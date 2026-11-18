import { beforeEach, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { api } from './api.js';
import { onboardingStore, completeSetup, setStep } from './onboarding.js';

vi.mock('./api.js', () => ({
	api: { completeSetup: vi.fn() },
	auth: { set: vi.fn() },
}));

beforeEach(() => {
	setStep(7);
});

it.each([
	['SETUP_INVALID_HOSTNAME', 3],
	['SETUP_HOSTNAME_FAILED', 3],
	['SETUP_INVALID_USERNAME', 4],
	['SETUP_INVALID_PASSWORD', 5],
	['INTERNAL_ERROR', 7],
])(
	'returns setup error %s to step %s with a translated message',
	async (code, step) => {
		vi.mocked(api.completeSetup).mockRejectedValueOnce(
			Object.assign(new Error('raw backend error'), { code }),
		);
		expect(await completeSetup()).toBe(false);
		const state = get(onboardingStore);
		expect(state.currentStep).toBe(step);
		expect(state.isTransitioning).toBe(false);
		expect(state.error).toBeTruthy();
		expect(state.error).not.toContain('raw backend error');
	},
);
