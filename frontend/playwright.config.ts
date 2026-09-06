import { defineConfig } from '@playwright/test';

// Config-level grepInvert cannot be overridden from the command line, so which half of
// the suite runs is selected here rather than by --grep.
const networkOnly = process.env.E2E_NETWORK === '1';

// The smoke tests drive the real backend on :8080 — start it with
// `mise run backend:start` first. Anything that reaches Open Food Facts is tagged
// @network and skipped by default, so the suite does not fail because a third party
// is unreachable; `mise run frontend:e2e:network` runs those.
export default defineConfig({
	testDir: 'e2e',
	fullyParallel: false,
	workers: 1,
	// Tests that reach Open Food Facts are opt-in; the default run stays offline.
	...(networkOnly ? { grep: /@network/ } : { grepInvert: /@network/ }),
	webServer: {
		command: 'bun --bun run build && bun --bun run preview',
		port: 4173,
		reuseExistingServer: true,
		timeout: 120_000,
	},
	use: { baseURL: 'http://localhost:4173' },
});
