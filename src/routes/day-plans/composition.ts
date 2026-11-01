import {
	listCategories,
	listMeals,
	listProducts,
	loadTargets,
	loadWeightSeries,
	read,
} from '$lib/api';

/** Everything a day plan editor picks from. */
export async function loadComposition() {
	const [meals, products, categories, mealCategories, targets, weight] = await read(
		Promise.all([
			listMeals(),
			listProducts(),
			listCategories('day-plan'),
			listCategories('meal'),
			loadTargets(),
			loadWeightSeries(),
		]),
	);
	return { meals, products, categories, mealCategories, targets, kilograms: weight.trend };
}
