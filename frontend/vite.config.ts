import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.ts'],
		setupFiles: ['src/tests/setup.ts'],
		restoreMocks: true
	},
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:3388',
				changeOrigin: true
			}
		}
	}
});
