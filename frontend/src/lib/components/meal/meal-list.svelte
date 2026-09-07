<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import UtensilsIcon from '@lucide/svelte/icons/utensils';
	import { deleteMeal, type Meal } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import MacroStrip from '$lib/components/macros/macro-strip.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Item from '$lib/components/ui/item';

	// A list of rows rather than a table: a meal's identity is its label and what it
	// weighs in kcal, not five numeric columns to compare across.
	let { meals }: { meals: Meal[] } = $props();
</script>

<Item.Group class="gap-2">
	{#each meals as meal (meal.id)}
		<Item.Root variant="outline">
			<Item.Media variant="icon">
				<UtensilsIcon class="size-4" />
			</Item.Media>
			<Item.Content>
				<Item.Title>
					<a href="/meals/{meal.id}" class="hover:underline">{meal.label}</a>
				</Item.Title>
				<Item.Description>
					{meal.servings.length}
					{meal.servings.length === 1 ? 'serving' : 'servings'}
				</Item.Description>
			</Item.Content>
			<MacroStrip macros={meal.macros} />
			<Item.Actions>
				<Button href="/meals/{meal.id}/edit" variant="ghost" size="icon" title="Edit">
					<PencilIcon class="size-4" />
				</Button>
				<DeleteAction
					name={meal.label}
					size="icon"
					description="The meal is removed. Day plans that use it will show it as removed."
					action={() => deleteMeal({ path: { mealId: meal.id }, throwOnError: true })}
				/>
			</Item.Actions>
		</Item.Root>
	{/each}
</Item.Group>
