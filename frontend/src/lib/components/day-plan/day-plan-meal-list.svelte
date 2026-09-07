<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Meal } from '$lib/api';
	import MoveButtons from '$lib/components/app/move-buttons.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { formatKcal } from '$lib/domain/format';

	/*
	 * A day plan has no time-of-day slots, which is what keeps it reusable — "High day"
	 * should work whether you eat three times or five. That makes the order of the meals
	 * carry whatever meaning slots would have, so it is editable here. The schema has
	 * persisted `position` all along; only the UI never exposed it.
	 */
	let {
		rows,
		onRemove,
		onMove
	}: {
		rows: readonly { id: string; mealId: string; meal: Meal | undefined }[];
		onRemove: (id: string) => void;
		onMove: (from: number, to: number) => void;
	} = $props();
</script>

<ul class="divide-border divide-y rounded-md border">
	{#each rows as entry, index (entry.id)}
		<li class="flex items-center gap-2 px-2 py-2">
			<MoveButtons
				{index}
				length={rows.length}
				name={entry.meal?.label ?? 'meal'}
				onMove={(to) => onMove(index, to)}
			/>
			<Badge variant="secondary" class="tabular-nums">{index + 1}</Badge>
			{#if entry.meal}
				<a href="/meals/{entry.meal.id}" class="flex-1 truncate hover:underline">
					{entry.meal.label}
				</a>
				<span class="text-muted-foreground text-sm tabular-nums">
					{formatKcal(entry.meal.macros.energyKcal)}
				</span>
			{:else}
				<!-- Meals are soft-deleted and never returned, so this join can miss. -->
				<span class="text-muted-foreground flex-1 italic">Meal removed</span>
			{/if}
			<Button
				type="button"
				variant="ghost"
				size="icon"
				onclick={() => onRemove(entry.id)}
				aria-label="Remove {entry.meal?.label ?? 'meal'}"
			>
				<XIcon class="size-4" />
			</Button>
		</li>
	{/each}
</ul>
