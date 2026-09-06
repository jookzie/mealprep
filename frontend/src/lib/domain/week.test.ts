import { describe, expect, test } from 'bun:test';
import { isIsoDate, shiftWeeks, startOfIsoWeek, weekDates, weekdayLabel, weekRange } from './week';

describe('startOfIsoWeek', () => {
	test('a Monday is its own week start', () => {
		expect(startOfIsoWeek('2026-09-07')).toBe('2026-09-07');
	});

	test('a Sunday belongs to the week that began six days earlier', () => {
		expect(startOfIsoWeek('2026-09-06')).toBe('2026-08-31');
	});

	test('crosses a month boundary', () => {
		expect(startOfIsoWeek('2026-09-02')).toBe('2026-08-31');
	});

	test('crosses a year boundary', () => {
		expect(startOfIsoWeek('2027-01-01')).toBe('2026-12-28');
	});
});

describe('UTC handling', () => {
	// The bug this guards: new Date('2026-09-07').getDate() reads local, which is the
	// 6th anywhere west of UTC. Every helper must be immune to the runner's timezone.
	test('a date survives a round trip regardless of the local zone', () => {
		for (const date of ['2026-01-01', '2026-06-15', '2026-12-31']) {
			expect(weekDates(startOfIsoWeek(date))).toContain(date);
		}
	});

	test('weekday labels are derived in UTC', () => {
		expect(weekdayLabel('2026-09-07')).toBe(
			new Intl.DateTimeFormat(undefined, { weekday: 'short', timeZone: 'UTC' }).format(
				new Date(Date.UTC(2026, 8, 7)),
			),
		);
	});
});

describe('week shape', () => {
	test('a week is seven consecutive days', () => {
		const dates = weekDates('2026-08-31');
		expect(dates).toHaveLength(7);
		expect(dates).toEqual([
			'2026-08-31',
			'2026-09-01',
			'2026-09-02',
			'2026-09-03',
			'2026-09-04',
			'2026-09-05',
			'2026-09-06',
		]);
	});

	test('the range runs Monday to Sunday inclusive', () => {
		expect(weekRange('2026-08-31')).toEqual({ from: '2026-08-31', to: '2026-09-06' });
	});

	test('shifting weeks moves in seven-day steps', () => {
		expect(shiftWeeks('2026-08-31', 1)).toBe('2026-09-07');
		expect(shiftWeeks('2026-08-31', -1)).toBe('2026-08-24');
		expect(shiftWeeks('2026-08-31', 0)).toBe('2026-08-31');
	});
});

describe('isIsoDate', () => {
	test('accepts a real date', () => {
		expect(isIsoDate('2026-02-28')).toBe(true);
	});

	test('rejects a day that does not exist', () => {
		expect(isIsoDate('2026-02-30')).toBe(false);
	});

	test('rejects malformed and missing values', () => {
		expect(isIsoDate('2026-9-7')).toBe(false);
		expect(isIsoDate('not a date')).toBe(false);
		expect(isIsoDate(null)).toBe(false);
		expect(isIsoDate(undefined)).toBe(false);
	});
});
