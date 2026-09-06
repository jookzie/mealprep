import { loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => ({
	targets: await loadTargets(fetch),
});
