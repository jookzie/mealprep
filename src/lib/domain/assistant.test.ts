import { describe, expect, test } from 'bun:test';
import type {
	AppliedChange,
	AssistantRecord,
	Macros,
	Operation,
	Preview,
	PreviewDay,
} from '../api/types';
import {
	dayDelta,
	isPastDay,
	joinablePlans,
	operationLabel,
	rangeDates,
	rangeLabel,
	recordSummary,
	reversalHint,
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

function preview(days: PreviewDay[]): Preview {
	return {
		proposal: { request: 'next 4 days I have 40 g less whey', rationale: 'Done.', operations: [] },
		days,
		targets: macros(2000, 140),
	};
}

function change(overrides: Partial<AppliedChange> = {}): AppliedChange {
	return {
		date: '2026-10-20',
		previousDayPlanId: 'plan-1',
		previousLabel: 'Training day',
		dayPlanId: 'copy-1',
		label: 'Training day',
		detached: true,
		changes: ['Whey 60 g → 20 g'],
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

describe('operationLabel', () => {
	const dates = { from: '2026-10-20', to: '2026-10-20' };

	test('says what each kind of edit does and when', () => {
		const operations: Operation[] = [
			{ kind: 'adjust-product-amount', dates, productId: 'p', deltaAmount: -40 },
			{ kind: 'adjust-product-amount', dates, productId: 'p', deltaAmount: 40 },
			{ kind: 'add-item', dates, item: { kind: 'meal', mealId: 'm' } },
			{ kind: 'remove-item', dates, item: { kind: 'product', productId: 'p' } },
			{ kind: 'swap-meal', dates, fromMealId: 'a', toMealId: 'b' },
			{ kind: 'assign-plan', dates, dayPlanId: 'd' },
			{ kind: 'clear-plan', dates },
		];

		for (const operation of operations) {
			const label = operationLabel(operation);
			expect(label.length).toBeGreaterThan(0);
			expect(label).toContain(rangeLabel(dates));
		}

		expect(operationLabel(operations[0])).toStartWith('Less');
		expect(operationLabel(operations[1])).toStartWith('More');
		expect(operationLabel(operations[2])).toContain('meal');
		expect(operationLabel(operations[3])).toContain('product');
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

describe('dayDelta', () => {
	test('is what the day moves by, not the distance from the target', () => {
		const delta = dayDelta(day('2026-10-20'));
		expect(delta.energyKcal).toBe(-152);
		expect(delta.proteinG).toBe(-30);
		expect(delta.fatG).toBe(0);
	});
});

describe('reversalHint', () => {
	test('names the plan to assign again', () => {
		const hint = reversalHint(change());
		expect(hint).toContain('Training day');
		expect(hint).toContain('plan-1');
	});

	test('says to clear the day when it had nothing', () => {
		expect(reversalHint(change({ previousDayPlanId: undefined, previousLabel: undefined }))).toBe(
			'Was unplanned — clear the day to put it back.',
		);
	});

	test('keeps the id when the plan it names has since been deleted', () => {
		const hint = reversalHint(change({ previousLabel: undefined }));
		expect(hint).toContain('plan-1');
		expect(hint).toContain('deleted');
	});
});

describe('recordSummary', () => {
	function record(changes: AppliedChange[]): AssistantRecord {
		return {
			id: 'r1',
			request: 'next 4 days I have 40 g less whey',
			rationale: 'Done.',
			operations: [],
			changes,
			createdAt: '2026-10-20T08:00:00Z',
		};
	}

	test('counts the days and how many were detached', () => {
		expect(recordSummary(record([change(), change({ date: '2026-10-21' })]))).toBe(
			'2 days changed · 2 given their own copy of the plan',
		);
	});

	test('leaves the copy clause out when nothing was detached', () => {
		expect(recordSummary(record([change({ detached: false })]))).toBe('1 day changed');
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
