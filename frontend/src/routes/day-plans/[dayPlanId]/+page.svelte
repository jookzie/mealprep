<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteDayPlan } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacrosDelta from '$lib/components/macros/macros-delta.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import TargetProgress from '$lib/components/macros/target-progress.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';

	let { data } = $props();
</script>

<PageHeader title={data.dayPlan.label}>
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
	</Card.Header>
	<Card.Content class="space-y-6">
		<MacrosSummary macros={data.dayPlan.macros} variant="grid" />
		{#if data.targets}
			<div class="space-y-4">
				<TargetProgress actual={data.dayPlan.macros} target={data.targets.macros} />
				<MacrosDelta actual={data.dayPlan.macros} target={data.targets.macros} />
			</div>
		{:else}
			<p class="text-muted-foreground text-sm">
				<a href="/targets" class="underline underline-offset-4">Set your daily targets</a>
				to see this plan measured against them.
			</p>
		{/if}
	</Card.Content>
</Card.Root>

<Card.Root>
	<Card.Header>
		<Card.Title>Meals</Card.Title>
	</Card.Header>
	<Card.Content>
		{#if data.dayPlan.meals.length === 0}
			<p class="text-muted-foreground text-sm">This plan has no meals.</p>
		{:else}
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Meal</Table.Head>
						<Table.Head class="text-right">Energy</Table.Head>
						<Table.Head class="text-right">Protein</Table.Head>
						<Table.Head class="text-right">Fat</Table.Head>
						<Table.Head class="text-right">Carbs</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each data.dayPlan.meals as meal (meal.id)}
						<Table.Row>
							<Table.Cell>
								<a href="/meals/{meal.id}" class="font-medium hover:underline">{meal.label}</a>
							</Table.Cell>
							<Table.Cell class="text-right tabular-nums">
								{formatKcal(meal.macros.energyKcal)}
							</Table.Cell>
							<Table.Cell class="text-right tabular-nums">
								{formatGrams(meal.macros.proteinG)}
							</Table.Cell>
							<Table.Cell class="text-right tabular-nums">{formatGrams(meal.macros.fatG)}</Table.Cell
							>
							<Table.Cell class="text-right tabular-nums">
								{formatGrams(meal.macros.carbohydratesG)}
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		{/if}
	</Card.Content>
</Card.Root>
