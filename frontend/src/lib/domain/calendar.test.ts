import { describe, expect, test } from 'bun:test';
import type { CalendarDay, DayPlan, Macros, Targets } from '../api/gen/types.gen';
import { buildPlannedWeek } from './calendar';
import { ZERO_MACROS } from './macros';

const WEEK_START = '2026-08-31';

function macros(energyKcal: number): Macros {
	return { energyKcal, fatG: 10, proteinG: 20, carbohydratesG: 30 };
}

function plan(id: string, energyKcal: number): DayPlan {
	return {
		id,
		label: `Plan ${id}`,
		meals: [],
		macros: macros(energyKcal),
		createdAt: '2026-08-01T00:00:00Z',
		updatedAt: '2026-08-01T00:00:00Z',
	};
}

function assignment(date: string, dayPlanId: string): CalendarDay {
	return {
		date,
		dayPlanId,
		createdAt: '2026-08-01T00:00:00Z',
		updatedAt: '2026-08-01T00:00:00Z',
	};
}

const targets: Targets = {
	macros: macros(2000),
	createdAt: '2026-08-01T00:00:00Z',
	updatedAt: '2026-08-01T00:00:00Z',
};

test('an empty week still has seven days', () => {
	const week = buildPlannedWeek({ weekStart: WEEK_START, days: [], dayPlans: [], targets: null });
	expect(week.days).toHaveLength(7);
	expect(week.days.every((day) => day.dayPlan === null)).toBe(true);
	expect(week.plannedCount).toBe(0);
	expect(week.total).toEqual(ZERO_MACROS);
});

test('assigned days carry the plan and its macros', () => {
	const week = buildPlannedWeek({
		weekStart: WEEK_START,
		days: [assignment('2026-09-01', 'a')],
		dayPlans: [plan('a', 1800)],
		targets: null,
	});
	const tuesday = week.days[1];
	expect(tuesday.date).toBe('2026-09-01');
	expect(tuesday.dayPlan?.id).toBe('a');
	expect(tuesday.macros?.energyKcal).toBe(1800);
});

describe('targets', () => {
	// The point of the whole screen: an unplanned day must not read as a shortfall.
	test('the week target is the daily target times the days actually planned', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'a')],
			dayPlans: [plan('a', 1800)],
			targets,
		});
		expect(week.plannedCount).toBe(2);
		expect(week.total.energyKcal).toBe(3600);
		expect(week.targetTotal?.energyKcal).toBe(4000);
	});

	test('without targets there is nothing to compare against', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a')],
			dayPlans: [plan('a', 1800)],
			targets: null,
		});
		expect(week.targetPerDay).toBeNull();
		expect(week.targetTotal).toBeNull();
	});
});

describe('a plan that has been deleted', () => {
	const week = buildPlannedWeek({
		weekStart: WEEK_START,
		days: [assignment('2026-08-31', 'gone'), assignment('2026-09-01', 'a')],
		dayPlans: [plan('a', 1800)],
		targets,
	});

	test('keeps the id so the day can still be unassigned', () => {
		expect(week.days[0].dayPlanId).toBe('gone');
		expect(week.days[0].dayPlan).toBeNull();
		expect(week.days[0].macros).toBeNull();
	});

	test('does not count towards the total or the target', () => {
		expect(week.plannedCount).toBe(1);
		expect(week.total.energyKcal).toBe(1800);
		expect(week.targetTotal?.energyKcal).toBe(2000);
	});
});

test('assignments outside the week are ignored', () => {
	const week = buildPlannedWeek({
		weekStart: WEEK_START,
		days: [assignment('2026-09-14', 'a')],
		dayPlans: [plan('a', 1800)],
		targets: null,
	});
	expect(week.plannedCount).toBe(0);
});

describe('the per-day average', () => {
	// The target the user set is a daily figure, so the week total cannot be compared
	// against it directly. The average over the planned days can.
	test('divides by the days planned, not by seven', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'b')],
			dayPlans: [plan('a', 1800), plan('b', 2200)],
			targets,
		});
		expect(week.averagePerPlannedDay?.energyKcal).toBe(2000);
	});

	test('a day whose plan was deleted stays out of the denominator', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'gone')],
			dayPlans: [plan('a', 1800)],
			targets,
		});
		expect(week.averagePerPlannedDay?.energyKcal).toBe(1800);
	});

	test('a week with nothing planned has no average rather than a zero', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [],
			dayPlans: [],
			targets,
		});
		expect(week.averagePerPlannedDay).toBeNull();
	});
});
