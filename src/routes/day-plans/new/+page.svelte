<script lang="ts">
	import { createDayPlan, runMutation } from '$lib/api';
	import DayPlanForm, { type DayPlanFormState } from '$lib/components/DayPlanForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { emptyDayPlanDraft } from '$lib/domain/day-plan';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const draft = $derived(keepDraft<DayPlanFormState>('day-plan:new'));
</script>

<PageHeader title="New day plan" back="/day-plans" backLabel="Day plans" />

<DayPlanForm
	initial={emptyDayPlanDraft()}
	meals={data.meals}
	products={data.products}
	categories={data.categories}
	mealCategories={data.mealCategories}
	targets={data.targets}
	{draft}
	submitLabel="Create day plan"
	onSubmit={(submitted) =>
		forgetOnSave(
			draft,
			runMutation(() => createDayPlan(submitted), {
				success: 'Day plan created',
				redirectTo: (plan) => `/day-plans/${plan.id}`,
			}),
		)}
/>
