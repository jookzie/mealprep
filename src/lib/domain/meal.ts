import type { Cost, Macros, Meal, MealDraft, Product, Serving, Unit } from '../api/types';
import { addCost, servingCost, ZERO_COST } from './cost';
import { addMacros, servingMacros, ZERO_MACROS } from './macros';

export type ServingRow = {
	productId: string;
	/** Undefined when the product has been deleted; the row still renders. */
	product: Product | undefined;
	amount: number;
	unit: Unit | undefined;
	/** Zero for a deleted product, whose per-100 figures are no longer available. */
	macros: Macros;
	/** Zero and incomplete when the product is unpriced, or gone. */
	cost: Cost;
};

/**
 * A serving carries only a product id, so every meal view joins against the product
 * list. Products are soft-deleted and then never returned, so a hit is not guaranteed.
 */
export function servingRows(
	servings: readonly Serving[],
	products: readonly Product[],
): ServingRow[] {
	const byId = new Map(products.map((product) => [product.id, product]));
	return servings.map((serving) => {
		const product = byId.get(serving.productId);
		return {
			productId: serving.productId,
			product,
			amount: serving.amount,
			unit: product?.unit,
			macros: product ? servingMacros(product.macros, serving.amount) : ZERO_MACROS,
			cost: servingCost(product, serving.amount),
		};
	});
}

/**
 * The total of an unsaved draft, for preview only. A saved meal carries the backend's
 * own figure and must be displayed from that instead.
 */
export function draftMacros(rows: readonly ServingRow[]): Macros {
	return rows.reduce((sum, row) => addMacros(sum, row.macros), ZERO_MACROS);
}

/** The cost of that same unsaved draft, on the same terms. */
export function draftCost(rows: readonly ServingRow[]): Cost {
	return rows.reduce((sum, row) => addCost(sum, row.cost), ZERO_COST);
}

export function toMealDraft(meal: Meal): MealDraft {
	return {
		label: meal.label,
		categoryId: meal.categoryId,
		servings: meal.servings.map((serving) => ({ ...serving })),
	};
}

export function emptyMealDraft(): MealDraft {
	return { label: '', servings: [] };
}
