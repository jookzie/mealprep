import { expect, test } from 'bun:test';
import type { Product } from '../api/types';
import { addCost, productCost, scaleCost, servingCost, sumCosts, ZERO_COST } from './cost';

const oats: Product = {
	id: 'p1',
	name: 'Oats',
	unit: 'g',
	macros: { energyKcal: 380, fatG: 7, proteinG: 13, carbohydratesG: 60 },
	nutrients: {},
	cost: 0.25,
	createdAt: '2026-08-01T00:00:00Z',
	updatedAt: '2026-08-01T00:00:00Z',
};

const unpriced: Product = { ...oats, id: 'p2', name: 'Salt', cost: undefined };

test('summing nothing costs nothing and is not missing anything', () => {
	expect(sumCosts([])).toEqual(ZERO_COST);
});

test('costs add, and one incomplete figure makes the sum incomplete', () => {
	expect(addCost({ amount: 1.5, complete: true }, { amount: 2, complete: true })).toEqual({
		amount: 3.5,
		complete: true,
	});
	expect(addCost({ amount: 1.5, complete: true }, { amount: 0, complete: false })).toEqual({
		amount: 1.5,
		complete: false,
	});
});

test('scaling keeps the claim about the figure', () => {
	expect(scaleCost({ amount: 3, complete: false }, 1 / 3)).toEqual({ amount: 1, complete: false });
});

test('a serving is the price per 100 units scaled by its amount', () => {
	expect(servingCost(oats, 250)).toEqual({ amount: 0.625, complete: true });
});

test('an unpriced product adds zero and marks the total incomplete', () => {
	expect(servingCost(unpriced, 250)).toEqual({ amount: 0, complete: false });
});

test('a deleted product costs as little as it is knowable', () => {
	// The join can miss, because products are soft-deleted and then never returned.
	expect(servingCost(undefined, 250)).toEqual({ amount: 0, complete: false });
});

test('a product without a price has no cost of its own to show', () => {
	expect(productCost(oats)).toEqual({ amount: 0.25, complete: true });
	expect(productCost(unpriced)).toBeNull();
});
