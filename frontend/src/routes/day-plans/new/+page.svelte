<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { createDayPlan, type DayPlanDraft, runMutation } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import DayPlanForm from '$lib/components/day-plan/day-plan-form.svelte';
	import * as Card from '$lib/components/ui/card';
	import { emptyDayPlanDraft } from '$lib/domain/day-plan';

	let { data } = $props();

	// A day plan created from the calendar returns there, so the user can assign it
	// straight away rather than navigating back by hand.
	const returnTo = $derived(page.url.searchParams.get('returnTo'));

	async function create(draft: DayPlanDraft) {
		const created = await runMutation(() => createDayPlan({ body: draft, throwOnError: true }), {
			success: 'Day plan created'
		});
		if (created) await goto(returnTo ?? `/day-plans/${created.data.dayPlan.id}`);
	}
</script>

<PageHeader title="New day plan" description="A label and the meals it groups." />

<Card.Root>
	<Card.Content class="pt-6">
		<DayPlanForm
			initial={emptyDayPlanDraft()}
			meals={data.meals}
			submitLabel="Create day plan"
			onSubmit={create}
		/>
	</Card.Content>
</Card.Root>
