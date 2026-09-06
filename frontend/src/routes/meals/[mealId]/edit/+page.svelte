<script lang="ts">
	import { type MealDraft, runMutation, updateMeal } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MealForm from '$lib/components/meal/meal-form.svelte';
	import * as Card from '$lib/components/ui/card';
	import { toMealDraft } from '$lib/domain/meal';

	let { data } = $props();

	const initial = $derived(toMealDraft(data.meal));

	async function save(draft: MealDraft) {
		await runMutation(
			() => updateMeal({ path: { mealId: data.meal.id }, body: draft, throwOnError: true }),
			{ success: 'Meal saved', redirectTo: `/meals/${data.meal.id}` }
		);
	}
</script>

<PageHeader
	title="Edit {data.meal.label}"
	description="The label and the whole set of servings are replaced on save."
/>

<Card.Root>
	<Card.Content class="pt-6">
		{#key data.meal.updatedAt}
			<MealForm {initial} products={data.products} submitLabel="Save meal" onSubmit={save} />
		{/key}
	</Card.Content>
</Card.Root>
