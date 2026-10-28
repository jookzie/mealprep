import { listAssistantRequests, loadAssistantKeyStatus, read } from '$lib/api';
import type { PageLoad } from './$types';

// The key status and the log are the whole of the screen's loaded state. The chat is not
// loaded: it lives in the session (`$lib/assistant.svelte`) and is never stored; only
// what was applied reaches the log.
export const load: PageLoad = async () => {
	const [key, history] = await read(
		Promise.all([loadAssistantKeyStatus(), listAssistantRequests()]),
	);
	return { key, history };
};
