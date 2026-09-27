import type { DayPlan, DayPlanDraft, DayPlanItem, DayPlanItemDraft } from '../api/types';

export function toDayPlanDraft(plan: DayPlan): DayPlanDraft {
	return {
		label: plan.label,
		categoryId: plan.categoryId,
		items: plan.items.map(toItemDraft),
	};
}

function toItemDraft(item: DayPlanItem): DayPlanItemDraft {
	if (item.kind === 'meal') return { kind: 'meal', mealId: item.meal.id };
	return { kind: 'product', productId: item.product.id, amount: item.product.amount };
}

export function emptyDayPlanDraft(): DayPlanDraft {
	return { label: '', items: [] };
}

/** What an item is called wherever a plan is summarised in one line. */
export function itemLabel(item: DayPlanItem): string {
	if (item.kind === 'meal') return item.meal.label;
	return item.product.name ?? 'Product removed';
}
