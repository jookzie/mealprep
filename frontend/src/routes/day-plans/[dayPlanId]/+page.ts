import { getDayPlan, listProducts, loadTargets } from '$lib/api';
import type { PageLoad } from './$types';

// The plan now carries its meals' servings, so the product-level breakdown needs the
// product list to name them but no second read of every meal.
export const load: PageLoad = async ({ fetch, params }) => {
	const [plan, products, targets] = await Promise.all([
		getDayPlan({ fetch, path: { dayPlanId: params.dayPlanId }, throwOnError: true }),
		listProducts({ fetch, throwOnError: true }),
		loadTargets(fetch),
	]);
	return { dayPlan: plan.data.dayPlan, products: products.data.products, targets };
};
