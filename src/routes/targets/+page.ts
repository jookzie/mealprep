import { isAcceptanceBuild, loadTargets, loadWeightSeries, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => ({
	targets: await read(loadTargets()),
	weight: await read(loadWeightSeries()),
	// False in every production build, which is where the reset control must not appear.
	acceptance: await isAcceptanceBuild(),
});
