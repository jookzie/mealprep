import type { MacroKey } from '$lib/domain/macros';

/**
 * The macro a colour belongs to, resolved to its token. These are identity colours,
 * never status colours: the same hue means protein on every screen, and no hue in the
 * set carries a verdict. Energy is the sum of the other three rather than a fourth
 * series, so it stays neutral and earns its prominence from size and position.
 */
export const MACRO_HUE: Record<MacroKey, string> = {
	energyKcal: 'var(--macro-energy)',
	proteinG: 'var(--macro-protein)',
	fatG: 'var(--macro-fat)',
	carbohydratesG: 'var(--macro-carbs)',
};

/** Grams for the three macros, kcal for energy — units are derived, never chosen (TG-4). */
export const MACRO_UNIT: Record<MacroKey, 'kcal' | 'g'> = {
	energyKcal: 'kcal',
	proteinG: 'g',
	fatG: 'g',
	carbohydratesG: 'g',
};
