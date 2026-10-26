import {
	applyAssistantEdit,
	errorKind,
	messageOf,
	type Preview,
	proposeAssistantEdit,
	runMutation,
} from '$lib/api';
import { toasts } from '$lib/toasts.svelte';

/**
 * The assistant's request and the proposal it is waiting on or showing.
 *
 * Held here rather than in the screen: a proposal takes the better part of a minute, and
 * state owned by the page died with it, so switching tabs while it ran threw the answer
 * away. Still memory only — nothing is written until Apply, and Discard leaves no trace.
 */
class AssistantSession {
	request = $state('');
	preview = $state<Preview | null>(null);
	asking = $state(false);
	applying = $state(false);
	#backgrounded = 0;

	constructor() {
		document.addEventListener('visibilitychange', () => {
			if (document.hidden) this.#backgrounded++;
		});
	}

	async ask() {
		const text = this.request.trim();
		if (text === '' || this.asking) return;
		this.asking = true;
		try {
			this.preview = await this.#propose(text);
		} catch (reason) {
			toasts.push('error', messageOf(reason));
		} finally {
			this.asking = false;
		}
	}

	/**
	 * Android freezes an app soon after it leaves the screen, and the connection to the
	 * assistant does not survive that. A request that failed while the app was away is
	 * still wanted, so it runs again once the app is back instead of reporting the drop.
	 */
	async #propose(text: string): Promise<Preview> {
		for (;;) {
			const before = this.#backgrounded;
			try {
				return await proposeAssistantEdit(text);
			} catch (reason) {
				const away = this.#backgrounded !== before || document.hidden;
				if (!away || errorKind(reason) !== 'assistant-unavailable') throw reason;
				await shown();
			}
		}
	}

	async apply() {
		const proposal = this.preview?.proposal;
		if (!proposal || this.applying) return;
		this.applying = true;
		const applied = await runMutation(() => applyAssistantEdit(proposal), {
			success: 'Applied to your plan',
		});
		this.applying = false;
		if (applied) {
			this.preview = null;
			this.request = '';
		}
	}

	discard() {
		this.preview = null;
	}
}

function shown(): Promise<void> {
	if (!document.hidden) return Promise.resolve();
	return new Promise((resolve) => {
		const listener = () => {
			if (document.hidden) return;
			document.removeEventListener('visibilitychange', listener);
			resolve();
		};
		document.addEventListener('visibilitychange', listener);
	});
}

export const assistant = new AssistantSession();
