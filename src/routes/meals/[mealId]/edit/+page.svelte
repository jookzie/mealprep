<script lang="ts">
	import { runMutation, updateMeal } from '$lib/api';
	import MealForm, { type MealFormState } from '$lib/components/MealForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { toMealDraft } from '$lib/domain/meal';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const meal = $derived(data.meal);

	const draft = $derived(keepDraft<MealFormState>(`meal:${meal.id}:${meal.updatedAt}`));
</script>

<PageHeader title="Edit meal" back="/meals/{meal.id}" backLabel={meal.label} />

{#key meal.updatedAt}
	<MealForm
		initial={toMealDraft(meal)}
		products={data.products}
		categories={data.categories}
		{draft}
		submitLabel="Save"
		onSubmit={(submitted) =>
			forgetOnSave(
				draft,
				runMutation(() => updateMeal(meal.id, submitted), {
					success: 'Meal saved',
					redirectTo: `/meals/${meal.id}`,
				}),
			)}
	/>
{/key}
