import { expect, test } from 'bun:test';
import type { Macros } from '../api/types';
import { byRecency, formatSort, nextSort, parseSort, sortRows } from './sort';

function macros(energyKcal: number, proteinG = 0): Macros {
	return { energyKcal, proteinG, fatG: 0, carbohydratesG: 0 };
}

const products = [
	{ name: 'Oats', macros: macros(380, 13) },
	{ name: 'Chicken', macros: macros(120, 23) },
	{ name: 'Butter', macros: macros(740, 1) },
];

test('sorting by name is alphabetical, either way round', () => {
	expect(sortRows(products, { key: 'name', direction: 'asc' }).map((p) => p.name)).toEqual([
		'Butter',
		'Chicken',
		'Oats',
	]);
	expect(sortRows(products, { key: 'name', direction: 'desc' }).map((p) => p.name)).toEqual([
		'Oats',
		'Chicken',
		'Butter',
	]);
});

test('sorting by a macro uses that macro, not the name', () => {
	expect(sortRows(products, { key: 'proteinG', direction: 'desc' }).map((p) => p.name)).toEqual([
		'Chicken',
		'Oats',
		'Butter',
	]);
});

test('equal figures fall back to the name, so the order is reproducible', () => {
	const tied = [
		{ name: 'Beta', macros: macros(100) },
		{ name: 'Alpha', macros: macros(100) },
	];
	expect(sortRows(tied, { key: 'energyKcal', direction: 'desc' }).map((p) => p.name)).toEqual([
		'Alpha',
		'Beta',
	]);
});

test('the loaded array is never sorted in place', () => {
	const rows = [...products];
	sortRows(rows, { key: 'energyKcal', direction: 'asc' });
	expect(rows.map((p) => p.name)).toEqual(['Oats', 'Chicken', 'Butter']);
});

test('meals sort by label the same way products sort by name', () => {
	const meals = [
		{ label: 'Porridge', macros: macros(400) },
		{ label: 'Chicken rice', macros: macros(600) },
	];
	expect(sortRows(meals, { key: 'name', direction: 'asc' }).map((m) => m.label)).toEqual([
		'Chicken rice',
		'Porridge',
	]);
});

test('clicking the sorted column flips it; another column starts fresh', () => {
	expect(nextSort({ key: 'name', direction: 'asc' }, 'name')).toEqual({
		key: 'name',
		direction: 'desc',
	});
	// Nobody opens a food list to find the least protein first.
	expect(nextSort({ key: 'name', direction: 'asc' }, 'proteinG')).toEqual({
		key: 'proteinG',
		direction: 'desc',
	});
});

test('the sort survives a round trip through the URL', () => {
	const sort = { key: 'fatG', direction: 'desc' } as const;
	expect(parseSort(formatSort(sort))).toEqual(sort);
});

test('an unreadable sort parameter falls back rather than throwing', () => {
	expect(parseSort(null)).toEqual({ key: 'name', direction: 'asc' });
	expect(parseSort('nonsense:desc')).toEqual({ key: 'name', direction: 'asc' });
});

test('pickers put what you last touched at the top', () => {
	const rows = [
		{ id: 'old', updatedAt: '2026-01-01T00:00:00Z' },
		{ id: 'new', updatedAt: '2026-09-01T00:00:00Z' },
	];
	expect(byRecency(rows).map((row) => row.id)).toEqual(['new', 'old']);
});
