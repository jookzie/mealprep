import { expect, test } from 'bun:test';
import { canMoveDown, canMoveUp, move, moveAnnouncement } from './order';

test('an entry moves and the rest close up behind it', () => {
	expect(move(['a', 'b', 'c'], 2, 0)).toEqual(['c', 'a', 'b']);
	expect(move(['a', 'b', 'c'], 0, 1)).toEqual(['b', 'a', 'c']);
});

test('the source array is never mutated', () => {
	const items = ['a', 'b', 'c'];
	move(items, 0, 2);
	expect(items).toEqual(['a', 'b', 'c']);
});

test('a move that goes nowhere, or off either end, is a copy', () => {
	expect(move(['a', 'b'], 1, 1)).toEqual(['a', 'b']);
	expect(move(['a', 'b'], -1, 0)).toEqual(['a', 'b']);
	expect(move(['a', 'b'], 0, 5)).toEqual(['a', 'b']);
});

test('the ends of the list disable the button that would leave it', () => {
	expect(canMoveUp(0)).toBe(false);
	expect(canMoveUp(1)).toBe(true);
	expect(canMoveDown(1, 2)).toBe(false);
	expect(canMoveDown(0, 2)).toBe(true);
});

test('the announcement counts from one, as a person reads a list', () => {
	expect(moveAnnouncement('Rice', 1, 4)).toBe('Rice moved to position 2 of 4.');
});
