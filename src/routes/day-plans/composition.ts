import { listCategories, listMeals, listProducts, loadTargets, read } from '$lib/api';

/** Everything a day plan editor picks from. */
export async function loadComposition() {
	const [meals, products, categories, mealCategories, targets] = await read(
		Promise.all([
			listMeals(),
			listProducts(),
			listCategories('day-plan'),
			listCategories('meal'),
			loadTargets(),
		]),
	);
	return { meals, products, categories, mealCategories, targets };
}
