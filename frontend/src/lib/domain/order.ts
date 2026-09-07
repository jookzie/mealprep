/**
 * Reordering a composition. The schema persists `position` on meal servings and day
 * plan meals, and the API's array order carries it, so moving an entry is a move
 * within the draft's array and nothing more.
 *
 * Movement is by button rather than by drag: WCAG 2.2 SC 2.5.7 requires a
 * single-pointer alternative to any dragging motion, so the buttons are the
 * required baseline and drag could only ever be added on top of them.
 */
export function move<T>(items: readonly T[], from: number, to: number): T[] {
	if (from === to || from < 0 || from >= items.length || to < 0 || to >= items.length) {
		return [...items];
	}
	const next = [...items];
	const [moved] = next.splice(from, 1);
	next.splice(to, 0, moved);
	return next;
}

export function canMoveUp(index: number): boolean {
	return index > 0;
}

export function canMoveDown(index: number, length: number): boolean {
	return index < length - 1;
}

/** Announced through aria-live, because a reorder is otherwise invisible to a screen reader. */
export function moveAnnouncement(name: string, index: number, length: number): string {
	return `${name} moved to position ${index + 1} of ${length}.`;
}
