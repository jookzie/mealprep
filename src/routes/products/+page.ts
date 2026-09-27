import { listProducts, read } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => ({
	products: await read(listProducts()),
});
