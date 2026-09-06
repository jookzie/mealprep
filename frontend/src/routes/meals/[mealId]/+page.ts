import { getMeal, listProducts } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const [meal, products] = await Promise.all([
		getMeal({ fetch, path: { mealId: params.mealId }, throwOnError: true }),
		listProducts({ fetch, throwOnError: true }),
	]);
	return { meal: meal.data.meal, products: products.data.products };
};
