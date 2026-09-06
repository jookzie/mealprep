import { listDayPlans } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const { data } = await listDayPlans({ fetch, throwOnError: true });
	return { dayPlans: data.dayPlans };
};
