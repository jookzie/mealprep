import type { Band } from '$lib/api/types';
import type { Domain, Scale } from './weight';
import type { IsoDate } from './week';

/**
 * One day of a chart that draws raw readings as dots, a smoothed or interpolated figure as
 * a line, and optionally a normal range behind both. Every part is optional per day: a
 * reading can be missing, a rolling average can lack enough readings, and a range needs
 * weeks of history before it exists at all.
 */
export type ChartPoint = {
	date: IsoDate;
	value?: number;
	line?: number;
	band?: Band;
};

/**
 * How much of the plot is left empty above and below the data, as a share of its span.
 *
 * Unlike weight these figures have no zero worth showing either, so the axis is cropped
 * to the data with the same margin on both sides.
 */
export const MARGIN = 0.2;

/** The value range the plot covers, padded and never narrower than `minSpan`. */
export function chartDomain(points: readonly ChartPoint[], minSpan: number): Domain {
	const values = points.flatMap((point) => [
		...(point.value === undefined ? [] : [point.value]),
		...(point.line === undefined ? [] : [point.line]),
		...(point.band === undefined ? [] : [point.band.low, point.band.high]),
	]);
	if (values.length === 0) return { min: 0, max: minSpan };

	const low = Math.min(...values);
	const high = Math.max(...values);
	const span = Math.max(high - low, minSpan);
	const middle = (low + high) / 2;
	return {
		min: middle - span / 2 - span * MARGIN,
		max: middle + span / 2 + span * MARGIN,
	};
}

/** Splits the series into runs of consecutive days that all have the given part. */
function runs<T>(
	points: readonly ChartPoint[],
	part: (point: ChartPoint) => T | undefined,
): { index: number; value: T }[][] {
	const found: { index: number; value: T }[][] = [];
	let current: { index: number; value: T }[] = [];
	points.forEach((point, index) => {
		const value = part(point);
		if (value === undefined) {
			if (current.length > 0) found.push(current);
			current = [];
		} else {
			current.push({ index, value });
		}
	});
	if (current.length > 0) found.push(current);
	return found;
}

/**
 * The line as an SVG path, broken wherever a day has no value: a gap in the readings is
 * shown as a gap, not bridged by a line that implies data nobody recorded.
 */
export function linePath(points: readonly ChartPoint[], scale: Scale): string {
	return runs(points, (point) => point.line)
		.map((run) =>
			run
				.map(({ index, value }, step) => `${step === 0 ? 'M' : 'L'}${scale.x(index)} ${scale.y(value)}`)
				.join(' '),
		)
		.join(' ');
}

/** The normal range as closed SVG shapes, one per unbroken run of days that have it. */
export function bandPath(points: readonly ChartPoint[], scale: Scale): string {
	return runs(points, (point) => point.band)
		.map((run) => {
			const upper = run.map(({ index, value }) => `${scale.x(index)} ${scale.y(value.high)}`);
			const lower = run
				.toReversed()
				.map(({ index, value }) => `${scale.x(index)} ${scale.y(value.low)}`);
			return `M${upper.join(' L')} L${lower.join(' L')} Z`;
		})
		.join(' ');
}

export type ValueDot = { x: number; y: number; date: IsoDate; value: number };

/** The days that have a reading, which are the only days that get a mark. */
export function valueDots(points: readonly ChartPoint[], scale: Scale): ValueDot[] {
	return runs(points, (point) => point.value).flatMap((run) =>
		run.map(({ index, value }) => ({
			x: scale.x(index),
			y: scale.y(value),
			date: points[index].date,
			value,
		})),
	);
}
