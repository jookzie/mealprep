import { getProduct, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => ({
	product: await read(getProduct(params.productId)),
});
