<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Meal } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import { resolveMeals } from '$lib/domain/day-plan';
	import { formatKcal } from '$lib/domain/format';

	let {
		mealIds,
		meals,
		onRemove
	}: { mealIds: readonly string[]; meals: readonly Meal[]; onRemove: (index: number) => void } =
		$props();

	const resolved = $derived(resolveMeals(mealIds, meals));
</script>

<ul class="divide-border divide-y rounded-md border">
	{#each resolved as meal, index (`${mealIds[index]}-${index}`)}
		<li class="flex items-center gap-3 px-3 py-2">
			{#if meal}
				<span class="flex-1 truncate">{meal.label}</span>
				<span class="text-muted-foreground text-sm tabular-nums">
					{formatKcal(meal.macros.energyKcal)}
				</span>
			{:else}
				<!-- Meals are soft-deleted and never returned, so this join can miss. -->
				<span class="text-muted-foreground flex-1 italic">Meal removed</span>
			{/if}
			<Button
				type="button"
				variant="ghost"
				size="icon"
				onclick={() => onRemove(index)}
				aria-label="Remove meal"
			>
				<XIcon class="size-4" />
			</Button>
		</li>
	{/each}
</ul>
