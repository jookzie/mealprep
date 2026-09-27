import { listCategories, listMeals, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [categories, meals] = await read(Promise.all([listCategories('meal'), listMeals()]));
	return { categories, meals };
};
