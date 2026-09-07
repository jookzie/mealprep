import type { Macros } from '../api/gen/types.gen';
import { type MacroKey, MACRO_KEYS } from './macros';

/**
 * The four macros on one radar.
 *
 * The axes are in different units — kcal against grams — so they cannot share a
 * linear scale. Each is expressed as a fraction of its own target, which is the
 * comparison the app exists to make and also puts every plan on the same footing.
 * With no targets set there is no shared denominator, so the largest value on each
 * axis across the compared plans stands in for one.
 *
 * The chart is a companion to the figures, never a replacement: a shape cannot be
 * read back to grams, so the numbers stay printed beside it.
 */
export type RadarPoint = {
	key: MacroKey;
	label: string;
	value: number;
	fraction: number;
	x: number;
	y: number;
};

/** Beyond this the polygon leaves the box, so the point clamps and the figure carries the rest. */
export const MAX_FRACTION = 1.5;

/**
 * Axis angles start at twelve o'clock and run clockwise, so the first macro is at
 * the top where the eye lands.
 */
export function axisAngle(index: number, count: number): number {
	return (index / count) * 2 * Math.PI - Math.PI / 2;
}

export function pointAt(
	index: number,
	count: number,
	radius: number,
	fraction: number,
): { x: number; y: number } {
	const angle = axisAngle(index, count);
	const r = (radius * Math.min(fraction, MAX_FRACTION)) / MAX_FRACTION;
	return { x: Math.cos(angle) * r, y: Math.sin(angle) * r };
}

export function radarPoints(
	macros: Macros,
	denominator: Macros,
	radius: number,
	labels: Record<MacroKey, string>,
): RadarPoint[] {
	return MACRO_KEYS.map((key, index) => {
		const target = denominator[key];
		// A denominator of zero has no meaningful fraction; the axis reads empty
		// rather than infinite.
		const fraction = target === 0 ? 0 : macros[key] / target;
		return {
			key,
			label: labels[key],
			value: macros[key],
			fraction,
			...pointAt(index, MACRO_KEYS.length, radius, fraction),
		};
	});
}

export function polygon(points: readonly { x: number; y: number }[]): string {
	return points.map((p) => `${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(' ');
}

/** The ring a fraction of 1 sits on — the target, when there is one. */
export function ringRadius(radius: number, fraction: number): number {
	return (radius * fraction) / MAX_FRACTION;
}

/** Standing in for targets: the largest value seen on each axis, so shapes stay comparable. */
export function maxima(all: readonly Macros[]): Macros {
	return {
		energyKcal: Math.max(0, ...all.map((m) => m.energyKcal)),
		proteinG: Math.max(0, ...all.map((m) => m.proteinG)),
		fatG: Math.max(0, ...all.map((m) => m.fatG)),
		carbohydratesG: Math.max(0, ...all.map((m) => m.carbohydratesG)),
	};
}
