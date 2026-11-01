import { expect, test } from 'bun:test';
import { formatAmount, formatGrams, formatSigned } from './format';

test('overviews show whole grams', () => {
	expect(formatGrams(12.6)).toBe('13 g');
	expect(formatAmount(37.4, 'ml')).toBe('37 ml');
	expect(formatSigned(-4.6, 'g')).toBe('−5 g');
});

test('editors keep a tenth', () => {
	expect(formatGrams(12.64, true)).toBe('12.6 g');
	expect(formatAmount(37.45, 'ml', true)).toBe('37.5 ml');
	expect(formatSigned(-4.64, 'g', true)).toBe('−4.6 g');
});
