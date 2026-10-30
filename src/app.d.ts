// See https://svelte.dev/docs/kit/types#app.d.ts
declare global {
	namespace App {
		/**
		 * Shallow-routing state. The Ask sheet is a history entry of its own, so Android's back
		 * gesture closes it rather than leaving the screen underneath.
		 */
		interface PageState {
			ask?: boolean;
		}
	}
}

export {};
