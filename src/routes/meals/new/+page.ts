import { listCategories, listProducts, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const [products, categories] = await read(
		Promise.all([listProducts(), listCategories('meal')]),
	);
	return { products, categories };
};
