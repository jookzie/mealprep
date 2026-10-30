/**
 * The four sections of the tab bar, in the order a sideways swipe walks them, and the
 * rules for reading a finger movement as that swipe.
 *
 * The assistant is not a section: it is a sheet that opens over whichever one is showing.
 */
export type SectionKey = 'today' | 'plan' | 'library' | 'you';

export type Section = {
	key: SectionKey;
	label: string;
	/** Where the tab goes the first time, before the user has left the section anywhere. */
	href: string;
	/** Every path that belongs to the section; the first match wins. */
	prefixes: readonly string[];
};

export const SECTIONS: readonly Section[] = [
	{ key: 'today', label: 'Today', href: '/', prefixes: [] },
	{ key: 'plan', label: 'Plan', href: '/calendar', prefixes: ['/calendar', '/day-plans'] },
	{ key: 'library', label: 'Library', href: '/meals', prefixes: ['/meals', '/products'] },
	{ key: 'you', label: 'You', href: '/you', prefixes: ['/you', '/weight'] },
];

function within(pathname: string, prefix: string): boolean {
	return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

/** The section a path belongs to. The root is Today; a path no section claims is none. */
export function sectionOf(pathname: string): Section | undefined {
	if (pathname === '/') return SECTIONS[0];
	return SECTIONS.find((section) => section.prefixes.some((prefix) => within(pathname, prefix)));
}

/** The section one step along, wrapping from the last back to the first and the reverse. */
export function neighbour(key: SectionKey, step: 1 | -1): Section {
	const index = SECTIONS.findIndex((section) => section.key === key);
	return SECTIONS[(index + step + SECTIONS.length) % SECTIONS.length];
}

/** Which way a tab change goes, so the transition can slide with it. */
export function direction(from: SectionKey, to: SectionKey): 'next' | 'previous' | null {
	if (from === to) return null;
	if (neighbour(from, 1).key === to) return 'next';
	if (neighbour(from, -1).key === to) return 'previous';
	const fromIndex = SECTIONS.findIndex((section) => section.key === from);
	const toIndex = SECTIONS.findIndex((section) => section.key === to);
	return toIndex > fromIndex ? 'next' : 'previous';
}

/** Far enough that a sloppy vertical scroll never changes the tab. */
export const SWIPE_MIN_PX = 72;
/** Mostly sideways: the horizontal travel has to be this many times the vertical. */
export const SWIPE_SLOPE = 1.8;
/** A deliberate flick, not a slow drag that was really a scroll or a text selection. */
export const SWIPE_MAX_MS = 700;

/**
 * What a finger movement on the page asks for: the next section (finger moved left), the
 * previous one (finger moved right), or nothing.
 */
export function classifySwipe(move: { dx: number; dy: number; ms: number }): 1 | -1 | null {
	const { dx, dy, ms } = move;
	if (ms > SWIPE_MAX_MS) return null;
	if (Math.abs(dx) < SWIPE_MIN_PX) return null;
	if (Math.abs(dx) < Math.abs(dy) * SWIPE_SLOPE) return null;
	return dx < 0 ? 1 : -1;
}

/** Upward travel on the Ask handle that opens the sheet, as a tap does. */
export const SHEET_OPEN_PX = 24;
/** Downward travel on a sheet's grabber that closes it. */
export const SHEET_CLOSE_PX = 80;
