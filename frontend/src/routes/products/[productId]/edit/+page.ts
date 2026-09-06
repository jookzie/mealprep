import { getProduct } from '$lib/api';
import type { PageLoad } from './$types';

// A product is replaced wholesale by PUT, so the editor always starts from a fresh read.
export const load: PageLoad = async ({ fetch, params }) => {
	const { data } = await getProduct({
		fetch,
		path: { productId: params.productId },
		throwOnError: true,
	});
	return { product: data.product };
};
