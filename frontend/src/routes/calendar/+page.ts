import { listCalendarDays, listDayPlans, loadTargets } from '$lib/api';
import { buildPlannedPeriod } from '$lib/domain/calendar';
import { isIsoDate, periodRange, startOfIsoWeek, todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

// The first week of the block lives in the URL, so paging is a navigation and `url` is
// already a tracked dependency of this load. There is no endpoint totalling a range, so
// the block is joined here from the assignments and the plans they name.
export const load: PageLoad = async ({ fetch, url }) => {
	const requested = url.searchParams.get('week');
	const weekStart = startOfIsoWeek(isIsoDate(requested) ? requested : todayIso());
	const { from, to } = periodRange(weekStart);

	const [days, dayPlans, targets] = await Promise.all([
		listCalendarDays({ fetch, query: { from, to }, throwOnError: true }),
		listDayPlans({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);

	return {
		period: buildPlannedPeriod({
			weekStart,
			days: days.data.days,
			dayPlans: dayPlans.data.dayPlans,
			targets,
		}),
		dayPlans: dayPlans.data.dayPlans,
	};
};
