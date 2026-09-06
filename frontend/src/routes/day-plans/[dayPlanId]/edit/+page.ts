import { getDayPlan, listMeals } from '$lib/api';
import type { PageLoad } from './$types';

// A day plan is replaced wholesale by PUT, so the editor always starts from a fresh read.
export const load: PageLoad = async ({ fetch, params }) => {
	const [plan, meals] = await Promise.all([
		getDayPlan({ fetch, path: { dayPlanId: params.dayPlanId }, throwOnError: true }),
		listMeals({ fetch, throwOnError: true }),
	]);
	return { dayPlan: plan.data.dayPlan, meals: meals.data.meals };
};
