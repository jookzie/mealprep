import {
	applyAssistantEdit,
	type ChatTurn,
	errorKind,
	messageOf,
	type Preview,
	proposeAssistantEdit,
	runMutation,
} from '$lib/api';
import { type ChatMessage, chatHistory, summarisePreview } from '$lib/domain/assistant';
import { todayIso } from '$lib/domain/week';
import { toasts } from '$lib/toasts.svelte';

/**
 * The conversation with the assistant: what was asked, what it answered, and what became
 * of each proposed change.
 *
 * Held here rather than in the screen: a proposal takes the better part of a minute, and
 * state owned by the page died with it, so switching tabs while it ran threw the answer
 * away. Still memory only — nothing is written until Apply, and closing the app or
 * starting over leaves no trace beyond what was applied.
 */
class AssistantSession {
	/** What the user is typing, kept so it survives leaving the tab. */
	draft = $state('');
	messages = $state<ChatMessage[]>([]);
	applying = $state(false);
	#nextId = 1;
	#backgrounded = 0;

	constructor() {
		document.addEventListener('visibilitychange', () => {
			if (document.hidden) this.#backgrounded++;
		});
	}

	get asking(): boolean {
		return this.messages.some((message) => message.status === 'asking');
	}

	/**
	 * Sends what is in the draft. Only the latest change can be applied, so a change still
	 * waiting is discarded by asking again — a follow-up ("make it 30 g instead") replaces
	 * it with a new proposal.
	 */
	async send() {
		const text = this.draft.trim();
		if (text === '' || this.asking) return;
		const history = chatHistory(this.messages);
		for (const message of this.messages) {
			if (message.status === 'proposed') message.status = 'discarded';
		}

		const id = this.#nextId++;
		this.messages.push({ id, request: text, preview: null, status: 'asking' });
		this.draft = '';
		try {
			const preview = await this.#propose(text, history);
			const message = this.#find(id);
			if (!message) return;
			message.preview = preview;
			message.status = summarisePreview(preview, todayIso()).isEmpty ? 'answered' : 'proposed';
		} catch (reason) {
			// The request goes back into the box rather than staying as a turn with no reply,
			// so trying again is one tap.
			this.messages = this.messages.filter((message) => message.id !== id);
			if (this.draft === '') this.draft = text;
			toasts.push('error', messageOf(reason));
		}
	}

	/**
	 * Android freezes an app soon after it leaves the screen, and the connection to the
	 * assistant does not survive that. A request that failed while the app was away is
	 * still wanted, so it runs again once the app is back instead of reporting the drop.
	 */
	async #propose(text: string, history: ChatTurn[]): Promise<Preview> {
		for (;;) {
			const before = this.#backgrounded;
			try {
				return await proposeAssistantEdit(text, history);
			} catch (reason) {
				const away = this.#backgrounded !== before || document.hidden;
				if (!away || errorKind(reason) !== 'assistant-unavailable') throw reason;
				await shown();
			}
		}
	}

	async apply(id: number) {
		const message = this.#find(id);
		const proposal = message?.preview?.proposal;
		if (!message || !proposal || message.status !== 'proposed' || this.applying) return;
		this.applying = true;
		const applied = await runMutation(() => applyAssistantEdit(proposal), {
			success: 'Applied to your plan',
		});
		this.applying = false;
		if (applied) message.status = 'applied';
	}

	discard(id: number) {
		const message = this.#find(id);
		if (message?.status === 'proposed') message.status = 'discarded';
	}

	/** Starts over. Whatever was applied stays applied; the rest is forgotten. */
	clear() {
		if (this.asking) return;
		this.messages = [];
	}

	#find(id: number): ChatMessage | undefined {
		return this.messages.find((message) => message.id === id);
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
