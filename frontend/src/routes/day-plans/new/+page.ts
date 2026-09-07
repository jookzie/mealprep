import { listMeals, loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const [meals, targets] = await Promise.all([
		listMeals({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);
	return { meals: meals.data.meals, targets };
};
