import { expect, test } from 'bun:test';
import { columnScale, MAX_TINT, tint } from './heatmap';

test('the scale runs from the column minimum to its maximum', () => {
	const scale = columnScale([100, 200, 300]);
	expect(scale(100)).toBe(0);
	expect(scale(200)).toBe(0.5);
	expect(scale(300)).toBe(1);
});

test('a column with no spread shades flat rather than fully', () => {
	const scale = columnScale([5, 5, 5]);
	expect(scale(5)).toBe(0);
});

test('an empty column shades nothing', () => {
	expect(columnScale([])(42)).toBe(0);
});

test('values outside the column clamp instead of overshooting', () => {
	const scale = columnScale([10, 20]);
	expect(scale(0)).toBe(0);
	expect(scale(99)).toBe(1);
});

test('negatives are ordinary values, not a special case', () => {
	const scale = columnScale([-10, 0, 10]);
	expect(scale(-10)).toBe(0);
	expect(scale(0)).toBe(0.5);
});

test('the tint stays well short of opaque so the figure stays readable', () => {
	expect(tint(1)).toBe(MAX_TINT);
	expect(MAX_TINT).toBeLessThan(0.5);
});
