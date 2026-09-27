import { goto, invalidateAll } from '$app/navigation';
import { toasts } from '$lib/toasts.svelte';
import { messageOf } from './errors';

export type MutationOptions<T> = {
	/** Toast shown when the call succeeds. */
	success?: string;
	/** Re-run every load function afterwards. On by default. */
	invalidate?: boolean;
	/** Where to go once it has succeeded; replaces the current history entry. */
	redirectTo?: string | ((result: T) => string);
};

/**
 * Runs one write and settles what follows it: a toast, a refresh of the loaded data, and
 * a redirect. It never throws, and answers undefined when the call failed.
 *
 * The redirect replaces the history entry, so going back never returns to a submitted form.
 */
export async function runMutation<T>(
	call: () => Promise<T>,
	options: MutationOptions<T> = {},
): Promise<T | undefined> {
	try {
		const result = await call();
		if (options.success) toasts.push('success', options.success);
		if (options.redirectTo) {
			const target =
				typeof options.redirectTo === 'function' ? options.redirectTo(result) : options.redirectTo;
			await goto(target, { replaceState: true, invalidateAll: options.invalidate !== false });
		} else if (options.invalidate !== false) {
			await invalidateAll();
		}
		return result;
	} catch (reason) {
		toasts.push('error', messageOf(reason));
		return undefined;
	}
}
