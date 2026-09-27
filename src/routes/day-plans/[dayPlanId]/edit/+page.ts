import { getDayPlan, read } from '$lib/api';
import { loadComposition } from '../../composition';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const [dayPlan, composition] = await Promise.all([
		read(getDayPlan(params.dayPlanId)),
		loadComposition(),
	]);
	return { dayPlan, ...composition };
};
