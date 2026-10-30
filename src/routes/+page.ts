import {
	listCalendarDayPlans,
	listCalendarDays,
	listDayPlans,
	loadTargets,
	loadWeightSeries,
	read,
} from '$lib/api';
import { joinablePlans } from '$lib/domain/assistant';
import { plannedDays } from '$lib/domain/calendar';
import { datesInRange, shiftDays, todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

/** Today and the six days after it: the day in hand, and what is coming. */
const DAYS_AHEAD = 6;

// Today is the home screen: the plan in hand, measured against the targets, with the
// days after it one glance away.
export const load: PageLoad = async () => {
	const today = todayIso();
	const to = shiftDays(today, DAYS_AHEAD);
	// The calendar read resolves a date the assistant detached onto its own copy, which
	// the plan list deliberately leaves out, so both are joined as on the calendar.
	const [days, dayPlans, assignedPlans, targets, weight] = await read(
		Promise.all([
			listCalendarDays(today, to),
			listDayPlans(),
			listCalendarDayPlans(today, to),
			loadTargets(),
			loadWeightSeries(),
		]),
	);
	const planned = plannedDays(datesInRange(today, to), days, joinablePlans(dayPlans, assignedPlans));
	return { today: planned[0], ahead: planned.slice(1), targets, weight };
};
