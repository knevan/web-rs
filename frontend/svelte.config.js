import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
	preprocess: vitePreprocess(),
	kit: { adapter: adapter() },
	vite: {
		server: {
			proxy: {
				'/mangaupdates': 'http://localhost:3000'
			}
		}
	}
};

export default config;
