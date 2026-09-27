import type { Nutrients, Unit } from '../api/types';
import { formatGrams, formatKcal } from './format';
import { impliedEnergyKcal, type MacroFigures } from './macros';
import { canonical } from './nutrients';

/**
 * Catalogue figures are crowd-sourced and hand-entered ones are typed, so both can carry
 * the classic slips: kJ in the kcal field, a decimal in the wrong place, a component
 * larger than its whole. Each check below only fires on a disagreement too large for
 * label rounding to explain. Like the targets hint (TG-3), none of them blocks a save:
 * the package in the user's hand is the authority, not these rules.
 */
export type Figures = {
	unit: Unit;
	macros: MacroFigures;
	nutrients: Nutrients;
};

/** Pure fat, the most energy-dense thing food can be. */
const MAX_KCAL = 900;

const KJ_PER_KCAL = 4.184;

/**
 * Regulation (EU) 1169/2011 Annex XIV. The declared carbohydrate includes polyols, which
 * yield 2.4 kcal/g rather than 4, and energy also counts fibre and alcohol, which the
 * three macros leave out.
 */
const POLYOL_DISCOUNT_KCAL = 4 - 2.4;
const FIBRE_KCAL = 2;
const ALCOHOL_KCAL = 7;

/**
 * EU tolerance guidance lets a declared value sit 20% off the analysed one, so an energy
 * figure is only doubted beyond that, and beyond 20 kcal for foods near zero.
 */
const ENERGY_SLACK_SHARE = 0.2;
const ENERGY_SLACK_KCAL = 20;

/** kJ and kcal are rounded apart, and some labels convert at 4.2. */
const KJ_SLACK_SHARE = 0.05;
const KJ_SLACK = 10;

/** A component and its whole are each rounded to a tenth. */
const COMPONENT_SLACK_G = 0.1;

/** Open Food Facts' own threshold: the figures of 100 g can outweigh it only by rounding. */
const MAX_GRAMS = 105;

const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

function nutrient(nutrients: Nutrients, key: string): number | undefined {
	for (const [name, value] of Object.entries(nutrients)) {
		if (canonical(name) === key) return value;
	}
	return undefined;
}

function outside(actual: number, expected: number, share: number, floor: number): boolean {
	return Math.abs(actual - expected) > Math.max(floor, share * Math.max(actual, expected));
}

/** Says, for people, which of the given per-100 figures cannot all be right. */
export function implausibilities({ unit, macros, nutrients }: Figures): string[] {
	const found: string[] = [];
	const { energyKcal, fatG, proteinG, carbohydratesG } = macros;
	const fibre = nutrient(nutrients, 'fiber') ?? 0;

	if (energyKcal !== null && energyKcal > MAX_KCAL) {
		found.push(
			`${formatKcal(energyKcal)} per 100 ${unit} is more than pure fat. It may be the kJ figure.`,
		);
	}

	if (energyKcal !== null && fatG !== null && proteinG !== null && carbohydratesG !== null) {
		const expected =
			impliedEnergyKcal({ energyKcal, fatG, proteinG, carbohydratesG }) -
			POLYOL_DISCOUNT_KCAL * (nutrient(nutrients, 'polyols') ?? 0) +
			FIBRE_KCAL * fibre +
			ALCOHOL_KCAL * (nutrient(nutrients, 'alcohol') ?? 0);
		if (outside(energyKcal, expected, ENERGY_SLACK_SHARE, ENERGY_SLACK_KCAL)) {
			found.push(
				`Energy is ${formatKcal(energyKcal)}, but fat, protein and carbs add up to about ${formatKcal(expected)}.`,
			);
		}
	}

	const kj = nutrient(nutrients, 'energy-kj');
	if (kj !== undefined && energyKcal !== null) {
		const expected = energyKcal * KJ_PER_KCAL;
		if (outside(kj, expected, KJ_SLACK_SHARE, KJ_SLACK)) {
			found.push(
				`${whole.format(kj)} kJ does not match ${formatKcal(energyKcal)}, which is about ${whole.format(expected)} kJ.`,
			);
		}
	}

	const components = [
		{ part: nutrient(nutrients, 'saturated-fat'), name: 'Saturates', whole: fatG, of: 'fat' },
		{ part: nutrient(nutrients, 'sugars'), name: 'Sugars', whole: carbohydratesG, of: 'carbs' },
	];
	for (const { part, name, whole: total, of } of components) {
		if (part !== undefined && total !== null && part > total + COMPONENT_SLACK_G) {
			found.push(`${name} (${formatGrams(part)}) are more than total ${of} (${formatGrams(total)}).`);
		}
	}

	// A millilitre of syrup or oil weighs more or less than a gram, so volume has no limit.
	if (unit === 'g' && fatG !== null && proteinG !== null && carbohydratesG !== null) {
		const grams = fatG + proteinG + carbohydratesG + fibre + (nutrient(nutrients, 'salt') ?? 0);
		if (grams > MAX_GRAMS) {
			found.push(
				`Fat, protein, carbs, fibre and salt add up to ${formatGrams(grams)}, more than the 100 g they are measured in.`,
			);
		}
	}

	return found;
}
