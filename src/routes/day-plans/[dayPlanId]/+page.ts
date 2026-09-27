import { getDayPlan, listCategories, loadTargets, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const [dayPlan, categories, targets] = await read(
		Promise.all([getDayPlan(params.dayPlanId), listCategories('day-plan'), loadTargets()]),
	);
	return { dayPlan, categories, targets };
};
