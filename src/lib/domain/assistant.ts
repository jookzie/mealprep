import type {
	AppliedChange,
	AppliedEntityChange,
	AssistantRecord,
	DateRange,
	EntityKind,
	Macros,
	Preview,
	PreviewDay,
	PreviewEntity,
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

const ENTITY_NAMES: Record<EntityKind, string> = {
	product: 'product',
	meal: 'meal',
	'day-plan': 'day plan',
	targets: 'targets',
};

/** "New meal" or "Changed product", the heading of one entity in a preview or the log. */
export function entityHeading(entity: Pick<PreviewEntity, 'kind' | 'action'>): string {
	const name = ENTITY_NAMES[entity.kind];
	return entity.action === 'create' ? `New ${name}` : `Changed ${name}`;
}

/**
 * The one line that says how far an update reaches, or nothing for a create. An update
 * to a meal or plan moves every day that uses it, the past included, which is the thing
 * worth saying before Apply.
 */
export function entityReach(entity: PreviewEntity): string | null {
	if (entity.action === 'create') return null;
	// Every day is measured against the targets, so a count would say nothing.
	if (entity.kind === 'targets') return null;
	if (entity.datesUsing === 0) return 'No day in view uses it yet.';
	const days = entity.datesUsing === 1 ? 'day' : 'days';
	return `${entity.datesUsing} ${days} in view use it, and all of them follow.`;
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
	/** How many products, meals and day plans it creates or changes. */
	entityCount: number;
	dayCount: number;
	/** How many of those days get their own copy of their plan when this is applied. */
	detachCount: number;
	/** How many of those days have already happened. */
	pastCount: number;
	/** True when the assistant proposed nothing that would change anything. */
	isEmpty: boolean;
	/**
	 * True when it proposed nothing and said why — the answer to a question, or why a
	 * change could not be made. The prose is then the whole of the reply.
	 */
	isAnswer: boolean;
	dates: IsoDate[];
	/** The one line above Apply and Discard. */
	label: string;
};

function plural(count: number, one: string, many: string): string {
	return `${count} ${count === 1 ? one : many}`;
}

export function summarisePreview(preview: Preview, today: IsoDate): PreviewSummary {
	const dates = preview.days.map((day) => day.date);
	const dayCount = dates.length;
	const entityCount = preview.entities.length;
	const detachCount = preview.days.filter((day) => day.detaches).length;
	const pastCount = preview.days.filter((day) => isPastDay(day, today)).length;

	// A past day is named in the line the user reads before pressing Apply, because it is
	// the one change the copy-on-write cannot make harmless: history is what moves.
	const parts = [
		...(entityCount > 0 ? [plural(entityCount, 'item', 'items')] : []),
		...(dayCount > 0 ? [plural(dayCount, 'day', 'days')] : []),
	];
	const isEmpty = parts.length === 0;
	const past = pastCount > 0 ? `, ${pastCount} already past` : '';
	const label = isEmpty ? 'Nothing would change.' : `${parts.join(' and ')} would change${past}.`;
	const isAnswer = isEmpty && preview.proposal.rationale.trim() !== '';
	return { entityCount, dayCount, detachCount, pastCount, isEmpty, isAnswer, dates, label };
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
 * reversing it takes: put that plan on the day again, or clear it when it had none.
 */
export function reversalHint(change: AppliedChange): string {
	if (!change.previousDayPlanId) {
		return 'Was unplanned — clear the day on the calendar to undo.';
	}
	if (!change.previousLabel) {
		return 'Was on a plan that has since been deleted.';
	}
	return `Was ${change.previousLabel} — put ${change.previousLabel} back on the day to undo.`;
}

/** "New meal: Oat bowl", one entity of the log in a line. */
export function appliedEntityLine(change: AppliedEntityChange): string {
	return `${entityHeading(change)}: ${change.label}`;
}

/** What one entry of the log did, in one line. */
export function recordSummary(record: AssistantRecord): string {
	const entities = record.entityChanges ?? [];
	const created = entities.filter((change) => change.action === 'create').length;
	const updated = entities.length - created;
	const parts = [
		...(created > 0 ? [`${plural(created, 'item', 'items')} added`] : []),
		...(updated > 0 ? [`${plural(updated, 'item', 'items')} changed`] : []),
		...(record.changes.length > 0 ? [`${plural(record.changes.length, 'day', 'days')} changed`] : []),
	];
	return parts.length === 0 ? 'Nothing changed' : parts.join(' · ');
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
