import { loadWeightSeries, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => ({
	series: await read(loadWeightSeries()),
});
