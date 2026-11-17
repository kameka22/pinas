import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			preprocess: vitePreprocess(),
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: 'index.html',
				precompress: false,
				strict: true
			})
		})
	],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.{test,spec}.ts'],
		setupFiles: ['src/tests/setup.ts'],
		restoreMocks: true
	},
	server: {
		proxy: {
			'/api': {
				// Backend dev server (backend/.env.dev). Overridden in docker-compose (service name).
				target: process.env.VITE_API_PROXY || 'http://localhost:3388',
				changeOrigin: true
			}
		}
	}
});
