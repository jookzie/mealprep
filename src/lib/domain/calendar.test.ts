import { describe, expect, test } from 'bun:test';
import type { CalendarDay, DayPlan, Macros, Targets } from '../api/types';
import { buildPlannedPeriod, buildPlannedWeek } from './calendar';
import { ZERO_COST } from './cost';
import { ZERO_MACROS } from './macros';

const WEEK_START = '2026-08-31';

function macros(energyKcal: number): Macros {
	return { energyKcal, fatG: 10, proteinG: 20, carbohydratesG: 30 };
}

function plan(id: string, energyKcal: number, cost = 3): DayPlan {
	return {
		id,
		label: `Plan ${id}`,
		items: [],
		macros: macros(energyKcal),
		cost: { amount: cost, complete: true },
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
	expect(week.totalCost).toEqual(ZERO_COST);
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

describe('a four-week block', () => {
	const period = buildPlannedPeriod({
		weekStart: WEEK_START,
		days: [
			assignment('2026-08-31', 'a'), // week 1
			assignment('2026-09-08', 'b'), // week 2
			assignment('2026-09-16', 'gone'), // week 3, plan deleted
			assignment('2026-10-05', 'a'), // outside the block
		],
		dayPlans: [plan('a', 1800), plan('b', 2200)],
		targets,
	});

	test('holds four weeks of seven days', () => {
		expect(period.weeks).toHaveLength(4);
		expect(period.weeks.flatMap((w) => w.days)).toHaveLength(28);
		expect(period.dayCount).toBe(28);
	});

	test('each week is joined the same way as a standalone week', () => {
		expect(period.weeks[0].days[0].dayPlan?.id).toBe('a');
		expect(period.weeks[1].days[1].dayPlan?.id).toBe('b');
	});

	test('an assignment past the block is left out', () => {
		expect(period.plannedCount).toBe(2);
		expect(period.total.energyKcal).toBe(4000);
	});

	test('a deleted plan stays out of the denominator here too', () => {
		expect(period.weeks[2].plannedCount).toBe(0);
		expect(period.averagePerPlannedDay?.energyKcal).toBe(2000);
	});

	test('the block target is the daily target times the days actually planned', () => {
		expect(period.targetTotal?.energyKcal).toBe(4000);
	});

	test('a block with nothing planned has no average rather than a zero', () => {
		const empty = buildPlannedPeriod({
			weekStart: WEEK_START,
			days: [],
			dayPlans: [],
			targets,
		});
		expect(empty.averagePerPlannedDay).toBeNull();
		expect(empty.plannedCount).toBe(0);
	});
});

describe('cost', () => {
	test('a week costs what its planned days cost', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'b')],
			dayPlans: [plan('a', 1800, 4), plan('b', 2200, 6)],
			targets,
		});
		expect(week.totalCost).toEqual({ amount: 10, complete: true });
		expect(week.averageCostPerPlannedDay).toEqual({ amount: 5, complete: true });
	});

	test('one plan short of a price makes the week say so', () => {
		const incomplete: DayPlan = { ...plan('b', 2200, 6), cost: { amount: 6, complete: false } };
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'b')],
			dayPlans: [plan('a', 1800, 4), incomplete],
			targets,
		});
		expect(week.totalCost).toEqual({ amount: 10, complete: false });
		expect(week.averageCostPerPlannedDay?.complete).toBe(false);
	});

	test('a day whose plan was deleted costs nothing and is not counted', () => {
		const week = buildPlannedWeek({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-01', 'gone')],
			dayPlans: [plan('a', 1800, 4)],
			targets,
		});
		expect(week.days[1].cost).toBeNull();
		expect(week.totalCost).toEqual({ amount: 4, complete: true });
	});

	test('a block totals the weeks in it', () => {
		const period = buildPlannedPeriod({
			weekStart: WEEK_START,
			days: [assignment('2026-08-31', 'a'), assignment('2026-09-08', 'a')],
			dayPlans: [plan('a', 1800, 4)],
			targets,
		});
		expect(period.totalCost).toEqual({ amount: 8, complete: true });
		expect(period.averageCostPerPlannedDay).toEqual({ amount: 4, complete: true });
	});

	test('a block with nothing planned has no average cost rather than a zero', () => {
		const empty = buildPlannedPeriod({
			weekStart: WEEK_START,
			days: [],
			dayPlans: [],
			targets,
		});
		expect(empty.averageCostPerPlannedDay).toBeNull();
		expect(empty.totalCost).toEqual(ZERO_COST);
	});
});
