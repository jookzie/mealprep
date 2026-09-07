<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteMeal, type Meal } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import SortableHead from '$lib/components/app/sortable-head.svelte';
	import MacroCell from '$lib/components/macros/macro-cell.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import { columnScale } from '$lib/domain/heatmap';
	import { MACRO_KEYS, MACRO_LABELS } from '$lib/domain/macros';
	import type { Sort, SortKey } from '$lib/domain/sort';

	// Meals are compared against each other the same way products are, so they get the
	// same table rather than a second layout that happens to show the same figures.
	let { meals, sort, onSort }: { meals: Meal[]; sort: Sort; onSort: (key: SortKey) => void } =
		$props();

	const scales = $derived(
		Object.fromEntries(
			MACRO_KEYS.map((key) => [key, columnScale(meals.map((m) => m.macros[key]))])
		) as Record<(typeof MACRO_KEYS)[number], (value: number) => number>
	);
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<SortableHead label="Meal" sortKey="name" {sort} {onSort} />
			{#each MACRO_KEYS as key (key)}
				<SortableHead label={MACRO_LABELS[key]} sortKey={key} {sort} {onSort} align="right" />
			{/each}
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
				{#each MACRO_KEYS as key (key)}
					<MacroCell
						value={meal.macros[key]}
						macro={key}
						intensity={scales[key](meal.macros[key])}
					/>
				{/each}
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
