import { listCategories, listMeals, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [meals, categories] = await read(Promise.all([listMeals(), listCategories('meal')]));
	return { meals, categories };
};
