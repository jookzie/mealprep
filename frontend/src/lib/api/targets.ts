import { getTargets } from './gen/sdk.gen';
import type { Targets } from './gen/types.gen';

/**
 * Targets are a singleton the user may never have set, and the API says so with a
 * 404. That is an empty state, not a failure, so this is the one read that does not
 * throw: it inspects the status and turns the absent case into null. Every screen
 * that needs targets goes through here rather than repeating the check.
 */
export async function loadTargets(fetch: typeof globalThis.fetch): Promise<Targets | null> {
	const { data, error, response } = await getTargets({ fetch, throwOnError: false });
	if (response?.status === 404) return null;
	if (error || !data) throw error ?? new Error('Failed to load targets');
	return data.targets;
}
