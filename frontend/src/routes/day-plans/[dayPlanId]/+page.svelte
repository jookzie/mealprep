<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteDayPlan } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacroMeters from '$lib/components/macros/macro-meters.svelte';
	import MacroRadar from '$lib/components/macros/macro-radar.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import ServingTable from '$lib/components/meal/serving-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';
	import { formatKcal } from '$lib/domain/format';
	import { servingRows } from '$lib/domain/meal';
	import { maxima } from '$lib/domain/radar';

	let { data } = $props();

	/*
	 * "How much room is left" is the question being asked mid-composition, so the
	 * framing is switchable. Planned is the default because "142 of 150" is more
	 * informative than "8 left" when deciding whether to add another meal.
	 */
	let mode = $state<'planned' | 'remaining'>('planned');

	// With no targets set the plan is drawn against its own largest axis, so the shape
	// still reads even though there is nothing to compare it to.
	const relativeTo = $derived(data.targets ? ('target' as const) : ('largest' as const));
	const denominator = $derived(data.targets?.macros ?? maxima([data.dayPlan.macros]));
</script>

<PageHeader title={data.dayPlan.label} description="A group of meals, in the order they are eaten.">
	{#snippet actions()}
		<Button href="/day-plans/{data.dayPlan.id}/edit" variant="outline">
			<PencilIcon class="size-4" />
			Edit
		</Button>
		<DeleteAction
			name={data.dayPlan.label}
			variant="outline"
			size="default"
			description="The plan is removed. Calendar days holding it will show it as removed."
			action={() => deleteDayPlan({ path: { dayPlanId: data.dayPlan.id }, throwOnError: true })}
			result={{ redirectTo: '/day-plans' }}
		/>
	{/snippet}
</PageHeader>

<Card.Root>
	<Card.Header>
		<Card.Title>Macros</Card.Title>
		<Card.Description>The sum of this plan's meals.</Card.Description>
		{#if data.targets}
			<Card.Action>
				<ToggleGroup.Root
					type="single"
					size="sm"
					variant="outline"
					value={mode}
					onValueChange={(value) => {
						if (value) mode = value as 'planned' | 'remaining';
					}}
				>
					<ToggleGroup.Item value="planned">Planned</ToggleGroup.Item>
					<ToggleGroup.Item value="remaining">Remaining</ToggleGroup.Item>
				</ToggleGroup.Root>
			</Card.Action>
		{/if}
	</Card.Header>
	<Card.Content>
		<div class="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_auto]">
			<div class="space-y-6">
				<MacrosSummary macros={data.dayPlan.macros} variant="grid" />
				{#if data.targets}
					<MacroMeters actual={data.dayPlan.macros} target={data.targets.macros} {mode} />
				{:else}
					<p class="text-muted-foreground text-sm">
						<a href="/targets" class="underline underline-offset-4">Set your daily targets</a>
						to see this plan measured against them.
					</p>
				{/if}
			</div>
			<div class="flex flex-col items-center gap-1 justify-self-center">
				<MacroRadar macros={data.dayPlan.macros} {denominator} {relativeTo} size={200} />
				<p class="text-muted-foreground text-xs">
					{relativeTo === 'target' ? 'Dashed ring is the daily target' : 'Drawn against its own largest figure'}
				</p>
			</div>
		</div>
	</Card.Content>
</Card.Root>

{#if data.dayPlan.meals.length === 0}
	<Card.Root>
		<Card.Content class="pt-6">
			<p class="text-muted-foreground text-sm">This plan has no meals.</p>
		</Card.Content>
	</Card.Root>
{:else}
	{#each data.dayPlan.meals as meal, index (`${meal.id}-${index}`)}
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">
					<span class="text-muted-foreground tabular-nums">{index + 1}.</span>
					<a href="/meals/{meal.id}" class="hover:underline">{meal.label}</a>
				</Card.Title>
				<Card.Description class="tabular-nums">
					{formatKcal(meal.macros.energyKcal)}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				{#if meal.servings.length === 0}
					<p class="text-muted-foreground text-sm">This meal has no servings.</p>
				{:else}
					<ServingTable rows={servingRows(meal.servings, data.products)} />
				{/if}
			</Card.Content>
		</Card.Root>
	{/each}
{/if}
