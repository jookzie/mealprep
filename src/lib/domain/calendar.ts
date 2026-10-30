import type { CalendarDay, Cost, DayPlan, Macros, Targets } from '../api/types';
import { addCost, scaleCost, ZERO_COST } from './cost';
import { addMacros, scaleMacros, ZERO_MACROS } from './macros';
import { type IsoDate, periodWeekStarts, weekDates } from './week';

export type PlannedDay = {
	date: IsoDate;
	/** Kept even when the plan itself is gone, so the day can still be unassigned. */
	dayPlanId: string | null;
	/** Null when nothing is assigned, or when the assigned plan has been deleted. */
	dayPlan: DayPlan | null;
	/** The plan's own figure from the API. Never recomputed here. */
	macros: Macros | null;
	/** The plan's own cost, with the same provenance as its macros. */
	cost: Cost | null;
};

export type PlannedWeek = {
	weekStart: IsoDate;
	/** Exactly seven, in date order, unplanned days included. */
	days: PlannedDay[];
	plannedCount: number;
	total: Macros;
	/**
	 * The total over the days actually planned. The target the user set is a daily
	 * figure, so this is the number that can be compared against it directly; the
	 * week total cannot.
	 */
	averagePerPlannedDay: Macros | null;
	targetPerDay: Macros | null;
	/** The per-day target times the days actually planned, not times seven. */
	targetTotal: Macros | null;
	/** What the planned days cost, incomplete as soon as one plan's cost is. */
	totalCost: Cost;
	averageCostPerPlannedDay: Cost | null;
};

/**
 * Joins the given dates to the assignments on them and the plans those name, one entry
 * per date in the order given, unplanned dates included. A date whose plan has been
 * deleted keeps its dayPlanId and reads as unplanned.
 */
export function plannedDays(
	dates: readonly IsoDate[],
	days: readonly CalendarDay[],
	dayPlans: readonly DayPlan[],
): PlannedDay[] {
	const plansById = new Map(dayPlans.map((plan) => [plan.id, plan]));
	const assignedByDate = new Map(days.map((day) => [day.date, day]));
	return dates.map((date): PlannedDay => {
		const assigned = assignedByDate.get(date);
		if (!assigned) {
			return { date, dayPlanId: null, dayPlan: null, macros: null, cost: null };
		}
		const dayPlan = plansById.get(assigned.dayPlanId) ?? null;
		return {
			date,
			dayPlanId: assigned.dayPlanId,
			dayPlan,
			macros: dayPlan?.macros ?? null,
			cost: dayPlan?.cost ?? null,
		};
	});
}

/**
 * Joins a week of assignments to the plans they name. This is the only place a
 * dayPlanId is resolved to macros, so the grid and the summary cannot disagree.
 *
 * A day whose plan has been deleted counts as unplanned: its macros are unknowable,
 * and counting it would read as a shortfall against the target rather than as the
 * dangling reference it is.
 */
export function buildPlannedWeek(input: {
	weekStart: IsoDate;
	days: readonly CalendarDay[];
	dayPlans: readonly DayPlan[];
	targets: Targets | null;
}): PlannedWeek {
	const days = plannedDays(weekDates(input.weekStart), input.days, input.dayPlans);

	const planned = days.filter((day) => day.macros !== null);
	const total = planned.reduce((sum, day) => addMacros(sum, day.macros as Macros), ZERO_MACROS);
	const totalCost = planned.reduce((sum, day) => addCost(sum, day.cost as Cost), ZERO_COST);
	const targetPerDay = input.targets?.macros ?? null;

	return {
		weekStart: input.weekStart,
		days,
		plannedCount: planned.length,
		total,
		averagePerPlannedDay: planned.length === 0 ? null : scaleMacros(total, 1 / planned.length),
		targetPerDay,
		targetTotal: targetPerDay ? scaleMacros(targetPerDay, planned.length) : null,
		totalCost,
		averageCostPerPlannedDay:
			planned.length === 0 ? null : scaleCost(totalCost, 1 / planned.length),
	};
}

export type PlannedPeriod = {
	/** The first week of the block; paging moves by the whole block. */
	weekStart: IsoDate;
	weeks: PlannedWeek[];
	plannedCount: number;
	dayCount: number;
	total: Macros;
	averagePerPlannedDay: Macros | null;
	targetPerDay: Macros | null;
	targetTotal: Macros | null;
	totalCost: Cost;
	averageCostPerPlannedDay: Cost | null;
};

/**
 * A block of consecutive weeks, each built by the same join as a single week, so the
 * period and the rows inside it cannot disagree about a day.
 *
 * The unplanned-day rule carries all the way up: a day with no plan, or one whose plan
 * has been deleted, is excluded from every denominator here too.
 */
export function buildPlannedPeriod(input: {
	weekStart: IsoDate;
	days: readonly CalendarDay[];
	dayPlans: readonly DayPlan[];
	targets: Targets | null;
}): PlannedPeriod {
	const weeks = periodWeekStarts(input.weekStart).map((weekStart) =>
		buildPlannedWeek({ ...input, weekStart }),
	);

	const plannedCount = weeks.reduce((sum, week) => sum + week.plannedCount, 0);
	const total = weeks.reduce((sum, week) => addMacros(sum, week.total), ZERO_MACROS);
	const totalCost = weeks.reduce((sum, week) => addCost(sum, week.totalCost), ZERO_COST);
	const targetPerDay = input.targets?.macros ?? null;

	return {
		weekStart: input.weekStart,
		weeks,
		plannedCount,
		dayCount: weeks.length * 7,
		total,
		averagePerPlannedDay: plannedCount === 0 ? null : scaleMacros(total, 1 / plannedCount),
		targetPerDay,
		targetTotal: targetPerDay ? scaleMacros(targetPerDay, plannedCount) : null,
		totalCost,
		averageCostPerPlannedDay: plannedCount === 0 ? null : scaleCost(totalCost, 1 / plannedCount),
	};
}

/** What assigning a plan would do to a date: nothing, replace another plan, or fill a gap. */
export type AssignmentEffect = 'same' | 'replace' | 'new';

/**
 * Says, per date, what a bulk assignment of one plan would do, so the sheet can name the
 * days it changes before anything is written (CL-4). A date whose plan was deleted counts
 * as a replacement: there is an assignment there, even if it no longer resolves.
 */
export function assignmentEffects(
	dates: readonly IsoDate[],
	days: readonly CalendarDay[],
	dayPlanId: string,
): { date: IsoDate; effect: AssignmentEffect }[] {
	const assignedByDate = new Map(days.map((day) => [day.date, day.dayPlanId]));
	return dates.map((date) => {
		const current = assignedByDate.get(date);
		if (current === undefined) return { date, effect: 'new' };
		return { date, effect: current === dayPlanId ? 'same' : 'replace' };
	});
}
