import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// The backend port comes from the same .env mise loads for `mise run backend:start`,
// so the proxy target cannot drift from the server it points at.
const backend = `http://localhost:${process.env.PORT ?? 8080}`;

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
			},

			// A single-page app: every route is served from one shell and rendered in the
			// browser. The backend stays a separate process; nothing here runs on a server.
			adapter: adapter({ fallback: 'index.html' }),
		}),
	],
	server: {
		proxy: {
			'/v1': backend,
		},
	},
});
