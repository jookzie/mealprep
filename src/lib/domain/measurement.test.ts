import { describe, expect, test } from 'bun:test';
import type { Measurement, MeasurementKind, WeightPoint } from '$lib/api/types';
import {
	formatChange,
	latestByKind,
	measurementChange,
	measurementChart,
	recompositionNote,
	trendChange,
	waistToHeight,
} from './measurement';

function measurement(date: string, kind: MeasurementKind, value: number): Measurement {
	return { date, kind, value, createdAt: '', updatedAt: '' };
}

describe('measurementChart', () => {
	test('draws a dot per measurement and a straight line between them', () => {
		const points = measurementChart(
			[
				measurement('2026-09-01', 'waist', 90),
				measurement('2026-09-05', 'waist', 88),
				measurement('2026-09-03', 'hips', 100),
			],
			'waist',
		);

		expect(points).toHaveLength(5);
		expect(points.filter((point) => point.value !== undefined)).toHaveLength(2);
		expect(points[2].line).toBeCloseTo(89);
		expect(points[4]).toEqual({ date: '2026-09-05', value: 88, line: 88 });
	});

	test('a kind never measured has nothing to draw', () => {
		expect(measurementChart([], 'waist')).toEqual([]);
	});
});

describe('latestByKind', () => {
	test('keeps the most recent of each kind, waist first', () => {
		const latest = latestByKind([
			measurement('2026-09-01', 'height', 180),
			measurement('2026-09-01', 'waist', 90),
			measurement('2026-09-08', 'waist', 89),
		]);

		expect(latest.map((entry) => [entry.kind, entry.value])).toEqual([
			['waist', 89],
			['height', 180],
		]);
	});
});

describe('waistToHeight', () => {
	test('reads the latest waist over the latest height in the NICE bands', () => {
		const ratio = waistToHeight([
			measurement('2026-09-01', 'height', 180),
			measurement('2026-09-08', 'waist', 92),
		]);

		expect(ratio?.ratio).toBeCloseTo(0.511, 3);
		expect(ratio?.band).toBe('increased');
		expect(waistToHeight([measurement('2026-09-08', 'waist', 92)])).toBeNull();
	});
});

describe('changes over four weeks', () => {
	test('compares the latest measurement with the last one at least four weeks older', () => {
		const change = measurementChange(
			[
				measurement('2026-08-01', 'waist', 92),
				measurement('2026-08-10', 'waist', 91),
				measurement('2026-09-10', 'waist', 89.5),
			],
			'waist',
		);

		expect(change).toEqual({ change: -1.5, from: '2026-08-10', to: '2026-09-10' });
		expect(measurementChange([measurement('2026-09-10', 'waist', 89)], 'waist')).toBeNull();
	});

	test('the weight trend change may end a few days before the tape measure', () => {
		const points: WeightPoint[] = [
			{ date: '2026-08-10', trend: 80 },
			{ date: '2026-09-08', trend: 80.4 },
		];

		expect(trendChange(points, '2026-08-10', '2026-09-10')).toBeCloseTo(0.4);
		expect(trendChange(points, '2026-08-10', '2026-09-20')).toBeNull();
		expect(trendChange(points, '2026-08-01', '2026-09-10')).toBeNull();
	});

	test('changes are signed', () => {
		expect(formatChange(-1.5, 'cm')).toBe('−1.5 cm');
		expect(formatChange(0.01, 'kg')).toBe('±0.0 kg');
	});
});

describe('recompositionNote', () => {
	test('names only the patterns worth naming', () => {
		expect(recompositionNote(-1.5, 0.1)).toContain('weight holds');
		expect(recompositionNote(-1.5, -1.3)).toContain('Both coming down');
		expect(recompositionNote(-0.2, 0)).toBeNull();
		expect(recompositionNote(-1.5, 1.2)).toBeNull();
		expect(recompositionNote(1, 1)).toBeNull();
	});
});
