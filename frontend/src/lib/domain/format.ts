import type { Unit } from '../api/gen/types.gen';

// Energy is kcal, mass is grams, volume is millilitres, throughout (TG-4). Draft
// previews and saved figures both round here, so a value does not visibly shift
// when an unsaved total is replaced by the API's own.
const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
const oneDecimal = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

export function formatKcal(value: number): string {
	return `${whole.format(value)} kcal`;
}

export function formatGrams(value: number): string {
	return `${oneDecimal.format(value)} g`;
}

export function formatAmount(value: number, unit: Unit | undefined): string {
	return `${oneDecimal.format(value)} ${unit ?? 'g'}`;
}

export function formatUnitName(unit: Unit): string {
	return unit === 'ml' ? 'millilitres' : 'grams';
}

/** Deviations read as +/− so over and under target are distinguishable at a glance. */
export function formatSigned(value: number, kind: 'kcal' | 'g'): string {
	const magnitude =
		kind === 'kcal' ? whole.format(Math.abs(value)) : oneDecimal.format(Math.abs(value));
	const sign = value > 0 ? '+' : value < 0 ? '−' : '';
	return `${sign}${magnitude} ${kind}`;
}

export function formatPercent(value: number): string {
	return `${whole.format(value * 100)}%`;
}
