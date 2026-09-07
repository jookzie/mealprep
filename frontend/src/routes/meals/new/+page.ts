import { listProducts, loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

// Servings reference products by id, so the picker needs the product list.
export const load: PageLoad = async ({ fetch }) => {
	const [products, targets] = await Promise.all([
		listProducts({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);
	return { products: products.data.products, targets };
};
