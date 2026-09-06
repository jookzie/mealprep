import { listProducts } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const { data } = await listProducts({ fetch, throwOnError: true });
	return { products: data.products };
};
