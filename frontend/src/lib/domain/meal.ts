import type { Macros, Meal, MealDraft, Product, Serving, Unit } from '../api/gen/types.gen';
import { addMacros, servingMacros, ZERO_MACROS } from './macros';

export type ServingRow = {
	productId: string;
	/** Undefined when the product has been deleted; the row still renders. */
	product: Product | undefined;
	amount: number;
	unit: Unit | undefined;
	/** Zero for a deleted product, whose per-100 figures are no longer available. */
	macros: Macros;
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
		};
	});
}

/**
 * The total of an unsaved draft, for preview only. A saved meal carries the API's
 * own figure and must be displayed from that instead.
 */
export function draftMacros(rows: readonly ServingRow[]): Macros {
	return rows.reduce((sum, row) => addMacros(sum, row.macros), ZERO_MACROS);
}

export function toMealDraft(meal: Meal): MealDraft {
	return {
		label: meal.label,
		servings: meal.servings.map((serving) => ({ ...serving })),
	};
}

export function emptyMealDraft(): MealDraft {
	return { label: '', servings: [] };
}
