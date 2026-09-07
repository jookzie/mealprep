import { messageOf, searchProducts } from '$lib/api';
import type { PageLoad } from './$types';

// The query lives in the URL, so the search is a load like every other read and the
// result is shareable and back-button correct. A blank q would 400, so it is never sent.
export const load: PageLoad = async ({ fetch, url }) => {
	const query = url.searchParams.get('q')?.trim() ?? '';
	if (query === '') return { query, entries: null, rateLimited: false, failure: null };

	// Open Food Facts allows ten searches a minute per address. Hitting that is a
	// normal outcome of typing quickly, not a broken screen, so it is an empty state
	// with an explanation rather than a thrown error.
	const { data, error, response } = await searchProducts({
		fetch,
		query: { q: query },
		throwOnError: false,
	});

	if (response?.status === 429) {
		return { query, entries: null, rateLimited: true, failure: null };
	}
	if (error || !data) {
		return { query, entries: null, rateLimited: false, failure: messageOf(error) };
	}

	return { query, entries: data.entries, rateLimited: false, failure: null };
};
