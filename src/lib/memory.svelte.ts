/**
 * What the user was in the middle of — a list's filter and sort, a form not yet saved —
 * kept for as long as the app runs, so that leaving a screen and coming back finds it as
 * it was. A screen's own `$state` dies with the screen, and the tab bar unmounts it on
 * every switch.
 *
 * Memory only: closing the app forgets all of it, and none of it reaches the database
 * until the form it belongs to is saved.
 */
export class Kept<T> {
	value = $state() as T;

	constructor(value: T) {
		this.value = value;
	}
}

const kept = new Map<string, Kept<unknown>>();

/** The value kept under the key, created with the initial value on first use. */
export function keep<T>(key: string, initial: T): Kept<T> {
	let entry = kept.get(key) as Kept<T> | undefined;
	if (entry === undefined) {
		entry = new Kept(initial);
		kept.set(key, entry);
	}
	return entry;
}

/**
 * A form's draft, kept between visits. The key names the record and the version it was
 * loaded at, so a draft of an edit never outlives the record changing underneath it.
 */
export function keepDraft<T>(key: string): Kept<T | null> {
	return keep<T | null>(`draft:${key}`, null);
}

/** Awaits a form's save, and forgets the draft once there is nothing unsaved left in it. */
export async function forgetOnSave<T>(
	draft: Kept<unknown>,
	saving: Promise<T | undefined>,
): Promise<T | undefined> {
	const saved = await saving;
	if (saved !== undefined) draft.value = null;
	return saved;
}
