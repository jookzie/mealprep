<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { type DayPlan, deleteDayPlan } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';

	let { dayPlans }: { dayPlans: DayPlan[] } = $props();
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head>Day plan</Table.Head>
			<Table.Head class="text-right">Energy</Table.Head>
			<Table.Head class="text-right">Protein</Table.Head>
			<Table.Head class="text-right">Fat</Table.Head>
			<Table.Head class="text-right">Carbs</Table.Head>
			<Table.Head class="w-1"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each dayPlans as plan (plan.id)}
			<Table.Row>
				<Table.Cell>
					<a href="/day-plans/{plan.id}" class="font-medium hover:underline">{plan.label}</a>
					<p class="text-muted-foreground text-xs">
						{plan.meals.length}
						{plan.meals.length === 1 ? 'meal' : 'meals'}
					</p>
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatKcal(plan.macros.energyKcal)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(plan.macros.proteinG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(plan.macros.fatG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(plan.macros.carbohydratesG)}
				</Table.Cell>
				<Table.Cell>
					<div class="flex items-center justify-end gap-1">
						<Button href="/day-plans/{plan.id}/edit" variant="ghost" size="icon" title="Edit">
							<PencilIcon class="size-4" />
						</Button>
						<DeleteAction
							name={plan.label}
							size="icon"
							description="The plan is removed. Calendar days holding it will show it as removed."
							action={() => deleteDayPlan({ path: { dayPlanId: plan.id }, throwOnError: true })}
						/>
					</div>
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
