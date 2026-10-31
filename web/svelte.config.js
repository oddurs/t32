import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
export default {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({ fallback: '404.html' }),
		// Set BASE_PATH=/t32 when serving from https://<user>.github.io/t32/
		paths: { base: process.env.BASE_PATH ?? '' }
	}
};
