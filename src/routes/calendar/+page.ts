import {
	listCalendarDayPlans,
	listCalendarDays,
	listCategories,
	listDayPlans,
	loadTargets,
	read,
} from '$lib/api';
import { joinablePlans } from '$lib/domain/assistant';
import { isIsoDate, periodRange, startOfIsoWeek, todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

/*
 * The block in view and the day picked in it live in the URL, so paging and picking are
 * navigation and survive a refresh. A link to a day (from Today, say) opens the block that
 * starts on that day's week.
 */
export const load: PageLoad = async ({ url }) => {
	const requestedWeek = url.searchParams.get('week');
	const requestedDay = url.searchParams.get('day');
	const anchor = isIsoDate(requestedWeek)
		? requestedWeek
		: isIsoDate(requestedDay)
			? requestedDay
			: todayIso();
	const weekStart = startOfIsoWeek(anchor);
	const { from, to } = periodRange(weekStart);
	const today = todayIso();
	const day = isIsoDate(requestedDay) && requestedDay >= from && requestedDay <= to
		? requestedDay
		: today >= from && today <= to
			? today
			: from;
	// Two reads of the plans, because they answer two questions: dayPlans is what can be
	// assigned, and assignedPlans resolves the dates in view — including a date the
	// assistant has detached onto its own copy, which the list deliberately leaves out.
	const [days, dayPlans, assignedPlans, targets, categories] = await read(
		Promise.all([
			listCalendarDays(from, to),
			listDayPlans(),
			listCalendarDayPlans(from, to),
			loadTargets(),
			listCategories('day-plan'),
		]),
	);
	return {
		weekStart,
		day,
		days,
		dayPlans,
		joinedPlans: joinablePlans(dayPlans, assignedPlans),
		targets,
		categories,
	};
};
