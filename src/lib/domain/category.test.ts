import { describe, expect, test } from 'bun:test';
import type { Category, DayPlan, Meal } from '../api/types';
import {
	categoryName,
	countByCategory,
	groupByCategory,
	nonEmptyGroups,
	REMOVED_KEY,
	UNCATEGORISED_KEY,
} from './category';

const ZERO = { energyKcal: 0, fatG: 0, proteinG: 0, carbohydratesG: 0 };
const STAMP = '2026-01-01T00:00:00Z';

function category(id: string, name: string, scope: Category['scope'] = 'day-plan'): Category {
	return { id, name, scope, createdAt: STAMP, updatedAt: STAMP };
}

function plan(id: string, label: string, categoryId?: string): DayPlan {
	return {
		id,
		label,
		categoryId,
		items: [],
		macros: ZERO,
		cost: { amount: 0, complete: true },
		createdAt: STAMP,
		updatedAt: STAMP,
	};
}

function meal(id: string, label: string, categoryId?: string): Meal {
	return {
		id,
		label,
		categoryId,
		servings: [],
		macros: ZERO,
		cost: { amount: 0, complete: true },
		createdAt: STAMP,
		updatedAt: STAMP,
	};
}

describe('groupByCategory', () => {
	test('keeps the order the categories arrived in, then uncategorised, then removed', () => {
		const groups = groupByCategory(
			[
				plan('1', 'Rest', 'a'),
				plan('2', 'Loose'),
				plan('3', 'Ghost', 'gone'),
				plan('4', 'Hard', 'b'),
			],
			[category('a', 'Cut'), category('b', 'Bulk')],
		);

		expect(groups.map((group) => group.key)).toEqual(['a', 'b', UNCATEGORISED_KEY, REMOVED_KEY]);
		expect(groups.map((group) => group.items.map((entry) => entry.id))).toEqual([
			['1'],
			['4'],
			['2'],
			['3'],
		]);
	});

	test('an item naming a deleted category is not folded into uncategorised', () => {
		const groups = groupByCategory([plan('1', 'Ghost', 'gone')], []);

		expect(groups).toHaveLength(1);
		expect(groups[0].key).toBe(REMOVED_KEY);
		expect(groups[0].label).toBe('Category removed');
	});

	test('an empty category is returned, and a picker drops it', () => {
		const groups = groupByCategory(
			[plan('1', 'Rest', 'a')],
			[category('a', 'Cut'), category('b', 'Bulk')],
		);

		expect(groups.map((group) => group.key)).toEqual(['a', 'b']);
		expect(nonEmptyGroups(groups).map((group) => group.key)).toEqual(['a']);
	});

	test('the order within a group is the order given', () => {
		const groups = groupByCategory(
			[plan('1', 'Second', 'a'), plan('2', 'First', 'a')],
			[category('a', 'Cut')],
		);

		expect(groups[0].items.map((entry) => entry.label)).toEqual(['Second', 'First']);
	});

	test('groups meals on the same terms as day plans', () => {
		const groups = groupByCategory(
			[meal('1', 'Porridge', 'a'), meal('2', 'Toast')],
			[category('a', 'Breakfast', 'meal')],
		);

		expect(groups.map((group) => group.label)).toEqual(['Breakfast', 'Uncategorised']);
	});
});

describe('categoryName', () => {
	test('names the category, the gap, or nothing at all', () => {
		const categories = [category('a', 'Cut')];

		expect(categoryName(plan('1', 'Rest', 'a'), categories)).toBe('Cut');
		expect(categoryName(plan('2', 'Ghost', 'gone'), categories)).toBe('Category removed');
		expect(categoryName(plan('3', 'Loose'), categories)).toBeUndefined();
	});
});

describe('countByCategory', () => {
	test('counts what each category holds and ignores the uncategorised', () => {
		const counts = countByCategory([
			meal('1', 'A', 'a'),
			meal('2', 'B', 'a'),
			meal('3', 'C', 'b'),
			meal('4', 'D'),
		]);

		expect(counts.get('a')).toBe(2);
		expect(counts.get('b')).toBe(1);
		expect(counts.size).toBe(2);
	});
});
