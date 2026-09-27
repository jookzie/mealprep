import type { Category, CategoryScope } from '../api/types';

export type Categorised = { id: string; categoryId?: string };

export type CategoryGroup<T extends Categorised> = {
	key: string;
	label: string;
	category: Category | undefined;
	items: T[];
};

export const UNCATEGORISED_KEY = 'uncategorised';
export const REMOVED_KEY = 'removed';
export const REMOVED_LABEL = 'Category removed';

export const SCOPE_LABEL: Record<CategoryScope, string> = {
	meal: 'Meal categories',
	'day-plan': 'Day plan categories',
};

export function groupByCategory<T extends Categorised>(
	items: readonly T[],
	categories: readonly Category[],
): CategoryGroup<T>[] {
	const groups = new Map<string, CategoryGroup<T>>(
		categories.map((category) => [
			category.id,
			{ key: category.id, label: category.name, category, items: [] },
		]),
	);
	const uncategorised: T[] = [];
	const removed: T[] = [];

	for (const item of items) {
		if (item.categoryId === undefined) {
			uncategorised.push(item);
			continue;
		}
		const group = groups.get(item.categoryId);
		if (group) group.items.push(item);
		else removed.push(item);
	}

	const ordered = [...groups.values()];
	if (uncategorised.length > 0) {
		ordered.push({
			key: UNCATEGORISED_KEY,
			label: 'Uncategorised',
			category: undefined,
			items: uncategorised,
		});
	}
	if (removed.length > 0) {
		ordered.push({ key: REMOVED_KEY, label: REMOVED_LABEL, category: undefined, items: removed });
	}
	return ordered;
}

export function nonEmptyGroups<T extends Categorised>(
	groups: readonly CategoryGroup<T>[],
): CategoryGroup<T>[] {
	return groups.filter((group) => group.items.length > 0);
}

export function categoryName(
	item: Pick<Categorised, 'categoryId'>,
	categories: readonly Category[],
): string | undefined {
	if (item.categoryId === undefined) return undefined;
	return categories.find((category) => category.id === item.categoryId)?.name ?? REMOVED_LABEL;
}

export function countByCategory(items: readonly Categorised[]): Map<string, number> {
	const counts = new Map<string, number>();
	for (const item of items) {
		if (item.categoryId === undefined) continue;
		counts.set(item.categoryId, (counts.get(item.categoryId) ?? 0) + 1);
	}
	return counts;
}
