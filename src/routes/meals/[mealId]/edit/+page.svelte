<script lang="ts">
	import { runMutation, updateMeal } from '$lib/api';
	import MealForm from '$lib/components/MealForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { toMealDraft } from '$lib/domain/meal';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const meal = $derived(data.meal);
</script>

<PageHeader title="Edit meal" back="/meals/{meal.id}" />

{#key meal.updatedAt}
	<MealForm
		initial={toMealDraft(meal)}
		products={data.products}
		categories={data.categories}
		submitLabel="Save"
		onSubmit={(draft) =>
			runMutation(() => updateMeal(meal.id, draft), {
				success: 'Meal saved',
				redirectTo: `/meals/${meal.id}`,
			})}
	/>
{/key}
