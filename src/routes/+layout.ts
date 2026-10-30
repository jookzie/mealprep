import { type AssistantKeyStatus, loadAssistantKeyStatus } from '$lib/api';
import type { LayoutLoad } from './$types';

// A static single-page app: nothing is rendered or prerendered ahead of the webview.
export const ssr = false;
export const prerender = false;

/*
 * The Ask sheet opens over every screen, so whether the assistant has a key is the shell's
 * to know. A failure reads as "no key": the sheet then explains itself and sends nothing,
 * rather than taking every screen down to the error page with it.
 */
export const load: LayoutLoad = async () => ({
	assistantKey: await loadAssistantKeyStatus().catch((): AssistantKeyStatus => ({ present: false })),
});
