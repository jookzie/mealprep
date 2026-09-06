import type { Macros } from '../api/gen/types.gen';

/**
 * The four macros, in the order they are shown everywhere. Energy leads because it
 * is the figure the user reads first; the rest follow the requirements' wording.
 */
export const MACRO_KEYS = ['energyKcal', 'proteinG', 'fatG', 'carbohydratesG'] as const;

export type MacroKey = (typeof MACRO_KEYS)[number];

export const MACRO_LABELS: Record<MacroKey, string> = {
	energyKcal: 'Energy',
	proteinG: 'Protein',
	fatG: 'Fat',
	carbohydratesG: 'Carbs',
};

export const ZERO_MACROS: Macros = {
	energyKcal: 0,
	fatG: 0,
	proteinG: 0,
	carbohydratesG: 0,
};

export function addMacros(a: Macros, b: Macros): Macros {
	return {
		energyKcal: a.energyKcal + b.energyKcal,
		fatG: a.fatG + b.fatG,
		proteinG: a.proteinG + b.proteinG,
		carbohydratesG: a.carbohydratesG + b.carbohydratesG,
	};
}

export function sumMacros(values: readonly Macros[]): Macros {
	return values.reduce(addMacros, ZERO_MACROS);
}

export function scaleMacros(macros: Macros, factor: number): Macros {
	return {
		energyKcal: macros.energyKcal * factor,
		fatG: macros.fatG * factor,
		proteinG: macros.proteinG * factor,
		carbohydratesG: macros.carbohydratesG * factor,
	};
}

/**
 * A product's macros are per 100 of its unit, and a serving amount is in that same
 * unit, so a serving is the product scaled by amount/100.
 */
export function servingMacros(per100: Macros, amount: number): Macros {
	return scaleMacros(per100, amount / 100);
}

/** Signed distance from the target. Informational only: nothing acts on it (TG-3). */
export function deviation(actual: Macros, target: Macros): Macros {
	return {
		energyKcal: actual.energyKcal - target.energyKcal,
		fatG: actual.fatG - target.fatG,
		proteinG: actual.proteinG - target.proteinG,
		carbohydratesG: actual.carbohydratesG - target.carbohydratesG,
	};
}

/** A target of zero has no meaningful ratio, so it reads as zero rather than infinity. */
export function ratio(actual: number, target: number): number {
	return target === 0 ? 0 : actual / target;
}

export function ratios(actual: Macros, target: Macros): Macros {
	return {
		energyKcal: ratio(actual.energyKcal, target.energyKcal),
		fatG: ratio(actual.fatG, target.fatG),
		proteinG: ratio(actual.proteinG, target.proteinG),
		carbohydratesG: ratio(actual.carbohydratesG, target.carbohydratesG),
	};
}
