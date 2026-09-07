<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronUpIcon from '@lucide/svelte/icons/chevron-up';
	import { Button } from '$lib/components/ui/button';
	import { canMoveDown, canMoveUp } from '$lib/domain/order';

	/*
	 * Reordering by button rather than by drag. WCAG 2.2 SC 2.5.7 requires a single
	 * pointer alternative to any dragging motion, so this is the required baseline
	 * rather than the cheap option — drag could only ever be added on top of it.
	 */
	let {
		index,
		length,
		name,
		onMove
	}: { index: number; length: number; name: string; onMove: (to: number) => void } = $props();
</script>

<div class="flex flex-col">
	<Button
		type="button"
		variant="ghost"
		size="icon"
		class="size-5"
		disabled={!canMoveUp(index)}
		aria-label="Move {name} up"
		onclick={() => onMove(index - 1)}
	>
		<ChevronUpIcon class="size-3.5" />
	</Button>
	<Button
		type="button"
		variant="ghost"
		size="icon"
		class="size-5"
		disabled={!canMoveDown(index, length)}
		aria-label="Move {name} down"
		onclick={() => onMove(index + 1)}
	>
		<ChevronDownIcon class="size-3.5" />
	</Button>
</div>
