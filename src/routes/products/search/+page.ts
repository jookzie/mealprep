import { type CatalogueEntry, messageOf, searchCatalogue } from '$lib/api';
import type { PageLoad } from './$types';

// The query lives in the URL, so a search is navigable and survives a refresh. A catalogue
// failure stays on this page: products can still be entered by hand.
export const load: PageLoad = async ({ url }) => {
	const query = url.searchParams.get('q')?.trim() ?? '';
	let entries: CatalogueEntry[] = [];
	let failure: string | null = null;
	if (query !== '') {
		try {
			entries = await searchCatalogue(query);
		} catch (reason) {
			failure = messageOf(reason);
		}
	}
	return { query, entries, failure };
};
