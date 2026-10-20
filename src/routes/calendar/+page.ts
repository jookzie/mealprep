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

// The block in view lives in the URL, so paging is navigation and survives a refresh.
export const load: PageLoad = async ({ url }) => {
	const requested = url.searchParams.get('week');
	const weekStart = startOfIsoWeek(isIsoDate(requested) ? requested : todayIso());
	const { from, to } = periodRange(weekStart);
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
		days,
		dayPlans,
		joinedPlans: joinablePlans(dayPlans, assignedPlans),
		targets,
		categories,
	};
};
