import { defineConfig } from '@hey-api/openapi-ts';

// The same api/openapi.yaml that generates the Go server generates this client, so a
// call that drifts from the contract stops typechecking. Regenerate both sides with
// `mise run openapi:gen`; never edit src/lib/api/gen by hand.
export default defineConfig({
	input: '../api/openapi.yaml',
	output: {
		path: 'src/lib/api/gen',
		// Biome owns formatting; it skips this directory entirely.
		postProcess: [],
	},
	plugins: ['@hey-api/client-fetch', '@hey-api/sdk', '@hey-api/typescript'],
});
