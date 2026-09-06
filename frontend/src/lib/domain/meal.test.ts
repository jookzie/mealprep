import { expect, test } from 'bun:test';
import type { DayPlan, Meal, Product } from '../api/gen/types.gen';
import { toDayPlanDraft } from './day-plan';
import { draftMacros, servingRows, toMealDraft } from './meal';

const oats: Product = {
	id: 'p1',
	name: 'Oats',
	unit: 'g',
	macros: { energyKcal: 380, fatG: 7, proteinG: 13, carbohydratesG: 60 },
	nutrients: {},
	createdAt: '2026-08-01T00:00:00Z',
	updatedAt: '2026-08-01T00:00:00Z',
};

const milk: Product = {
	...oats,
	id: 'p2',
	name: 'Milk',
	unit: 'ml',
	macros: { energyKcal: 47, fatG: 1.5, proteinG: 3.4, carbohydratesG: 4.8 },
};

test('a serving resolves to its product and unit', () => {
	const [row] = servingRows([{ productId: 'p2', amount: 250 }], [oats, milk]);
	expect(row.product?.name).toBe('Milk');
	expect(row.unit).toBe('ml');
	expect(row.macros.energyKcal).toBeCloseTo(117.5, 10);
});

test('a serving whose product was deleted still produces a row', () => {
	// Products are soft-deleted and then never returned, so this join can miss.
	const [row] = servingRows([{ productId: 'gone', amount: 100 }], [oats]);
	expect(row.productId).toBe('gone');
	expect(row.product).toBeUndefined();
	expect(row.unit).toBeUndefined();
	expect(row.macros.energyKcal).toBe(0);
});

test('a draft total is the sum of its rows', () => {
	const rows = servingRows(
		[
			{ productId: 'p1', amount: 100 },
			{ productId: 'p2', amount: 200 },
		],
		[oats, milk],
	);
	expect(draftMacros(rows).energyKcal).toBeCloseTo(474, 10);
});

test('a meal maps to a draft carrying its label and servings', () => {
	const meal: Meal = {
		id: 'm1',
		label: 'Porridge',
		servings: [{ productId: 'p1', amount: 80 }],
		macros: { energyKcal: 304, fatG: 5.6, proteinG: 10.4, carbohydratesG: 48 },
		createdAt: '2026-08-01T00:00:00Z',
		updatedAt: '2026-08-01T00:00:00Z',
	};
	expect(toMealDraft(meal)).toEqual({
		label: 'Porridge',
		servings: [{ productId: 'p1', amount: 80 }],
	});
});

test('a day plan reads back as meals but writes as ids', () => {
	const plan: DayPlan = {
		id: 'd1',
		label: 'Training day',
		meals: [
			{
				id: 'm1',
				label: 'Porridge',
				macros: { energyKcal: 1, fatG: 1, proteinG: 1, carbohydratesG: 1 },
			},
			{
				id: 'm2',
				label: 'Chili',
				macros: { energyKcal: 2, fatG: 2, proteinG: 2, carbohydratesG: 2 },
			},
		],
		macros: { energyKcal: 3, fatG: 3, proteinG: 3, carbohydratesG: 3 },
		createdAt: '2026-08-01T00:00:00Z',
		updatedAt: '2026-08-01T00:00:00Z',
	};
	expect(toDayPlanDraft(plan)).toEqual({ label: 'Training day', mealIds: ['m1', 'm2'] });
});
