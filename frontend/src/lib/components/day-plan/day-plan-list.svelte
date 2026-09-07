<script lang="ts">
	import LayersIcon from '@lucide/svelte/icons/layers';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { type DayPlan, deleteDayPlan } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import MacroStrip from '$lib/components/macros/macro-strip.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Item from '$lib/components/ui/item';

	// The plan's meals are listed on the row, in order: a plan's label alone says it
	// exists but not what it is, and the order is the only sequence a plan carries.
	let { dayPlans }: { dayPlans: DayPlan[] } = $props();
</script>

<Item.Group class="gap-2">
	{#each dayPlans as plan (plan.id)}
		<Item.Root variant="outline">
			<Item.Media variant="icon">
				<LayersIcon class="size-4" />
			</Item.Media>
			<Item.Content>
				<Item.Title>
					<a href="/day-plans/{plan.id}" class="hover:underline">{plan.label}</a>
				</Item.Title>
				<Item.Description>
					{#if plan.meals.length === 0}
						No meals
					{:else}
						{plan.meals.map((meal) => meal.label).join(' · ')}
					{/if}
				</Item.Description>
			</Item.Content>
			<MacroStrip macros={plan.macros} />
			<Item.Actions>
				<Button href="/day-plans/{plan.id}/edit" variant="ghost" size="icon" title="Edit">
					<PencilIcon class="size-4" />
				</Button>
				<DeleteAction
					name={plan.label}
					size="icon"
					description="The plan is removed. Calendar days holding it will show it as removed."
					action={() => deleteDayPlan({ path: { dayPlanId: plan.id }, throwOnError: true })}
				/>
			</Item.Actions>
		</Item.Root>
	{/each}
</Item.Group>
