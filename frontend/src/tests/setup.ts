// Vitest setup: SvelteKit's `$app/environment` is provided by the kit plugin,
// but a browser-like localStorage/fetch are jsdom's. Reset storage between tests.
import { afterEach, vi } from 'vitest';

afterEach(() => {
	localStorage.clear();
	vi.unstubAllGlobals();
});
