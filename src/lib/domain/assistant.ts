import type {
	AppliedChange,
	AssistantRecord,
	DateRange,
	Macros,
	Operation,
	Preview,
	PreviewDay,
} from '../api/types';
import { deviation } from './macros';
import { datesInRange, dayMonthLabel, type IsoDate } from './week';

/** The dates a range covers, in order; empty when it runs backwards. */
export function rangeDates(range: DateRange): IsoDate[] {
	return datesInRange(range.from, range.to);
}

/** How a range reads: one day, or a span. */
export function rangeLabel(range: DateRange): string {
	if (range.from === range.to) return dayMonthLabel(range.from);
	if (range.to < range.from) return 'no days';
	return `${dayMonthLabel(range.from)} – ${dayMonthLabel(range.to)}`;
}

/**
 * What a typed operation is, in the user's words. Shown beside the model's prose so the
 * confirmation is about the edit that will actually run, not about the sentence above it.
 */
export function operationLabel(operation: Operation): string {
	const when = rangeLabel(operation.dates);
	switch (operation.kind) {
		case 'adjust-product-amount': {
			const direction = operation.deltaAmount < 0 ? 'Less' : 'More';
			return `${direction} of one product · ${when}`;
		}
		case 'add-item':
			return `Add ${operation.item.kind === 'meal' ? 'a meal' : 'a product'} · ${when}`;
		case 'remove-item':
			return `Remove ${operation.item.kind === 'meal' ? 'a meal' : 'a product'} · ${when}`;
		case 'swap-meal':
			return `Swap one meal for another · ${when}`;
		case 'assign-plan':
			return `Put a day plan on the day · ${when}`;
		case 'clear-plan':
			return `Leave the day unplanned · ${when}`;
	}
}

/**
 * Whether a previewed day has already happened.
 *
 * The rules refuse a past date unless the request asked for one, so a day flagged here
 * passed that check — but a request only has to mention yesterday to pass it, and
 * rewriting a day that was already eaten is not the decision planning a future one is.
 * Flagged rather than blocked: the confirmation is the right place for it.
 */
export function isPastDay(day: PreviewDay, today: IsoDate): boolean {
	return day.date < today;
}

export type PreviewSummary = {
	dayCount: number;
	/** How many of those days get their own copy of their plan when this is applied. */
	detachCount: number;
	/** How many of those days have already happened. */
	pastCount: number;
	/** True when the assistant proposed nothing that would change a day. */
	isEmpty: boolean;
	dates: IsoDate[];
	/** The one line above Apply and Discard. */
	label: string;
};

export function summarisePreview(preview: Preview, today: IsoDate): PreviewSummary {
	const dates = preview.days.map((day) => day.date);
	const dayCount = dates.length;
	const detachCount = preview.days.filter((day) => day.detaches).length;
	const pastCount = preview.days.filter((day) => isPastDay(day, today)).length;
	const days = dayCount === 1 ? 'day' : 'days';

	// A past day is named in the line the user reads before pressing Apply, because it is
	// the one change the copy-on-write cannot make harmless: history is what moves.
	const label =
		dayCount === 0
			? 'Nothing would change.'
			: pastCount === 0
				? `${dayCount} ${days} would change.`
				: `${dayCount} ${days} would change, ${pastCount} already past.`;
	return { dayCount, detachCount, pastCount, isEmpty: dayCount === 0, dates, label };
}

/**
 * How much each macro moves on a day. The preview carries the day before and after, so
 * this is the figure the user is actually deciding about.
 */
export function dayDelta(day: PreviewDay): Macros {
	return deviation(day.after, day.before);
}

/**
 * The one line that says how to put an applied change back by hand.
 *
 * There is no undo command. The log keeps what each date pointed at before, which is all
 * reversing it takes: assign that plan again, or clear the date when it had none.
 */
export function reversalHint(change: AppliedChange): string {
	if (!change.previousDayPlanId) {
		return 'Was unplanned — clear the day to put it back.';
	}
	const label = change.previousLabel ?? 'a plan that has since been deleted';
	return `Was ${label} — assign day plan ${change.previousDayPlanId} to that date to put it back.`;
}

/** What one entry of the log did, in one line. */
export function recordSummary(record: AssistantRecord): string {
	const count = record.changes.length;
	const days = `${count} ${count === 1 ? 'day' : 'days'}`;
	const detached = record.changes.filter((change) => change.detached).length;
	if (detached === 0) return `${days} changed`;
	return `${days} changed · ${detached} given their own copy of the plan`;
}

/**
 * The plans a calendar view needs to join its dates to, which is the plan list plus the
 * day-local copies the dates in view point at.
 *
 * The list leaves copies out on purpose — they are not plans the user keeps — so without
 * this a date the assistant detached would read as "plan removed".
 */
export function joinablePlans<T extends { id: string }>(
	kept: readonly T[],
	assigned: readonly T[],
): T[] {
	const ids = new Set(kept.map((plan) => plan.id));
	return [...kept, ...assigned.filter((plan) => !ids.has(plan.id))];
}
