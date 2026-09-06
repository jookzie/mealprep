import { getProduct } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const { data } = await getProduct({
		fetch,
		path: { productId: params.productId },
		throwOnError: true,
	});
	return { product: data.product };
};
