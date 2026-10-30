import { isAcceptanceBuild, loadTargets, loadWeightSeries, read } from '$lib/api';
import type { PageLoad } from './$types';

// The settings-shaped screen: what plans are measured against, the weight they exist to
// move, and the data itself. The assistant's key status comes from the layout.
export const load: PageLoad = async () => ({
	targets: await read(loadTargets()),
	weight: await read(loadWeightSeries()),
	// False in every production build, which is where the reset control must not appear.
	acceptance: await isAcceptanceBuild(),
});
