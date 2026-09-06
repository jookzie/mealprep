import type { DayPlan, DayPlanDraft, Meal } from '../api/gen/types.gen';

/**
 * A day plan reads back as a list of meal objects but is written as a list of ids.
 * This is the one place that asymmetry is bridged.
 */
export function toDayPlanDraft(plan: DayPlan): DayPlanDraft {
	return {
		label: plan.label,
		mealIds: plan.meals.map((meal) => meal.id),
	};
}

export function emptyDayPlanDraft(): DayPlanDraft {
	return { label: '', mealIds: [] };
}

/** Undefined entries are meals that have since been deleted. */
export function resolveMeals(
	mealIds: readonly string[],
	meals: readonly Meal[],
): (Meal | undefined)[] {
	const byId = new Map(meals.map((meal) => [meal.id, meal]));
	return mealIds.map((id) => byId.get(id));
}
