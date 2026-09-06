import { toast } from 'svelte-sonner';
import { goto, invalidateAll } from '$app/navigation';
import { messageOf } from './errors';

export type MutationOptions = {
	/** Toast shown when the call succeeds. */
	success?: string;
	/** Re-run every load function afterwards. On by default. */
	invalidate?: boolean;
	/** Where to go once it has succeeded. */
	redirectTo?: string;
};

/**
 * Runs one write against the API and settles what follows it: a toast, a refresh of
 * the loaded data, and a redirect. Every mutation in the app goes through here, so
 * the failure path is written once; it never throws, and answers undefined when the
 * call failed.
 */
export async function runMutation<T>(
	call: () => Promise<T>,
	options: MutationOptions = {},
): Promise<T | undefined> {
	try {
		const result = await call();
		if (options.success) toast.success(options.success);
		if (options.invalidate !== false) await invalidateAll();
		if (options.redirectTo) await goto(options.redirectTo);
		return result;
	} catch (error) {
		toast.error(messageOf(error));
		return undefined;
	}
}
