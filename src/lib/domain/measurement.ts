import type { Measurement, MeasurementKind, WeightPoint } from '$lib/api/types';
import type { ChartPoint } from './trend-chart';
import { datesInRange, type IsoDate, shiftDays } from './week';

type KindInfo = { kind: MeasurementKind; label: string; unit: 'cm' | '%' };

/** In the order the form offers them: waist first, because it carries the most evidence. */
export const KINDS: readonly KindInfo[] = [
	{ kind: 'waist', label: 'Waist', unit: 'cm' },
	{ kind: 'hips', label: 'Hips', unit: 'cm' },
	{ kind: 'chest', label: 'Chest', unit: 'cm' },
	{ kind: 'neck', label: 'Neck', unit: 'cm' },
	{ kind: 'arm', label: 'Arm', unit: 'cm' },
	{ kind: 'thigh', label: 'Thigh', unit: 'cm' },
	{ kind: 'body-fat', label: 'Body fat', unit: '%' },
	{ kind: 'height', label: 'Height', unit: 'cm' },
];

function info(kind: MeasurementKind): KindInfo {
	return KINDS.find((entry) => entry.kind === kind) ?? { kind, label: kind, unit: 'cm' };
}

export function kindLabel(kind: MeasurementKind): string {
	return info(kind).label;
}

export function kindUnit(kind: MeasurementKind): string {
	return info(kind).unit;
}

export function formatMeasurement(kind: MeasurementKind, value: number): string {
	const unit = kindUnit(kind);
	return unit === '%' ? `${value.toFixed(1)}%` : `${value.toFixed(1)} cm`;
}

/** Signed always, like the weight rate: the direction is the point of the number. */
export function formatChange(value: number, unit: string): string {
	const rounded = Math.abs(value) < 0.05 ? 0 : value;
	const sign = rounded > 0 ? '+' : rounded < 0 ? '−' : '±';
	return `${sign}${Math.abs(rounded).toFixed(1)} ${unit}`;
}

/** One kind's measurements in date order. */
export function ofKind(measurements: readonly Measurement[], kind: MeasurementKind): Measurement[] {
	return measurements
		.filter((measurement) => measurement.kind === kind)
		.toSorted((a, b) => a.date.localeCompare(b.date));
}

/** The latest value of each kind that has one, in the order of `KINDS`. */
export function latestByKind(measurements: readonly Measurement[]): Measurement[] {
	return KINDS.flatMap(({ kind }) => {
		const latest = ofKind(measurements, kind).at(-1);
		return latest ? [latest] : [];
	});
}

/**
 * One kind as a daily series for the chart: a dot on each measured day and a straight
 * line between them. Tape measurements come weekly at best, so smoothing them the way the
 * daily weigh-ins are would only invent days.
 */
export function measurementChart(
	measurements: readonly Measurement[],
	kind: MeasurementKind,
): ChartPoint[] {
	const entries = ofKind(measurements, kind);
	const first = entries.at(0);
	const last = entries.at(-1);
	if (!first || !last) return [];

	// The index of the last measurement on or before the day being drawn.
	let at = 0;
	return datesInRange(first.date, last.date).map((date) => {
		while (entries[at + 1] && entries[at + 1].date <= date) at += 1;
		const here = entries[at];
		if (here.date === date) return { date, value: here.value, line: here.value };
		return { date, line: interpolate(here, entries[at + 1], date) };
	});
}

function daysBetween(from: IsoDate, to: IsoDate): number {
	return (Date.parse(to) - Date.parse(from)) / 86_400_000;
}

function interpolate(from: Measurement, to: Measurement | undefined, date: IsoDate): number {
	if (!to) return from.value;
	const fraction = daysBetween(from.date, date) / daysBetween(from.date, to.date);
	return from.value + (to.value - from.value) * fraction;
}

export type CentralAdiposity = 'healthy' | 'increased' | 'high';

/**
 * Waist over height, with the band NICE (NG246) reads it in: under 0.5 healthy, up to 0.6
 * increased, 0.6 and over high. Absent without both measurements.
 */
export function waistToHeight(
	measurements: readonly Measurement[],
): { ratio: number; band: CentralAdiposity } | null {
	const waist = ofKind(measurements, 'waist').at(-1);
	const height = ofKind(measurements, 'height').at(-1);
	if (!waist || !height) return null;
	const ratio = waist.value / height.value;
	const band = ratio < 0.5 ? 'healthy' : ratio < 0.6 ? 'increased' : 'high';
	return { ratio, band };
}

export function adiposityLabel(band: CentralAdiposity): string {
	switch (band) {
		case 'healthy':
			return 'under 0.5';
		case 'increased':
			return '0.5 to 0.6';
		case 'high':
			return '0.6 or over';
	}
}

/** How far back the recomposition comparison looks. */
export const COMPARE_DAYS = 28;

/**
 * How much a kind changed over the last `days` before its latest measurement: from the
 * last value on or before the start of the window. Null without a measurement that old.
 */
export function measurementChange(
	measurements: readonly Measurement[],
	kind: MeasurementKind,
	days = COMPARE_DAYS,
): { change: number; from: IsoDate; to: IsoDate } | null {
	const entries = ofKind(measurements, kind);
	const latest = entries.at(-1);
	if (!latest) return null;
	const start = shiftDays(latest.date, -days);
	const earlier = entries.filter((entry) => entry.date <= start).at(-1);
	if (!earlier) return null;
	return { change: latest.value - earlier.value, from: earlier.date, to: latest.date };
}

/** How many days the weight trend may end before `to` and still stand for it. */
const TREND_SLACK_DAYS = 3;

/**
 * How far the weight trend moved between two dates, when the series covers both. The end
 * may fall a few days short, so a tape measure used the day after a weigh-in still compares.
 */
export function trendChange(
	points: readonly WeightPoint[],
	from: IsoDate,
	to: IsoDate,
): number | null {
	const start = points.find((point) => point.date === from);
	const earliestEnd = shiftDays(to, -TREND_SLACK_DAYS);
	const end = points.findLast((point) => point.date <= to && point.date >= earliestEnd);
	if (!start || !end) return null;
	return end.trend - start.trend;
}

/** Below this a change in either is within tape and scale noise over four weeks. */
const STEADY_WAIST_CM = 0.5;
const STEADY_WEIGHT_KG = 0.5;

/**
 * The one reading of waist against weight that is well established, or null when the pair
 * says nothing clear: a waist coming down while weight holds is fat lost and lean mass
 * kept; both coming down says the loss is coming off the middle.
 */
export function recompositionNote(waistCm: number, weightKg: number): string | null {
	if (waistCm > -STEADY_WAIST_CM) return null;
	if (Math.abs(weightKg) < STEADY_WEIGHT_KG) {
		return 'Waist down while weight holds: fat is going and muscle is staying.';
	}
	if (weightKg < 0) return 'Both coming down: the weight you are losing is coming off your waist.';
	return null;
}
