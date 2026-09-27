import type { Nutrients } from '../api/types';
import { MACRO_KEYS } from './macros';

export type NutrientRow = {
	key: string;
	label: string;
	value: number;
	/**
	 * 1 for a component of a macro — saturates under fat, sugars under carbohydrate.
	 * The panel indents these rather than listing them as peers.
	 */
	depth: 0 | 1;
};

const MACRO_NAMES: ReadonlySet<string> = new Set(MACRO_KEYS);

/**
 * Regulation (EU) 1169/2011 Annex XV fixes the order a nutrition declaration is
 * presented in, and Open Food Facts is EU-origin, so its vocabulary is the one to
 * follow. The four macros are shown on their own (PR-9) and never repeat here, which
 * leaves the mandatory sub-items, salt, then the voluntary additions in the order the
 * annex lists them: mono-unsaturates, polyunsaturates, polyols, starch, fibre,
 * vitamins, minerals.
 */
const ORDER: readonly { key: string; label: string; depth: 0 | 1 }[] = [
	{ key: 'saturated-fat', label: 'of which saturates', depth: 1 },
	{ key: 'trans-fat', label: 'of which trans fats', depth: 1 },
	{ key: 'sugars', label: 'of which sugars', depth: 1 },
	{ key: 'salt', label: 'Salt', depth: 0 },
	{ key: 'monounsaturated-fat', label: 'Mono-unsaturates', depth: 1 },
	{ key: 'polyunsaturated-fat', label: 'Polyunsaturates', depth: 1 },
	{ key: 'polyols', label: 'Polyols', depth: 1 },
	{ key: 'starch', label: 'Starch', depth: 1 },
	{ key: 'fiber', label: 'Fibre', depth: 0 },
	{ key: 'sodium', label: 'Sodium', depth: 0 },
	{ key: 'vitamin-a', label: 'Vitamin A', depth: 0 },
	{ key: 'vitamin-d', label: 'Vitamin D', depth: 0 },
	{ key: 'vitamin-e', label: 'Vitamin E', depth: 0 },
	{ key: 'vitamin-k', label: 'Vitamin K', depth: 0 },
	{ key: 'vitamin-c', label: 'Vitamin C', depth: 0 },
	{ key: 'vitamin-b1', label: 'Thiamin', depth: 0 },
	{ key: 'vitamin-b2', label: 'Riboflavin', depth: 0 },
	{ key: 'vitamin-b6', label: 'Vitamin B6', depth: 0 },
	{ key: 'vitamin-b9', label: 'Folate', depth: 0 },
	{ key: 'vitamin-b12', label: 'Vitamin B12', depth: 0 },
	{ key: 'potassium', label: 'Potassium', depth: 0 },
	{ key: 'chloride', label: 'Chloride', depth: 0 },
	{ key: 'calcium', label: 'Calcium', depth: 0 },
	{ key: 'phosphorus', label: 'Phosphorus', depth: 0 },
	{ key: 'magnesium', label: 'Magnesium', depth: 0 },
	{ key: 'iron', label: 'Iron', depth: 0 },
	{ key: 'zinc', label: 'Zinc', depth: 0 },
	{ key: 'iodine', label: 'Iodine', depth: 0 },
	{ key: 'selenium', label: 'Selenium', depth: 0 },
];

const RANK = new Map(ORDER.map((entry, index) => [entry.key, index]));
const KNOWN = new Map(ORDER.map((entry) => [entry.key, entry]));

/** Different sources write the same nutrient as `saturated-fat`, `saturated_fat` or `saturatedFat`. */
export function canonical(key: string): string {
	return key
		.replace(/([a-z0-9])([A-Z])/g, '$1-$2')
		.replace(/[\s_]+/g, '-')
		.toLowerCase();
}

/**
 * Open Food Facts keys arrive in whatever shape the source used, and the set is open
 * (PR-6), so an unrecognised key still gets a readable label rather than being dropped.
 */
function humanise(key: string): string {
	const words = canonical(key).replace(/-+/g, ' ').trim();
	return words.charAt(0).toUpperCase() + words.slice(1);
}

/**
 * The macros are shown on their own, so they never repeat inside the expansion.
 * Known keys come first in the annex's order; because the vocabulary is open, the
 * rest follow alphabetically rather than being lost.
 */
export function nutrientRows(nutrients: Nutrients | undefined): NutrientRow[] {
	if (!nutrients) return [];

	const rows = Object.entries(nutrients)
		.filter(([key]) => !MACRO_NAMES.has(key))
		.map(([key, value]): NutrientRow => {
			const known = KNOWN.get(canonical(key));
			return {
				key,
				label: known?.label ?? humanise(key),
				value,
				depth: known?.depth ?? 0,
			};
		});

	return rows.sort((a, b) => {
		const rankA = RANK.get(canonical(a.key)) ?? Number.POSITIVE_INFINITY;
		const rankB = RANK.get(canonical(b.key)) ?? Number.POSITIVE_INFINITY;
		if (rankA !== rankB) return rankA - rankB;
		return a.label.localeCompare(b.label);
	});
}
