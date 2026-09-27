import type { WeightPoint } from '$lib/api/types';
import type { IsoDate } from './week';

/** How many days each range shows. `0` is the whole history. */
export const RANGES = [30, 90, 365, 0] as const;
export type Range = (typeof RANGES)[number];

export function rangeLabel(range: Range): string {
	if (range === 0) return 'All';
	if (range === 365) return '1y';
	return `${range}d`;
}

/** The last `days` of the series, or all of it when `days` is 0. */
export function visible(points: readonly WeightPoint[], days: Range): WeightPoint[] {
	if (days === 0 || points.length <= days) return [...points];
	return points.slice(points.length - days);
}

/**
 * The narrowest window a chart will show, in kilograms.
 *
 * Without a floor, a week of identical weigh-ins would scale its own rounding noise to the
 * full height of the chart and read as violent change.
 */
export const MIN_SPAN_KG = 1;

/**
 * How much of the plot is left empty below the data and above it.
 *
 * Weight charts truncate the y-axis — starting at zero would flatten every change worth
 * seeing — but a tight crop exaggerates as badly as a zero baseline hides. Leaving half the
 * data's own span below it keeps the lowest point out of the bottom third of the plot,
 * which is the usual guard against reading a wobble as a collapse.
 */
export const FOOTROOM = 0.5;
export const HEADROOM = 0.15;

export type Domain = { min: number; max: number };

/** The value range the plot covers, padded so the data never fills it edge to edge. */
export function domain(points: readonly WeightPoint[]): Domain {
	const values = points.flatMap((point) =>
		point.kilograms === undefined ? [point.trend] : [point.trend, point.kilograms],
	);
	if (values.length === 0) return { min: 0, max: MIN_SPAN_KG };

	const low = Math.min(...values);
	const high = Math.max(...values);
	const span = Math.max(high - low, MIN_SPAN_KG);
	const min = low - span * FOOTROOM;
	// The floor is on the window, not on the data: a week of identical weigh-ins pads to
	// nothing, and padding a nothing still leaves a window too narrow to draw in.
	return { min, max: Math.max(high + span * HEADROOM, min + MIN_SPAN_KG) };
}

export type Box = { width: number; height: number; padX: number; padY: number };

export type Scale = {
	x: (index: number) => number;
	y: (value: number) => number;
};

/** Maps a point's position in the series and its value onto the drawing box. */
export function scale(count: number, { min, max }: Domain, box: Box): Scale {
	const plotWidth = box.width - box.padX * 2;
	const plotHeight = box.height - box.padY * 2;
	const steps = Math.max(count - 1, 1);
	const spread = max - min || 1;

	return {
		x: (index) => box.padX + (index / steps) * plotWidth,
		y: (value) => box.padY + (1 - (value - min) / spread) * plotHeight,
	};
}

/** The trend as an SVG path. It is continuous: every day between the ends has a value. */
export function trendPath(points: readonly WeightPoint[], scale: Scale): string {
	return points
		.map((point, index) => `${index === 0 ? 'M' : 'L'}${scale.x(index)} ${scale.y(point.trend)}`)
		.join(' ');
}

export type Dot = { x: number; y: number; date: IsoDate; kilograms: number };

/** The days that were actually weighed, which are the only days that get a mark. */
export function dots(points: readonly WeightPoint[], scale: Scale): Dot[] {
	const found: Dot[] = [];
	points.forEach((point, index) => {
		if (point.kilograms === undefined) return;
		found.push({
			x: scale.x(index),
			y: scale.y(point.kilograms),
			date: point.date,
			kilograms: point.kilograms,
		});
	});
	return found;
}

/**
 * Round values to label the y-axis with, inside the domain.
 *
 * Steps are chosen from a 1/2/5 progression so the labels read as weights someone would say
 * out loud rather than as whatever the domain happened to divide into.
 */
export function ticks({ min, max }: Domain, count = 3): number[] {
	const rough = (max - min) / count;
	if (!Number.isFinite(rough) || rough <= 0) return [];

	// The largest round step that still fits, not the smallest that covers: rounding a
	// narrow weight range upwards lands on one gridline for the whole plot.
	const magnitude = 10 ** Math.floor(Math.log10(rough));
	const step =
		[10, 5, 2, 1].map((factor) => factor * magnitude).find((candidate) => candidate <= rough) ??
		magnitude;

	const found: number[] = [];
	for (let value = Math.ceil(min / step) * step; value <= max; value += step) {
		found.push(Number(value.toFixed(6)));
	}
	return found;
}

/** The index of the point nearest an x position, for reading a value off the chart. */
export function nearest(points: readonly WeightPoint[], x: number, scale: Scale): number {
	if (points.length === 0) return -1;
	let best = 0;
	let bestDistance = Infinity;
	points.forEach((_, index) => {
		const distance = Math.abs(scale.x(index) - x);
		if (distance < bestDistance) {
			bestDistance = distance;
			best = index;
		}
	});
	return best;
}

export function formatKilograms(value: number): string {
	return `${value.toFixed(1)} kg`;
}

/**
 * The trend's slope, as a figure to act on.
 *
 * Signed always: "0.4 kg/week" alone would not say which way, and the direction is the
 * whole point of the number.
 */
export function formatRate(kilogramsPerWeek: number): string {
	const rounded = Math.abs(kilogramsPerWeek) < 0.05 ? 0 : kilogramsPerWeek;
	const sign = rounded > 0 ? '+' : rounded < 0 ? '−' : '±';
	return `${sign}${Math.abs(rounded).toFixed(1)} kg/week`;
}
