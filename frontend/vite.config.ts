import tailwindcss from '@tailwindcss/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 1998,
		// This proxy configuration is the key to connecting the frontend to the backend.
		proxy: {
			'/api': {
				// The address of your Rust Axum backend.
				target: 'http://localhost:8000',
				// Necessary for virtual hosted sites.
				changeOrigin: true,
				secure: false,
				// Do not rewrite the path. For example, '/api/auth/login' remains '/api/auth/login'.
				rewrite: (path) => path.replace(/^\/api/, '/api')
			}
		}
	},
	test: {
		projects: [
			{
				extends: './vite.config.ts',
				plugins: [svelteTesting()],
				test: {
					name: 'client',
					environment: 'jsdom',
					clearMocks: true,
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**'],
					setupFiles: ['./vitest-setup-client.ts']
				}
			},
			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
