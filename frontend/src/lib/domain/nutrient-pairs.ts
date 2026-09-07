import type { Nutrients } from '../api/gen/types.gen';
import { parseDecimal } from './number';

/**
 * A product's nutrient set is open (PR-6), so the form edits it as free key/value
 * pairs and converts at the boundary rather than binding to a fixed set of fields.
 */
export type NutrientPair = {
	/**
	 * A row's own identity, so the editor can key on it. The nutrient's name is what
	 * the user is typing, so it cannot serve as one, and the index stops being an
	 * identity the moment a row can be removed from the middle.
	 */
	id: string;
	key: string;
	value: string;
};

export function newPair(): NutrientPair {
	return { id: crypto.randomUUID(), key: '', value: '' };
}

export function toPairs(nutrients: Nutrients | undefined): NutrientPair[] {
	return Object.entries(nutrients ?? {}).map(([key, value]) => ({
		id: crypto.randomUUID(),
		key,
		value: String(value),
	}));
}

/** Blank rows and unparsable values are dropped rather than sent as NaN. */
export function fromPairs(pairs: readonly NutrientPair[]): Nutrients {
	const nutrients: Nutrients = {};
	for (const pair of pairs) {
		const key = pair.key.trim();
		const value = parseDecimal(pair.value);
		if (key !== '' && value !== null) {
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
