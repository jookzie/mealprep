import { expect, test } from 'bun:test';
import { formatDecimal, parseDecimal, step } from './number';

test('an empty field is null, not zero', () => {
	expect(parseDecimal('')).toBeNull();
	expect(parseDecimal('   ')).toBeNull();
});

test('a comma is a decimal separator, not a reason to give up', () => {
	expect(parseDecimal('12,5')).toBe(12.5);
	expect(parseDecimal('12.5')).toBe(12.5);
});

test('half-typed exponents never reach the draft as NaN', () => {
	expect(parseDecimal('1e')).toBeNull();
	expect(parseDecimal('abc')).toBeNull();
});

test('what the field shows on the way back out is the number itself', () => {
	expect(formatDecimal(12.5)).toBe('12.5');
	expect(formatDecimal(null)).toBe('');
});

test('stepping is one, ten with shift, a tenth with ctrl', () => {
	expect(step(10, 1)).toBe(11);
	expect(step(10, -1)).toBe(9);
	expect(step(10, 1, { shift: true })).toBe(20);
	expect(step(10, 1, { ctrl: true })).toBe(10.1);
});

test('stepping an empty field starts from zero and keeps no float dust', () => {
	expect(step(null, 1)).toBe(1);
	expect(step(0.3, 1, { ctrl: true })).toBe(0.4);
});
