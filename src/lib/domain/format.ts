import type { Unit } from '../api/types';

// Energy is kcal, mass is grams, volume is millilitres, throughout (TG-4). Overview
// screens show whole numbers; an editor passes `precise` and keeps a tenth, so what is
// being typed stays exact while the screens that are only read stay uncluttered.
const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
const oneDecimal = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

export function formatKcal(value: number): string {
	return `${whole.format(value)} kcal`;
}

function grams(precise: boolean): Intl.NumberFormat {
	return precise ? oneDecimal : whole;
}

export function formatGrams(value: number, precise = false): string {
	return `${grams(precise).format(value)} g`;
}

export function formatAmount(value: number, unit: Unit | undefined, precise = false): string {
	return `${grams(precise).format(value)} ${unit ?? 'g'}`;
}

export function formatUnitName(unit: Unit): string {
	return unit === 'ml' ? 'millilitres' : 'grams';
}

/** Deviations read as +/− so over and under target are distinguishable at a glance. */
export function formatSigned(value: number, kind: 'kcal' | 'g', precise = false): string {
	const magnitude =
		kind === 'kcal' ? whole.format(Math.abs(value)) : grams(precise).format(Math.abs(value));
	const sign = value > 0 ? '+' : value < 0 ? '−' : '';
	return `${sign}${magnitude} ${kind}`;
}

/**
 * Cost carries no currency symbol: the application never asks which currency the
 * user keeps, so the figure is shown as the plain number they typed, at the two
 * decimals money is written in.
 */
const money = new Intl.NumberFormat(undefined, {
	minimumFractionDigits: 2,
	maximumFractionDigits: 2,
});

export function formatCost(value: number): string {
	return money.format(value);
}

export function formatPercent(value: number): string {
	return `${whole.format(value * 100)}%`;
}
