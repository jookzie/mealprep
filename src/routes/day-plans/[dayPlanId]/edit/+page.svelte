<script lang="ts">
	import { runMutation, updateDayPlan } from '$lib/api';
	import DayPlanForm, { type DayPlanFormState } from '$lib/components/DayPlanForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { toDayPlanDraft } from '$lib/domain/day-plan';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const plan = $derived(data.dayPlan);

	const draft = $derived(keepDraft<DayPlanFormState>(`day-plan:${plan.id}:${plan.updatedAt}`));
</script>

<PageHeader title="Edit day plan" back="/day-plans/{plan.id}" backLabel={plan.label} />

{#key plan.updatedAt}
	<DayPlanForm
		initial={toDayPlanDraft(plan)}
		meals={data.meals}
		products={data.products}
		categories={data.categories}
		mealCategories={data.mealCategories}
		targets={data.targets}
		{draft}
		submitLabel="Save"
		onSubmit={(submitted) =>
			forgetOnSave(
				draft,
				runMutation(() => updateDayPlan(plan.id, submitted), {
					success: 'Day plan saved',
					redirectTo: `/day-plans/${plan.id}`,
				}),
			)}
	/>
{/key}
