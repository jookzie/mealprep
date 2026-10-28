import { describe, expect, test } from 'bun:test';
import type {
	Macros,
	Preview,
	PreviewDay,
	PreviewEntity,
} from '../api/types';
import {
	type ChatMessage,
	chatHistory,
	dayDelta,
	entityHeading,
	entityReach,
	isPastDay,
	joinablePlans,
	rangeDates,
	rangeLabel,
	summarisePreview,
} from './assistant';

function macros(energyKcal: number, proteinG = 20): Macros {
	return { energyKcal, fatG: 10, proteinG, carbohydratesG: 30 };
}

function day(date: string, overrides: Partial<PreviewDay> = {}): PreviewDay {
	return {
		date,
		label: 'Training day',
		changes: ['Whey 60 g → 20 g'],
		before: macros(2200, 160),
		after: macros(2048, 130),
		detaches: true,
		...overrides,
	};
}

function preview(days: PreviewDay[], entities: PreviewEntity[] = []): Preview {
	return {
		proposal: { request: 'next 4 days I have 40 g less whey', rationale: 'Done.', operations: [] },
		entities,
		days,
		targets: macros(2000, 140),
	};
}

function entity(overrides: Partial<PreviewEntity> = {}): PreviewEntity {
	return {
		kind: 'meal',
		action: 'create',
		label: 'Oat bowl',
		changes: ['Made of 80 g Oats, 30 g Whey'],
		macros: macros(410, 35),
		datesUsing: 0,
		...overrides,
	};
}

describe('rangeDates and rangeLabel', () => {
	test('a span covers every date in it and reads as a span', () => {
		const range = { from: '2026-10-20', to: '2026-10-23' };
		expect(rangeDates(range)).toEqual(['2026-10-20', '2026-10-21', '2026-10-22', '2026-10-23']);
		expect(rangeLabel(range)).toContain('–');
	});

	test('one day is one date and reads as one day', () => {
		const range = { from: '2026-10-20', to: '2026-10-20' };
		expect(rangeDates(range)).toEqual(['2026-10-20']);
		expect(rangeLabel(range)).not.toContain('–');
	});

	test('a range that runs backwards covers nothing', () => {
		const range = { from: '2026-10-23', to: '2026-10-20' };
		expect(rangeDates(range)).toEqual([]);
		expect(rangeLabel(range)).toBe('no days');
	});
});

describe('entities', () => {
	test('a heading says whether the thing is new or changed, in plain words', () => {
		expect(entityHeading(entity())).toBe('New meal');
		expect(entityHeading(entity({ kind: 'day-plan', action: 'update' }))).toBe('Changed day plan');
		expect(entityHeading(entity({ kind: 'product' }))).toBe('New product');
	});

	test('an update says how many days follow it; a create says nothing', () => {
		expect(entityReach(entity())).toBeNull();
		expect(entityReach(entity({ action: 'update', datesUsing: 8 }))).toBe(
			'8 days in view use it, and all of them follow.',
		);
		expect(entityReach(entity({ action: 'update', datesUsing: 1 }))).toStartWith('1 day ');
		expect(entityReach(entity({ action: 'update', datesUsing: 0 }))).toBe(
			'No day in view uses it yet.',
		);
	});

	test('the targets read as targets and claim no reach, since every day is measured against them', () => {
		const targets = entity({ kind: 'targets', action: 'update', label: 'Daily targets' });
		expect(entityHeading(targets)).toBe('Changed targets');
		expect(entityReach(targets)).toBeNull();
	});
});

