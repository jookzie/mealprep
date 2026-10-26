<script lang="ts">
	import { createMeal, runMutation } from '$lib/api';
	import MealForm, { type MealFormState } from '$lib/components/MealForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { emptyMealDraft } from '$lib/domain/meal';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const draft = $derived(keepDraft<MealFormState>('meal:new'));
</script>

<PageHeader title="New meal" back="/meals" />

<MealForm
	initial={emptyMealDraft()}
	products={data.products}
	categories={data.categories}
	{draft}
	submitLabel="Create meal"
	onSubmit={(submitted) =>
		forgetOnSave(
			draft,
			runMutation(() => createMeal(submitted), {
				success: 'Meal created',
				redirectTo: (meal) => `/meals/${meal.id}`,
			}),
		)}
/>
