/// <reference types="bun" />
// Bun is the runtime and the unit-test runner, so its globals and `bun:test` are
// ambient here as well as at run time.

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
