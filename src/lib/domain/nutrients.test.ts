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

test('unknown keys are humanised whatever shape the catalog used', () => {
	const rows = nutrientRows({ caffeine_total: 2, alcoholContent: 3 });
	expect(rows.map((row) => row.label)).toEqual(['Alcohol content', 'Caffeine total']);
});

test('known nutrients follow Annex XV rather than the alphabet', () => {
	// Alphabetically this is salt, saturated fat, sodium, sugars — which interleaves a
	// component of fat with a component of carbohydrate. The regulation does not.
	const rows = nutrientRows({ sodium: 0.1, salt: 0.25, sugars: 12, 'saturated-fat': 3 });
	expect(rows.map((row) => row.key)).toEqual(['saturated-fat', 'sugars', 'salt', 'sodium']);
});

test('components of a macro are labelled and indented as the annex writes them', () => {
	const rows = nutrientRows({ 'saturated-fat': 3, sugars: 12, salt: 0.25 });
	expect(rows.map((row) => [row.label, row.depth])).toEqual([
		['of which saturates', 1],
		['of which sugars', 1],
		['Salt', 0],
	]);
});

test('a key is recognised however the source spelled it', () => {
	for (const key of ['saturated-fat', 'saturated_fat', 'saturatedFat']) {
		expect(nutrientRows({ [key]: 1 })[0].label).toBe('of which saturates');
	}
});

test('unknown keys follow the known ones, alphabetically', () => {
	const rows = nutrientRows({ zzz: 1, aaa: 2, salt: 3 });
	expect(rows.map((row) => row.key)).toEqual(['salt', 'aaa', 'zzz']);
});
