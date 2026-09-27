import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// The calendar is the working object, so it is the root; there is no dashboard.
export const load: PageLoad = () => {
	redirect(307, '/calendar');
};
