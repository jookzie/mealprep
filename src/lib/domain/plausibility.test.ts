import { expect, test } from 'bun:test';
import type { Nutrients } from '../api/types';
import type { MacroFigures } from './macros';
import { type Figures, implausibilities } from './plausibility';

const nutella: MacroFigures = { energyKcal: 539, fatG: 30.9, proteinG: 6.3, carbohydratesG: 57.5 };

function figures(macros: MacroFigures, nutrients: Nutrients = {}, unit: Figures['unit'] = 'g') {
	return implausibilities({ unit, macros, nutrients });
}

test('a real label, rounded as printed, raises nothing', () => {
	expect(
		figures(nutella, {
			'energy-kj': 2252,
			'saturated-fat': 10.6,
			sugars: 56.3,
			fiber: 0,
			salt: 0.107,
		}),
	).toEqual([]);
});

test('nothing is doubted before the figures it needs are entered', () => {
	expect(
		figures({ energyKcal: null, fatG: null, proteinG: null, carbohydratesG: null }, { sugars: 9 }),
	).toEqual([]);
});

test('kJ typed into the kcal field is more than pure fat and disagrees with the macros', () => {
	const found = figures({ ...nutella, energyKcal: 2252 });
	expect(found).toHaveLength(2);
	expect(found[0]).toContain('more than pure fat');
	expect(found[1]).toContain('fat, protein and carbs add up to');
});

test('near-zero foods get a floor rather than a percentage', () => {
	// 0.3 kcal against an implied 0 is infinitely off in relative terms, and still right.
	expect(figures({ energyKcal: 0.3, fatG: 0, proteinG: 0, carbohydratesG: 0 })).toEqual([]);
});

test('alcohol, fibre and polyols count towards energy as the regulation says', () => {
	const beer: MacroFigures = { energyKcal: 43, fatG: 0, proteinG: 0.5, carbohydratesG: 3.1 };
	expect(figures(beer)).toHaveLength(1);
	expect(figures(beer, { alcohol: 3.9 })).toEqual([]);

	// Sugar-free sweets: 95 g carbs, nearly all of it polyols at 2.4 kcal/g.
	const sweets: MacroFigures = { energyKcal: 230, fatG: 0, proteinG: 0, carbohydratesG: 95 };
	expect(figures(sweets)).toHaveLength(1);
	expect(figures(sweets, { polyols: 93 })).toEqual([]);
});

test('a kJ figure that does not convert to the kcal one is doubted', () => {
	const found = figures(nutella, { 'energy-kj': 539 });
	expect(found).toHaveLength(1);
	expect(found[0]).toContain('kJ does not match');
});

test('a component larger than its whole is doubted, beyond rounding', () => {
	expect(figures({ ...nutella, fatG: 1 }, { 'saturated-fat': 1.1 })).not.toContainEqual(
		expect.stringContaining('Saturates'),
	);
	const found = figures(nutella, { 'saturated-fat': 32, sugars: 60 });
	expect(found.filter((doubt) => doubt.includes('more than total'))).toHaveLength(2);
});

test('nutrient keys match whatever shape the source wrote them in', () => {
	expect(figures(nutella, { saturated_fat: 32 })).toHaveLength(1);
});

test('grams cannot outweigh the 100 g they are in, but millilitres can', () => {
	const syrup: MacroFigures = { energyKcal: 440, fatG: 0, proteinG: 0, carbohydratesG: 110 };
	expect(figures(syrup, {}, 'g')).toHaveLength(1);
	expect(figures(syrup, {}, 'ml')).toEqual([]);
});
