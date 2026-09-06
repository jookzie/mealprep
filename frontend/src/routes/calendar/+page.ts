import { listCalendarDays, listDayPlans, loadTargets } from '$lib/api';
import { buildPlannedWeek } from '$lib/domain/calendar';
import { isIsoDate, startOfIsoWeek, todayIso, weekRange } from '$lib/domain/week';
import type { PageLoad } from './$types';

// The week lives in the URL, so paging is a navigation and `url` is already a tracked
// dependency of this load. There is no endpoint totalling a range, so the week is
// joined here from the assignments and the plans they name.
export const load: PageLoad = async ({ fetch, url }) => {
	const requested = url.searchParams.get('week');
	const weekStart = startOfIsoWeek(isIsoDate(requested) ? requested : todayIso());
	const { from, to } = weekRange(weekStart);

	const [days, dayPlans, targets] = await Promise.all([
		listCalendarDays({ fetch, query: { from, to }, throwOnError: true }),
		listDayPlans({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);

	return {
		week: buildPlannedWeek({
			weekStart,
			days: days.data.days,
			dayPlans: dayPlans.data.dayPlans,
			targets,
		}),
		dayPlans: dayPlans.data.dayPlans,
	};
};
