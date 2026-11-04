import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

// The site is served at the root of https://t32.serre.ai. Set BASE_PATH=/t32 to
// serve a fork from https://<user>.github.io/t32/ instead.
const base = (process.env.BASE_PATH ?? '') as '' | `/${string}`;

export default defineConfig({
	plugins: [
		sveltekit({
			preprocess: vitePreprocess(),
			adapter: adapter({ fallback: '404.html' }),
			// SvelteKit 3 drops $lib in favour of #lib; keeping it leaves every import as it was.
			alias: { $lib: 'src/lib' },
			paths: { base }
		})
	],
	// SPEC.md lives at the repository root, one level up.
	server: { fs: { allow: ['..'] } }
});
