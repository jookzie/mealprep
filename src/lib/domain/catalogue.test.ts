import { expect, test } from 'bun:test';
import type { CatalogueEntry } from '../api/types';
import { barcodeIn, seedFromEntry } from './catalogue';

const entry: CatalogueEntry = {
	code: '5000112637922',
	name: 'Cola',
	unit: 'ml',
	macros: { energyKcal: 42, fatG: 0, proteinG: 0, carbohydratesG: 10.6 },
	nutrients: { sugars: 10.6 },
	brand: 'coca-cola',
	missingMacros: [],
	source: 'community',
	warnings: [],
};

test('a complete entry seeds the form with its figures', () => {
	expect(seedFromEntry(entry)).toEqual({
		name: 'Cola',
		brand: 'coca-cola',
		unit: 'ml',
		macros: { energyKcal: 42, fatG: 0, proteinG: 0, carbohydratesG: 10.6 },
		nutrients: { sugars: 10.6 },
	});
});

test('a missing macro seeds empty rather than as the zero it arrived as', () => {
	const seed = seedFromEntry({ ...entry, missingMacros: ['proteinG', 'fatG'] });
	expect(seed.macros).toEqual({ energyKcal: 42, fatG: null, proteinG: null, carbohydratesG: 10.6 });
});

test('retail barcode lengths are barcodes, printed groups included', () => {
	expect(barcodeIn('3017620422003')).toBe('3017620422003');
	expect(barcodeIn(' 3 017620 422003 ')).toBe('3017620422003');
	expect(barcodeIn('96385074')).toBe('96385074');
});

test('names and other numbers are not barcodes', () => {
	expect(barcodeIn('skyr')).toBeNull();
	expect(barcodeIn('1234567')).toBeNull();
	expect(barcodeIn('123456789012345')).toBeNull();
	expect(barcodeIn('7up 330ml')).toBeNull();
});
