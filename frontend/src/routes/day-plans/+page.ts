import { listDayPlans, loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

// The radars share one denominator so the shapes compare to each other; the daily
// target is that denominator when there is one.
export const load: PageLoad = async ({ fetch }) => {
	const [dayPlans, targets] = await Promise.all([
		listDayPlans({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);
	return { dayPlans: dayPlans.data.dayPlans, targets };
};
