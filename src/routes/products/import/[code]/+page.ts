import { type CatalogueEntry, errorKind, getCatalogueEntry, messageOf } from '$lib/api';
import type { PageLoad } from './$types';

// A barcode the catalogue lacks is an everyday outcome of scanning, not an error page:
// the product can still be entered by hand.
export const load: PageLoad = async ({ params }) => {
	let entry: CatalogueEntry | null = null;
	let failure: string | null = null;
	try {
		entry = await getCatalogueEntry(params.code);
	} catch (reason) {
		failure =
			errorKind(reason) === 'not-found'
				? `Open Food Facts has no entry for barcode ${params.code}.`
				: messageOf(reason);
	}
	return { code: params.code, entry, failure };
};
