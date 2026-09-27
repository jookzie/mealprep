import { listCategories, listDayPlans, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [categories, dayPlans] = await read(
		Promise.all([listCategories('day-plan'), listDayPlans()]),
	);
	return { categories, dayPlans };
};
