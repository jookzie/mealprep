<script lang="ts">
	import { type DayPlanDraft, runMutation, updateDayPlan } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import DayPlanForm from '$lib/components/day-plan/day-plan-form.svelte';
	import { toDayPlanDraft } from '$lib/domain/day-plan';

	let { data } = $props();

	// Reads return meal objects; writes take ids. toDayPlanDraft is the one bridge.
	const initial = $derived(toDayPlanDraft(data.dayPlan));

	async function save(draft: DayPlanDraft) {
		await runMutation(
			() =>
				updateDayPlan({
					path: { dayPlanId: data.dayPlan.id },
					body: draft,
					throwOnError: true
				}),
			{ success: 'Day plan saved', redirectTo: `/day-plans/${data.dayPlan.id}` }
		);
	}
</script>

<PageHeader
	title="Edit {data.dayPlan.label}"
	description="The label and the whole set of meals are replaced on save."
/>

{#key data.dayPlan.updatedAt}
	<DayPlanForm
		{initial}
		meals={data.meals}
		targets={data.targets?.macros ?? null}
		submitLabel="Save day plan"
		onSubmit={save}
	/>
{/key}
