import { listAssistantRequests, loadAssistantKeyStatus, read } from '$lib/api';
import type { PageLoad } from './$types';

// The key status and the log are the whole of the screen's state. A proposal is not
// loaded: it is asked for, lives in the component until it is applied or discarded, and
// is never stored.
export const load: PageLoad = async () => {
	const [key, history] = await read(
		Promise.all([loadAssistantKeyStatus(), listAssistantRequests()]),
	);
	return { key, history };
};
