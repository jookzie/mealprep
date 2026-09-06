import { expect, test } from 'bun:test';
import type { Macros } from '../api/gen/types.gen';
import {
	addMacros,
	deviation,
	ratio,
	scaleMacros,
	servingMacros,
	sumMacros,
	ZERO_MACROS,
} from './macros';

const oats: Macros = { energyKcal: 380, fatG: 7, proteinG: 13, carbohydratesG: 60 };

test('summing nothing gives zero', () => {
	expect(sumMacros([])).toEqual(ZERO_MACROS);
});

test('macros add field by field', () => {
	expect(addMacros(oats, oats)).toEqual({
		energyKcal: 760,
		fatG: 14,
		proteinG: 26,
		carbohydratesG: 120,
	});
});

test('scaling multiplies every field', () => {
	expect(scaleMacros(oats, 0.5)).toEqual({
		energyKcal: 190,
		fatG: 3.5,
		proteinG: 6.5,
		carbohydratesG: 30,
	});
});

test('a serving is the per-100 figure scaled by amount', () => {
	// 40 g of a product listed per 100 g is 40% of its macros. Scaling is plain
	// float arithmetic, so the comparison tolerates the representation error that
	// formatting rounds away before anything reaches the screen.
	const serving = servingMacros(oats, 40);
	expect(serving.energyKcal).toBeCloseTo(152, 10);
	expect(serving.fatG).toBeCloseTo(2.8, 10);
	expect(serving.proteinG).toBeCloseTo(5.2, 10);
	expect(serving.carbohydratesG).toBeCloseTo(24, 10);
});

test('a zero serving contributes nothing', () => {
	expect(servingMacros(oats, 0)).toEqual(ZERO_MACROS);
});

test('deviation is signed: over target is positive, under is negative', () => {
	const target: Macros = { energyKcal: 400, fatG: 5, proteinG: 13, carbohydratesG: 70 };
	expect(deviation(oats, target)).toEqual({
		energyKcal: -20,
		fatG: 2,
		proteinG: 0,
		carbohydratesG: -10,
	});
});

test('a zero target has no ratio rather than an infinite one', () => {
	expect(ratio(150, 0)).toBe(0);
	expect(ratio(150, 300)).toBe(0.5);
});
