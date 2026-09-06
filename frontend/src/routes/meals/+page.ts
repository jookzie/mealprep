import { listMeals } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const { data } = await listMeals({ fetch, throwOnError: true });
	return { meals: data.meals };
};
