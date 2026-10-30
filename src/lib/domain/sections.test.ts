import { describe, expect, test } from 'bun:test';
import { classifySwipe, direction, neighbour, SECTIONS, sectionOf } from './sections';

describe('sectionOf', () => {
	test('the root is Today', () => {
		expect(sectionOf('/')?.key).toBe('today');
	});

	test('calendar and day plans are Plan', () => {
		expect(sectionOf('/calendar')?.key).toBe('plan');
		expect(sectionOf('/day-plans/abc/edit')?.key).toBe('plan');
	});

	test('meals and products are Library', () => {
		expect(sectionOf('/meals')?.key).toBe('library');
		expect(sectionOf('/products/import/123')?.key).toBe('library');
	});

	test('targets and weight are You', () => {
		expect(sectionOf('/you')?.key).toBe('you');
		expect(sectionOf('/weight')?.key).toBe('you');
	});

	test('a prefix only matches whole segments', () => {
		expect(sectionOf('/mealsplus')).toBeUndefined();
	});
});

describe('neighbour', () => {
	test('walks the tab order', () => {
		expect(neighbour('today', 1).key).toBe('plan');
		expect(neighbour('library', -1).key).toBe('plan');
	});

	test('wraps around both ends', () => {
		expect(neighbour('you', 1).key).toBe('today');
		expect(neighbour('today', -1).key).toBe('you');
	});

	test('a full lap comes back', () => {
		let key = SECTIONS[0].key;
		for (let i = 0; i < SECTIONS.length; i++) key = neighbour(key, 1).key;
		expect(key).toBe(SECTIONS[0].key);
	});
});

describe('direction', () => {
	test('a wrap reads as the step it took', () => {
		expect(direction('you', 'today')).toBe('next');
		expect(direction('today', 'you')).toBe('previous');
	});

	test('a jump follows the tab order', () => {
		expect(direction('today', 'library')).toBe('next');
	});

	test('staying put has no direction', () => {
		expect(direction('plan', 'plan')).toBeNull();
	});
});

describe('classifySwipe', () => {
	test('a quick leftward flick goes to the next section', () => {
		expect(classifySwipe({ dx: -120, dy: 10, ms: 200 })).toBe(1);
	});

	test('a rightward flick goes back', () => {
		expect(classifySwipe({ dx: 120, dy: -10, ms: 200 })).toBe(-1);
	});

	test('a short movement is a tap or a wobble', () => {
		expect(classifySwipe({ dx: -40, dy: 0, ms: 100 })).toBeNull();
	});

	test('a diagonal is a scroll', () => {
		expect(classifySwipe({ dx: -100, dy: 80, ms: 200 })).toBeNull();
	});

	test('a slow drag is not a flick', () => {
		expect(classifySwipe({ dx: -200, dy: 0, ms: 1500 })).toBeNull();
	});
});
