import { redirect } from '@sveltejs/kit';

// The calendar is the screen the product exists for; there is no separate dashboard
// repeating its figures.
export function load() {
	redirect(307, '/calendar');
}
