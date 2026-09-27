import { describe, expect, test } from 'bun:test';
import { bandPath, type ChartPoint, chartDomain, linePath, MARGIN, valueDots } from './trend-chart';
import { type Box, scale } from './weight';

const BOX: Box = { width: 300, height: 100, padX: 0, padY: 0 };

describe('chartDomain', () => {
	test('covers readings, the line and the band, with a margin either side', () => {
		const points: ChartPoint[] = [
			{ date: '2026-09-01', value: 50, band: { low: 45, high: 60 } },
			{ date: '2026-09-02', line: 55 },
		];

		const { min, max } = chartDomain(points, 1);

		expect(min).toBeCloseTo(45 - 15 * MARGIN);
		expect(max).toBeCloseTo(60 + 15 * MARGIN);
	});

	test('a flat series still gets a window of the minimum span, centred on it', () => {
		const { min, max } = chartDomain([{ date: '2026-09-01', value: 50 }], 10);

		expect((min + max) / 2).toBeCloseTo(50);
		expect(max - min).toBeGreaterThanOrEqual(10);
	});
});

describe('linePath', () => {
	test('breaks where a day has no line', () => {
		const points: ChartPoint[] = [
			{ date: '2026-09-01', line: 1 },
			{ date: '2026-09-02', line: 2 },
			{ date: '2026-09-03' },
			{ date: '2026-09-04', line: 3 },
		];
		const to = scale(points.length, { min: 0, max: 4 }, BOX);

		const path = linePath(points, to);

		expect(path.match(/M/g)).toHaveLength(2);
		expect(path.match(/L/g)).toHaveLength(1);
	});
});

describe('bandPath', () => {
	test('closes one shape per run of days with a band', () => {
		const band = { low: 1, high: 2 };
		const points: ChartPoint[] = [
			{ date: '2026-09-01', band },
			{ date: '2026-09-02', band },
			{ date: '2026-09-03' },
			{ date: '2026-09-04', band },
		];
		const to = scale(points.length, { min: 0, max: 3 }, BOX);

		expect(bandPath(points, to).match(/Z/g)).toHaveLength(2);
		expect(bandPath([{ date: '2026-09-01' }], to)).toBe('');
	});
});

describe('valueDots', () => {
	test('marks only the days that were read', () => {
		const points: ChartPoint[] = [
			{ date: '2026-09-01', value: 1, line: 1 },
			{ date: '2026-09-02', line: 1 },
			{ date: '2026-09-03', value: 2 },
		];
		const to = scale(points.length, { min: 0, max: 2 }, BOX);

		const dots = valueDots(points, to);

		expect(dots.map((dot) => dot.date)).toEqual(['2026-09-01', '2026-09-03']);
		expect(dots[1].x).toBe(300);
	});
});
