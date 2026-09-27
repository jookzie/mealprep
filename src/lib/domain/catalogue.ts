import type { CatalogueEntry, ProductDraft } from '../api/types';
import type { MacroFigures } from './macros';

/** What a product form starts from: a draft whose unknown macros are empty, not zero. */
export type ProductSeed = Omit<ProductDraft, 'macros'> & { macros: MacroFigures };

/**
 * A macro the catalogue lacks reads as zero on the wire. Seeding the form with that zero
 * would let it be saved unnoticed, so it starts empty and the form asks for it.
 */
export function seedFromEntry(entry: CatalogueEntry): ProductSeed {
	const macros: MacroFigures = { ...entry.macros };
	for (const key of entry.missingMacros) {
		macros[key] = null;
	}
	return {
		name: entry.name,
		brand: entry.brand,
		unit: entry.unit,
		macros,
		nutrients: entry.nutrients,
	};
}

/**
 * The retail barcodes on food (EAN-8, UPC-E, UPC-A, EAN-13, GTIN-14) are 8 to 14 digits,
 * sometimes printed in groups. Anything else typed into search is a name.
 */
export function barcodeIn(text: string): string | null {
	const digits = text.replace(/[\s-]/g, '');
	return /^\d{8,14}$/.test(digits) ? digits : null;
}
