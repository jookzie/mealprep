import { listProducts } from '$lib/api';
import type { PageLoad } from './$types';

// Servings reference products by id, so the picker needs the product list.
export const load: PageLoad = async ({ fetch }) => {
	const { data } = await listProducts({ fetch, throwOnError: true });
	return { products: data.products };
};
