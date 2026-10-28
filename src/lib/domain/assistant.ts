import type {
	ChatTurn,
	DateRange,
	EntityKind,
	Macros,
	Preview,
	PreviewDay,
	PreviewEntity,
	TurnOutcome,
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
 * Where one message of the conversation stands: waiting on the assistant, a change
 * waiting on the user, or settled.
 */
export type MessageStatus = 'asking' | 'proposed' | 'answered' | 'applied' | 'discarded';

export type ChatMessage = {
	id: number;
	request: string;
	/** Absent while the assistant is still thinking. */
	preview: Preview | null;
	status: MessageStatus;
};

/**
 * The earlier turns the next request continues, in order.
 *
 * Only the words go: what an applied change did is already in the plan the request
 * carries. A change still waiting is reported as discarded, because asking again
 * replaces it.
 */
export function chatHistory(messages: readonly ChatMessage[]): ChatTurn[] {
	return messages.flatMap((message) => {
		if (message.status === 'asking' || !message.preview) return [];
		const outcome: TurnOutcome = message.status === 'proposed' ? 'discarded' : message.status;
		return [{ request: message.request, reply: message.preview.proposal.rationale, outcome }];
	});
}

/**
 * How much each macro moves on a day. The preview carries the day before and after, so
 * this is the figure the user is actually deciding about.
 */
export function dayDelta(day: PreviewDay): Macros {
	return deviation(day.after, day.before);
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
