<script lang="ts">
	import LayersIcon from '@lucide/svelte/icons/layers';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import DayPlanList from '$lib/components/day-plan/day-plan-list.svelte';
	import { Button } from '$lib/components/ui/button';
	import { maxima } from '$lib/domain/radar';
	import { sortRows } from '$lib/domain/sort';

	let { data } = $props();

	const shown = $derived(sortRows(data.dayPlans, { key: 'name', direction: 'asc' }));

	// Without targets there is no shared denominator, so the largest value on each axis
	// across the plans stands in: the shapes still compare, they just compare to the
	// biggest plan rather than to a goal.
	const relativeTo = $derived(data.targets ? ('target' as const) : ('largest' as const));
	const denominator = $derived(
		data.targets?.macros ?? maxima(data.dayPlans.map((plan) => plan.macros))
	);
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
		description="A day plan is a labelled group of meals, in the order you eat them."
	>
		{#snippet action()}
			<Button href="/day-plans/new">Create a day plan</Button>
		{/snippet}
	</EmptyState>
{:else}
	{#if !data.targets}
		<p class="text-muted-foreground text-sm">
			<a href="/targets" class="underline underline-offset-4">Set your daily targets</a>
			to compare these against them; for now each shape is drawn against the largest plan.
		</p>
	{/if}
	<DayPlanList dayPlans={shown} {denominator} {relativeTo} />
{/if}
