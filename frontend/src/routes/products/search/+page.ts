import { searchProducts } from '$lib/api';
import type { PageLoad } from './$types';

// The query lives in the URL, so the search is a load like every other read and the
// result is shareable and back-button correct. A blank q would 400, so it is never sent.
export const load: PageLoad = async ({ fetch, url }) => {
	const query = url.searchParams.get('q')?.trim() ?? '';
	if (query === '') return { query, entries: null };

	const { data } = await searchProducts({ fetch, query: { q: query }, throwOnError: true });
	return { query, entries: data.entries };
};
