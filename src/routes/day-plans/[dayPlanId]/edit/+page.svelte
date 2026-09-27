<script lang="ts">
	import { runMutation, updateDayPlan } from '$lib/api';
	import DayPlanForm from '$lib/components/DayPlanForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { toDayPlanDraft } from '$lib/domain/day-plan';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const plan = $derived(data.dayPlan);
</script>

<PageHeader title="Edit day plan" back="/day-plans/{plan.id}" />

{#key plan.updatedAt}
	<DayPlanForm
		initial={toDayPlanDraft(plan)}
		meals={data.meals}
		products={data.products}
		categories={data.categories}
		mealCategories={data.mealCategories}
		targets={data.targets}
		submitLabel="Save"
		onSubmit={(draft) =>
			runMutation(() => updateDayPlan(plan.id, draft), {
				success: 'Day plan saved',
				redirectTo: `/day-plans/${plan.id}`,
			})}
	/>
{/key}
