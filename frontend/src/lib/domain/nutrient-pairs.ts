import type { Nutrients } from '../api/gen/types.gen';

/**
 * A product's nutrient set is open (PR-6), so the form edits it as free key/value
 * pairs and converts at the boundary rather than binding to a fixed set of fields.
 */
export type NutrientPair = { key: string; value: string };

export function toPairs(nutrients: Nutrients | undefined): NutrientPair[] {
	return Object.entries(nutrients ?? {}).map(([key, value]) => ({ key, value: String(value) }));
}

/** Blank rows and unparsable values are dropped rather than sent as NaN. */
export function fromPairs(pairs: readonly NutrientPair[]): Nutrients {
	const nutrients: Nutrients = {};
	for (const pair of pairs) {
		const key = pair.key.trim();
		const value = Number(pair.value);
		if (key !== '' && pair.value.trim() !== '' && Number.isFinite(value)) {
			nutrients[key] = value;
		}
	}
	return nutrients;
}

export function duplicateKeys(pairs: readonly NutrientPair[]): Set<string> {
	const seen = new Set<string>();
	const duplicates = new Set<string>();
	for (const pair of pairs) {
		const key = pair.key.trim().toLowerCase();
		if (key === '') continue;
		if (seen.has(key)) duplicates.add(key);
		seen.add(key);
	}
	return duplicates;
}
