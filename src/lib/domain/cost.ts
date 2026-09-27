import type { Cost, Product } from '../api/types';

/**
 * Cost arithmetic, mirroring the macro arithmetic beside it.
 *
 * A cost is a figure and a claim about that figure: `complete` is false as soon as
 * one product in the composition carries no price of its own. The two travel
 * together so no view can print the number without the caveat that goes with it.
 */
export const ZERO_COST: Cost = { amount: 0, complete: true };

export function addCost(a: Cost, b: Cost): Cost {
	return { amount: a.amount + b.amount, complete: a.complete && b.complete };
}

export function sumCosts(values: readonly Cost[]): Cost {
	return values.reduce(addCost, ZERO_COST);
}

export function scaleCost(cost: Cost, factor: number): Cost {
	return { amount: cost.amount * factor, complete: cost.complete };
}

/**
 * A product's cost is per 100 of its unit, exactly as its macros are, so a serving
 * costs that scaled by amount/100.
 *
 * An unpriced product adds nothing rather than being left out — a missing price is
 * counted as zero — and marks the total incomplete, which is what turns the figure
 * into a floor the view can flag. A deleted product does the same: its price is as
 * unknowable as one that was never entered.
 */
export function servingCost(product: Product | undefined, amount: number): Cost {
	if (product?.cost === undefined) return { amount: 0, complete: false };
	return { amount: (product.cost * amount) / 100, complete: true };
}

/** A product's own price, or null when it was never given one. */
export function productCost(product: Product): Cost | null {
	return product.cost === undefined ? null : { amount: product.cost, complete: true };
}
