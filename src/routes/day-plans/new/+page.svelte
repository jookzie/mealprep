<script lang="ts">
	import { createDayPlan, runMutation } from '$lib/api';
	import DayPlanForm from '$lib/components/DayPlanForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { emptyDayPlanDraft } from '$lib/domain/day-plan';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<PageHeader title="New day plan" back="/day-plans" />

<DayPlanForm
	initial={emptyDayPlanDraft()}
	meals={data.meals}
	products={data.products}
	categories={data.categories}
	mealCategories={data.mealCategories}
	targets={data.targets}
	submitLabel="Create day plan"
	onSubmit={(draft) =>
		runMutation(() => createDayPlan(draft), {
			success: 'Day plan created',
			redirectTo: (plan) => `/day-plans/${plan.id}`,
		})}
/>
