import type { Macros } from '../api/gen/types.gen';

/** The columns a food list is worth sorting by: its name, and each of the four macros. */
export type SortKey = 'name' | 'energyKcal' | 'proteinG' | 'fatG' | 'carbohydratesG';
export type SortDirection = 'asc' | 'desc';

export type Sort = { key: SortKey; direction: SortDirection };

export const DEFAULT_SORT: Sort = { key: 'name', direction: 'asc' };

type Sortable = { macros: Macros } & ({ name: string } | { label: string });

function nameOf(row: Sortable): string {
	return 'name' in row ? row.name : row.label;
}

/**
 * Sorting is a pure comparator over a copy: the caller's array is loaded data and is
 * never mutated in place.
 */
export function sortRows<T extends Sortable>(rows: readonly T[], sort: Sort): T[] {
	const sign = sort.direction === 'asc' ? 1 : -1;
	return [...rows].sort((a, b) => {
		if (sort.key === 'name') return sign * nameOf(a).localeCompare(nameOf(b));
		const difference = a.macros[sort.key] - b.macros[sort.key];
		// Equal figures fall back to the name, so the order is stable and reproducible
		// rather than dependent on how the rows happened to arrive.
		return difference === 0 ? nameOf(a).localeCompare(nameOf(b)) : sign * difference;
	});
}

/**
 * Clicking the column already sorted flips it; clicking another starts it ascending,
 * except the macros, which start high — nobody sorts a food list to find the least
 * protein first.
 */
export function nextSort(current: Sort, key: SortKey): Sort {
	if (current.key === key) {
		return { key, direction: current.direction === 'asc' ? 'desc' : 'asc' };
	}
	return { key, direction: key === 'name' ? 'asc' : 'desc' };
}

export function parseSort(value: string | null): Sort {
	const [key, direction] = (value ?? '').split(':');
	const keys: SortKey[] = ['name', 'energyKcal', 'proteinG', 'fatG', 'carbohydratesG'];
	if (!keys.includes(key as SortKey)) return DEFAULT_SORT;
	return { key: key as SortKey, direction: direction === 'desc' ? 'desc' : 'asc' };
}

export function formatSort(sort: Sort): string {
	return `${sort.key}:${sort.direction}`;
}

/**
 * Pickers list everything alphabetically, which puts what you used a minute ago
 * wherever the alphabet left it. Most-recently-touched first is the ordering every
 * food app reaches for, and `updatedAt` already carries it.
 */
export function byRecency<T extends { updatedAt: string }>(rows: readonly T[]): T[] {
	return [...rows].sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
}
