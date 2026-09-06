<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteMeal, type Meal } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';

	let { meals }: { meals: Meal[] } = $props();
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head>Meal</Table.Head>
			<Table.Head class="text-right">Energy</Table.Head>
			<Table.Head class="text-right">Protein</Table.Head>
			<Table.Head class="text-right">Fat</Table.Head>
			<Table.Head class="text-right">Carbs</Table.Head>
			<Table.Head class="w-1"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each meals as meal (meal.id)}
			<Table.Row>
				<Table.Cell>
					<a href="/meals/{meal.id}" class="font-medium hover:underline">{meal.label}</a>
					<p class="text-muted-foreground text-xs">
						{meal.servings.length}
						{meal.servings.length === 1 ? 'serving' : 'servings'}
					</p>
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatKcal(meal.macros.energyKcal)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(meal.macros.proteinG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(meal.macros.fatG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(meal.macros.carbohydratesG)}
				</Table.Cell>
				<Table.Cell>
					<div class="flex items-center justify-end gap-1">
						<Button href="/meals/{meal.id}/edit" variant="ghost" size="icon" title="Edit">
							<PencilIcon class="size-4" />
						</Button>
						<DeleteAction
							name={meal.label}
							size="icon"
							description="The meal is removed. Day plans that use it will show it as removed."
							action={() => deleteMeal({ path: { mealId: meal.id }, throwOnError: true })}
						/>
					</div>
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
