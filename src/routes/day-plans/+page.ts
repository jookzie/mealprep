import { listCategories, listDayPlans, loadTargets, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [dayPlans, categories, targets] = await read(
		Promise.all([listDayPlans(), listCategories('day-plan'), loadTargets()]),
	);
	return { dayPlans, categories, targets };
};
