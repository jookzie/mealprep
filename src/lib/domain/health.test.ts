import { describe, expect, test } from 'bun:test';
import type { Impact, SleepNight } from '$lib/api/types';
import {
	formatClock,
	hourTicks,
	sleepDays,
	sleepDomain,
	formatDuration,
	impactHelps,
	impactPhrase,
	rankImpacts,
	SYNC_EVERY_MS,
	shouldSync,
	statusMeaning,
	syncedLabel,
} from './health';

function impact(changePercent: number, clear = true): Impact {
	return {
		behaviour: 'slept-seven-hours',
		withDays: 10,
		withoutDays: 10,
		withMean: 0,
		withoutMean: 0,
		changePercent,
		clear,
	};
}

describe('sync', () => {
	const now = new Date('2026-09-27T12:00:00Z');

	test('syncs when never synced or when the last sync is an hour old', () => {
		expect(shouldSync(undefined, now)).toBe(true);
		expect(shouldSync(new Date(now.getTime() - SYNC_EVERY_MS).toISOString(), now)).toBe(true);
		expect(shouldSync('2026-09-27T11:30:00Z', now)).toBe(false);
	});

	test('says how long ago in the largest unit that fits', () => {
		expect(syncedLabel('2026-09-27T11:59:40Z', now)).toBe('synced just now');
		expect(syncedLabel('2026-09-27T11:15:00Z', now)).toBe('synced 45 min ago');
		expect(syncedLabel('2026-09-27T09:00:00Z', now)).toBe('synced 3 h ago');
		expect(syncedLabel('2026-09-25T12:00:00Z', now)).toBe('synced 2 d ago');
	});
});

describe('formatting', () => {
	test('durations read as hours and minutes', () => {
		expect(formatDuration(440)).toBe('7h 20m');
		expect(formatDuration(480)).toBe('8h');
		expect(formatDuration(45)).toBe('45m');
	});

	test('minutes before midnight read as the evening before', () => {
		expect(formatClock(-30)).toBe('23:30');
		expect(formatClock(420)).toBe('07:00');
		expect(formatClock(-1440)).toBe('00:00');
	});
});

describe('meaning', () => {
	test('HRV below and resting heart rate above mean the same thing', () => {
		expect(statusMeaning('hrv', 'below')).toBe(statusMeaning('resting-heart-rate', 'above'));
		expect(statusMeaning('hrv', 'above')).toBe(statusMeaning('resting-heart-rate', 'below'));
	});
});

describe('impacts', () => {
	test('a difference within day-to-day noise is not given a number', () => {
		expect(impactPhrase(impact(3, false), 'hrv')).toBe('no clear difference');
		expect(impactPhrase(impact(-12.4), 'hrv')).toBe('HRV 12% lower');
	});

	test('lower is better for resting heart rate only', () => {
		expect(impactHelps(impact(5), 'hrv')).toBe(true);
		expect(impactHelps(impact(5), 'resting-heart-rate')).toBe(false);
	});

	test('clear impacts lead, largest first', () => {
		const ranked = rankImpacts([impact(2, false), impact(5), impact(-20)]);

		expect(ranked.map((entry) => entry.changePercent)).toEqual([-20, 5, 2]);
	});
});

describe('sleep chart', () => {
	function night(date: string, bedMinute: number, wakeMinute: number): SleepNight {
		return { date, bedMinute, wakeMinute, asleepMinutes: wakeMinute - bedMinute };
	}

	test('a missing night stays in as an empty day', () => {
		const days = sleepDays([night('2026-09-01', -60, 420), night('2026-09-03', -30, 400)]);

		expect(days.map((day) => day.night === null)).toEqual([false, true, false]);
	});

	test('the clock span runs from the earliest bedtime to the latest wake, to the hour', () => {
		const days = sleepDays([night('2026-09-01', -75, 420), night('2026-09-02', 10, 455)]);

		expect(sleepDomain(days)).toEqual({ min: -120, max: 480 });
		expect(hourTicks({ min: -120, max: 480 })).toEqual([-120, 0, 120, 240, 360, 480]);
	});
});
