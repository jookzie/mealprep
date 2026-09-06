<script lang="ts">
	import { goto } from '$app/navigation';
	import { createMeal, type MealDraft, runMutation } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MealForm from '$lib/components/meal/meal-form.svelte';
	import * as Card from '$lib/components/ui/card';
	import { emptyMealDraft } from '$lib/domain/meal';

	let { data } = $props();

	async function create(draft: MealDraft) {
		const created = await runMutation(() => createMeal({ body: draft, throwOnError: true }), {
			success: 'Meal created'
		});
		if (created) await goto(`/meals/${created.data.meal.id}`);
	}
</script>

<PageHeader title="New meal" description="Serving sizes are in each product's own unit." />

<Card.Root>
	<Card.Content class="pt-6">
		<MealForm
			initial={emptyMealDraft()}
			products={data.products}
			submitLabel="Create meal"
			onSubmit={create}
		/>
	</Card.Content>
</Card.Root>
