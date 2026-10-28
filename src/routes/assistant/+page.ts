import { loadAssistantKeyStatus, read } from '$lib/api';
import type { PageLoad } from './$types';

// Whether there is a key is the whole of the screen's loaded state. The chat lives in the
// session (`$lib/assistant.svelte`) and is never stored.
export const load: PageLoad = async () => {
	const key = await read(loadAssistantKeyStatus());
	return { key };
};
