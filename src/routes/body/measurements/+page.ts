import { listMeasurements, loadWeightSeries, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [measurements, weight] = await read(Promise.all([listMeasurements(), loadWeightSeries()]));
	return { measurements, weight };
};
