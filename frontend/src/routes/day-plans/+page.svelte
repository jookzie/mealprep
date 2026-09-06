<script lang="ts">
	import LayersIcon from '@lucide/svelte/icons/layers';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import DayPlanTable from '$lib/components/day-plan/day-plan-table.svelte';
	import { Button } from '$lib/components/ui/button';

	let { data } = $props();
</script>

<PageHeader title="Day plans" description="Groups of meals, ready to put on a date.">
	{#snippet actions()}
		<Button href="/day-plans/new">
			<PlusIcon class="size-4" />
			New day plan
		</Button>
	{/snippet}
</PageHeader>

{#if data.dayPlans.length === 0}
	<EmptyState
		icon={LayersIcon}
		title="No day plans yet"
		description="A day plan is a labelled group of meals, and nothing else."
	>
		{#snippet action()}
			<Button href="/day-plans/new">Create a day plan</Button>
		{/snippet}
	</EmptyState>
{:else}
	<DayPlanTable dayPlans={data.dayPlans} />
{/if}
