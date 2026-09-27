import { describe, expect, test } from 'bun:test';
import type { WeightPoint } from '$lib/api/types';
import {
	type Box,
	domain,
	dots,
	FOOTROOM,
	formatKilograms,
	formatRate,
	MIN_SPAN_KG,
	nearest,
	scale,
	ticks,
	trendPath,
	visible,
} from './weight';

function point(date: string, trend: number, kilograms?: number): WeightPoint {
	return { date, trend, kilograms };
}

const BOX: Box = { width: 300, height: 100, padX: 0, padY: 0 };

describe('visible', () => {
	test('keeps the last days of the series', () => {
		const points = [point('2026-09-01', 80), point('2026-09-02', 81), point('2026-09-03', 82)];

		expect(visible(points, 30)).toHaveLength(3);
		expect(visible(points, 0)).toHaveLength(3);
	});

	test('takes the most recent end when the series is longer than the range', () => {
		const points = Array.from({ length: 100 }, (_, index) =>
			point(`2026-09-${index}`, 80 + index),
		);

		const last30 = visible(points, 30);

		expect(last30).toHaveLength(30);
		expect(last30[29].trend).toBe(179);
	});
});

describe('domain', () => {
	test('leaves the lowest value out of the bottom third of the plot', () => {
		const points = [point('2026-09-01', 80, 80), point('2026-09-02', 90, 90)];

		const { min, max } = domain(points);
		const position = (80 - min) / (max - min);

		expect(position).toBeGreaterThan(0.3);
		expect(min).toBeLessThan(80);
		expect(max).toBeGreaterThan(90);
	});

	test('widens a flat series to the minimum span rather than scaling its noise', () => {
		const points = [point('2026-09-01', 80, 80), point('2026-09-02', 80, 80)];

		const { min, max } = domain(points);

		expect(max - min).toBeGreaterThanOrEqual(MIN_SPAN_KG);
		expect(min).toBeCloseTo(80 - MIN_SPAN_KG * FOOTROOM, 6);
	});

	test('covers the trend on days with no measurement', () => {
		const points = [point('2026-09-01', 80, 80), point('2026-09-02', 95)];

		expect(domain(points).max).toBeGreaterThan(95);
	});

	test('survives an empty series', () => {
		expect(domain([])).toEqual({ min: 0, max: MIN_SPAN_KG });
	});
});

describe('scale', () => {
	test('spreads the series across the width and puts the maximum at the top', () => {
		const to = scale(3, { min: 0, max: 100 }, BOX);

		expect(to.x(0)).toBe(0);
		expect(to.x(2)).toBe(300);
		expect(to.y(100)).toBe(0);
		expect(to.y(0)).toBe(100);
	});

	test('does not divide by zero on a single point', () => {
		const to = scale(1, { min: 80, max: 80 }, BOX);

		expect(Number.isFinite(to.x(0))).toBe(true);
		expect(Number.isFinite(to.y(80))).toBe(true);
	});
});

describe('trendPath and dots', () => {
	const points = [point('2026-09-01', 80, 80), point('2026-09-02', 81), point('2026-09-03', 82, 84)];
	const to = scale(points.length, { min: 80, max: 84 }, BOX);

	test('draws the trend through every day, gaps included', () => {
		expect(trendPath(points, to).match(/[ML]/g)).toHaveLength(3);
	});

	test('marks only the days that were weighed', () => {
		const marks = dots(points, to);

		expect(marks).toHaveLength(2);
		expect(marks.map((mark) => mark.date)).toEqual(['2026-09-01', '2026-09-03']);
	});
});

describe('ticks', () => {
	test('gives a narrow weight range more than one gridline', () => {
		expect(ticks({ min: 76.9, max: 84.8 }).length).toBeGreaterThanOrEqual(3);
	});

	test('lands on round values inside the domain', () => {
		const found = ticks({ min: 78.3, max: 84.1 });

		expect(found.length).toBeGreaterThan(0);
		for (const value of found) {
			expect(value).toBeGreaterThanOrEqual(78.3);
			expect(value).toBeLessThanOrEqual(84.1);
			expect(value % 1).toBeCloseTo(value % 1 === 0 ? 0 : value % 1, 6);
		}
	});

	test('returns nothing for a degenerate domain', () => {
		expect(ticks({ min: 80, max: 80 })).toEqual([]);
	});
});

describe('nearest', () => {
	test('finds the point under a position', () => {
		const points = [point('a', 80), point('b', 81), point('c', 82)];
		const to = scale(points.length, { min: 80, max: 82 }, BOX);

		expect(nearest(points, 0, to)).toBe(0);
		expect(nearest(points, 299, to)).toBe(2);
		expect(nearest(points, 151, to)).toBe(1);
	});

	test('answers -1 for an empty series', () => {
		expect(nearest([], 10, scale(0, { min: 0, max: 1 }, BOX))).toBe(-1);
	});
});

describe('formatting', () => {
	test('shows one decimal, which is what a scale reads', () => {
		expect(formatKilograms(80.25)).toBe('80.3 kg');
	});

	test('always signs the rate, because the direction is the point', () => {
		expect(formatRate(-0.42)).toBe('−0.4 kg/week');
		expect(formatRate(0.42)).toBe('+0.4 kg/week');
	});

	test('reads a negligible rate as holding steady rather than as a tiny loss', () => {
		expect(formatRate(-0.01)).toBe('±0.0 kg/week');
	});
});
