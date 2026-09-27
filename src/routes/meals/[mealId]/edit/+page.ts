import { getMeal, listCategories, listProducts, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const [meal, products, categories] = await read(
		Promise.all([getMeal(params.mealId), listProducts(), listCategories('meal')]),
	);
	return { meal, products, categories };
};
