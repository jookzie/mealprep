import { listCalendarDays, listCategories, listDayPlans, loadTargets, read } from '$lib/api';
import { isIsoDate, periodRange, startOfIsoWeek, todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

// The block in view lives in the URL, so paging is navigation and survives a refresh.
export const load: PageLoad = async ({ url }) => {
	const requested = url.searchParams.get('week');
	const weekStart = startOfIsoWeek(isIsoDate(requested) ? requested : todayIso());
	const { from, to } = periodRange(weekStart);
	const [days, dayPlans, targets, categories] = await read(
		Promise.all([
			listCalendarDays(from, to),
			listDayPlans(),
			loadTargets(),
			listCategories('day-plan'),
		]),
	);
	return { weekStart, days, dayPlans, targets, categories };
};
