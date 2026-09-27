<script lang="ts">
	import { createMeal, runMutation } from '$lib/api';
	import MealForm from '$lib/components/MealForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { emptyMealDraft } from '$lib/domain/meal';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<PageHeader title="New meal" back="/meals" />

<MealForm
	initial={emptyMealDraft()}
	products={data.products}
	categories={data.categories}
	submitLabel="Create meal"
	onSubmit={(draft) =>
		runMutation(() => createMeal(draft), {
			success: 'Meal created',
			redirectTo: (meal) => `/meals/${meal.id}`,
		})}
/>
