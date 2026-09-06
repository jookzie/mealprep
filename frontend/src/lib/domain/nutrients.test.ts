import { expect, test } from 'bun:test';
import { nutrientRows } from './nutrients';

test('no nutrients gives no rows', () => {
	expect(nutrientRows({})).toEqual([]);
	expect(nutrientRows(undefined)).toEqual([]);
});

test('the four macros are shown separately and never repeat here', () => {
	const rows = nutrientRows({ energyKcal: 380, fatG: 7, salt: 0.02 });
	expect(rows.map((row) => row.key)).toEqual(['salt']);
});

test('keys are humanised whatever shape the catalog used', () => {
	const rows = nutrientRows({ 'saturated-fat': 1, sugars_total: 2, dietaryFibre: 3 });
	expect(rows.map((row) => row.label)).toEqual(['Dietary fibre', 'Saturated fat', 'Sugars total']);
});

test('rows are sorted by label', () => {
	const rows = nutrientRows({ zinc: 1, calcium: 2, salt: 3 });
	expect(rows.map((row) => row.label)).toEqual(['Calcium', 'Salt', 'Zinc']);
});
