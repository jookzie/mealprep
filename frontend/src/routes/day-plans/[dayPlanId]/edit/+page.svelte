<script lang="ts">
	import { type DayPlanDraft, runMutation, updateDayPlan } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import DayPlanForm from '$lib/components/day-plan/day-plan-form.svelte';
	import * as Card from '$lib/components/ui/card';
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

<Card.Root>
	<Card.Content class="pt-6">
		{#key data.dayPlan.updatedAt}
			<DayPlanForm {initial} meals={data.meals} submitLabel="Save day plan" onSubmit={save} />
		{/key}
	</Card.Content>
</Card.Root>
