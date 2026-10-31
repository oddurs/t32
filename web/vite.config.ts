import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	// SPEC.md lives at the repository root, one level up.
	server: { fs: { allow: ['..'] } }
});
