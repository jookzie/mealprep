import type { CalendarDay, DayPlan, Macros, Targets } from '../api/gen/types.gen';
import { addMacros, scaleMacros, ZERO_MACROS } from './macros';
import { type IsoDate, weekDates } from './week';

export type PlannedDay = {
	date: IsoDate;
	/** Kept even when the plan itself is gone, so the day can still be unassigned. */
	dayPlanId: string | null;
	/** Null when nothing is assigned, or when the assigned plan has been deleted. */
	dayPlan: DayPlan | null;
	/** The plan's own figure from the API. Never recomputed here. */
	macros: Macros | null;
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
};

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
	const plansById = new Map(input.dayPlans.map((plan) => [plan.id, plan]));
	const assignedByDate = new Map(input.days.map((day) => [day.date, day]));

	const days = weekDates(input.weekStart).map((date): PlannedDay => {
		const assigned = assignedByDate.get(date);
		if (!assigned) {
			return { date, dayPlanId: null, dayPlan: null, macros: null };
		}
		const dayPlan = plansById.get(assigned.dayPlanId) ?? null;
		return {
			date,
			dayPlanId: assigned.dayPlanId,
			dayPlan,
			macros: dayPlan?.macros ?? null,
		};
	});

	const planned = days.filter((day) => day.macros !== null);
	const total = planned.reduce((sum, day) => addMacros(sum, day.macros as Macros), ZERO_MACROS);
	const targetPerDay = input.targets?.macros ?? null;

	return {
		weekStart: input.weekStart,
		days,
		plannedCount: planned.length,
		total,
		averagePerPlannedDay: planned.length === 0 ? null : scaleMacros(total, 1 / planned.length),
		targetPerDay,
		targetTotal: targetPerDay ? scaleMacros(targetPerDay, planned.length) : null,
	};
}
