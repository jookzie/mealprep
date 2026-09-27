import { expect, test } from 'bun:test';
import type { Macros } from '../api/types';
import {
	axisAngle,
	MAX_FRACTION,
	maxima,
	pointAt,
	polygon,
	radarPoints,
	ringRadius,
} from './radar';

const target: Macros = { energyKcal: 2000, proteinG: 100, fatG: 50, carbohydratesG: 200 };
const labels = { energyKcal: 'Energy', proteinG: 'Protein', fatG: 'Fat', carbohydratesG: 'Carbs' };

test('the first axis points straight up', () => {
	expect(axisAngle(0, 4)).toBeCloseTo(-Math.PI / 2);
});

test('four axes are a quarter turn apart', () => {
	expect(axisAngle(1, 4) - axisAngle(0, 4)).toBeCloseTo(Math.PI / 2);
});

test('hitting the target lands on the target ring, not the edge', () => {
	// The box holds 1.5x the target, so a fraction of 1 sits two thirds out.
	const { y } = pointAt(0, 4, 60, 1);
	expect(Math.abs(y)).toBeCloseTo(40);
	expect(ringRadius(60, 1)).toBeCloseTo(40);
});

test('the centre is where a zero lands', () => {
	const { x, y } = pointAt(0, 4, 60, 0);
	expect(x).toBeCloseTo(0);
	expect(y).toBeCloseTo(0);
});

test('a plan far over target clamps to the box instead of escaping it', () => {
	const { y } = pointAt(0, 4, 60, 9);
	expect(Math.abs(y)).toBeCloseTo(60);
	expect(MAX_FRACTION).toBe(1.5);
});

test('each axis is a fraction of its own target, not a shared scale', () => {
	const points = radarPoints(
		{ energyKcal: 1000, proteinG: 100, fatG: 25, carbohydratesG: 400 },
		target,
		60,
		labels,
	);
	expect(points.map((p) => p.fraction)).toEqual([0.5, 1, 0.5, 2]);
	expect(points.map((p) => p.label)).toEqual(['Energy', 'Protein', 'Fat', 'Carbs']);
});

test('a zero target reads as an empty axis rather than infinity', () => {
	const points = radarPoints(
		{ energyKcal: 500, proteinG: 0, fatG: 0, carbohydratesG: 0 },
		{ energyKcal: 0, proteinG: 0, fatG: 0, carbohydratesG: 0 },
		60,
		labels,
	);
	expect(points.every((p) => Number.isFinite(p.fraction))).toBe(true);
	expect(points[0].fraction).toBe(0);
});

test('the polygon is the points in axis order', () => {
	expect(
		polygon([
			{ x: 1.234, y: -2 },
			{ x: 0, y: 3 },
		]),
	).toBe('1.23,-2.00 0.00,3.00');
});

test('without targets the largest value on each axis stands in', () => {
	expect(
		maxima([
			{ energyKcal: 1000, proteinG: 50, fatG: 10, carbohydratesG: 300 },
			{ energyKcal: 2400, proteinG: 40, fatG: 90, carbohydratesG: 100 },
		]),
	).toEqual({ energyKcal: 2400, proteinG: 50, fatG: 90, carbohydratesG: 300 });
});

test('maxima of nothing is zero rather than negative infinity', () => {
	expect(maxima([])).toEqual({ energyKcal: 0, proteinG: 0, fatG: 0, carbohydratesG: 0 });
});
