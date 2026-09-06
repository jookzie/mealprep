import { getDayPlan, loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const [plan, targets] = await Promise.all([
		getDayPlan({ fetch, path: { dayPlanId: params.dayPlanId }, throwOnError: true }),
		loadTargets(fetch),
	]);
	return { dayPlan: plan.data.dayPlan, targets };
};
