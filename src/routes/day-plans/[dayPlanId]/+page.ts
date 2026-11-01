import { getDayPlan, listCategories, loadTargets, loadWeightSeries, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const [dayPlan, categories, targets, weight] = await read(
		Promise.all([
			getDayPlan(params.dayPlanId),
			listCategories('day-plan'),
			loadTargets(),
			loadWeightSeries(),
		]),
	);
	return { dayPlan, categories, targets, kilograms: weight.trend };
};
