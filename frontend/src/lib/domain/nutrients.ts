import type { Nutrients } from '../api/gen/types.gen';
import { MACRO_KEYS } from './macros';

export type NutrientRow = {
	key: string;
	label: string;
	value: number;
};

const MACRO_NAMES: ReadonlySet<string> = new Set(MACRO_KEYS);

/**
 * Open Food Facts keys arrive in whatever shape the source used — `saturated-fat`,
 * `saturated_fat`, `saturatedFat` — and the set is open (PR-6), so the label is
 * derived rather than looked up in a table of known nutrients.
 */
function humanise(key: string): string {
	const words = key
		.replace(/([a-z0-9])([A-Z])/g, '$1 $2')
		.replace(/[-_]+/g, ' ')
		.trim()
		.toLowerCase();
	return words.charAt(0).toUpperCase() + words.slice(1);
}

/** The macros are shown on their own, so they never repeat inside the expansion. */
export function nutrientRows(nutrients: Nutrients | undefined): NutrientRow[] {
	if (!nutrients) return [];
	return Object.entries(nutrients)
		.filter(([key]) => !MACRO_NAMES.has(key))
		.map(([key, value]) => ({ key, label: humanise(key), value }))
		.sort((a, b) => a.label.localeCompare(b.label));
}