describe('summarisePreview', () => {
	test('counts the days and the ones that get their own copy', () => {
		const summary = summarisePreview(
			preview([day('2026-10-20'), day('2026-10-21', { detaches: false })]),
			'2026-10-20',
		);
		expect(summary.dayCount).toBe(2);
		expect(summary.detachCount).toBe(1);
		expect(summary.isEmpty).toBe(false);
		expect(summary.dates).toEqual(['2026-10-20', '2026-10-21']);
		expect(summary.label).toBe('2 days would change.');
	});

	test('one day reads in the singular', () => {
		expect(summarisePreview(preview([day('2026-10-20')]), '2026-10-20').label).toBe(
			'1 day would change.',
		);
	});

	test('a proposal that changes nothing says so rather than offering an apply', () => {
		const summary = summarisePreview(preview([]), '2026-10-20');
		expect(summary.isEmpty).toBe(true);
		expect(summary.dayCount).toBe(0);
		expect(summary.label).toBe('Nothing would change.');
		// It said why, so the prose is the reply.
		expect(summary.isAnswer).toBe(true);
	});

	test('a reply with no prose and nothing to change is not an answer', () => {
		const silent = preview([]);
		silent.proposal.rationale = '  ';
		expect(summarisePreview(silent, '2026-10-20').isAnswer).toBe(false);
		expect(summarisePreview(preview([day('2026-10-20')]), '2026-10-20').isAnswer).toBe(false);
	});

	test('counts new and changed things alongside the days', () => {
		const both = summarisePreview(preview([day('2026-10-20')], [entity(), entity()]), '2026-10-20');
		expect(both.entityCount).toBe(2);
		expect(both.isEmpty).toBe(false);
		expect(both.label).toBe('2 items and 1 day would change.');

		const only = summarisePreview(preview([], [entity()]), '2026-10-20');
		expect(only.isEmpty).toBe(false);
		expect(only.label).toBe('1 item would change.');
	});
});

describe('past days', () => {
	test('a day before today is flagged, today and later are not', () => {
		expect(isPastDay(day('2026-10-19'), '2026-10-20')).toBe(true);
		expect(isPastDay(day('2026-10-20'), '2026-10-20')).toBe(false);
		expect(isPastDay(day('2026-10-21'), '2026-10-20')).toBe(false);
	});

	test('the line above Apply counts them, because that is the change history cannot undo', () => {
		const summary = summarisePreview(
			preview([day('2026-10-19'), day('2026-10-20'), day('2026-10-21')]),
			'2026-10-20',
		);
		expect(summary.pastCount).toBe(1);
		expect(summary.label).toBe('3 days would change, 1 already past.');
	});
});

describe('chatHistory', () => {
	function message(id: number, status: ChatMessage['status']): ChatMessage {
		return {
			id,
			request: `request ${id}`,
			preview: status === 'asking' ? null : preview([]),
			status,
		};
	}

	test('reports each settled turn with its reply, in order', () => {
		const history = chatHistory([
			message(1, 'applied'),
			message(2, 'answered'),
			message(3, 'discarded'),
		]);
		expect(history).toEqual([
			{ request: 'request 1', reply: 'Done.', outcome: 'applied' },
			{ request: 'request 2', reply: 'Done.', outcome: 'answered' },
			{ request: 'request 3', reply: 'Done.', outcome: 'discarded' },
		]);
	});

	test('a change still waiting is replaced by asking again, and one being asked is not yet a turn', () => {
		const history = chatHistory([message(1, 'proposed'), message(2, 'asking')]);
		expect(history).toEqual([{ request: 'request 1', reply: 'Done.', outcome: 'discarded' }]);
	});
});

describe('dayDelta', () => {
	test('is what the day moves by, not the distance from the target', () => {
		const delta = dayDelta(day('2026-10-20'));
		expect(delta.energyKcal).toBe(-152);
		expect(delta.proteinG).toBe(-30);
		expect(delta.fatG).toBe(0);
	});
});

describe('joinablePlans', () => {
	test('adds the day-local copies the dates point at without duplicating the list', () => {
		const kept = [{ id: 'plan-1' }, { id: 'plan-2' }];
		const assigned = [{ id: 'plan-1' }, { id: 'copy-1' }];

		expect(joinablePlans(kept, assigned).map((plan) => plan.id)).toEqual([
			'plan-1',
			'plan-2',
			'copy-1',
		]);
	});

	test('is the list itself when nothing has been detached', () => {
		const kept = [{ id: 'plan-1' }];
		expect(joinablePlans(kept, [{ id: 'plan-1' }])).toEqual(kept);
	});
});
