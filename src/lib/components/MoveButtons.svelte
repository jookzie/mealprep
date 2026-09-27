<script lang="ts">
	import { canMoveDown, canMoveUp } from '$lib/domain/order';

	let {
		index,
		length,
		name,
		onMove,
		onRemove,
	}: {
		index: number;
		length: number;
		name: string;
		onMove: (from: number, to: number) => void;
		onRemove: () => void;
	} = $props();
</script>

<!-- Buttons rather than drag: WCAG 2.2 SC 2.5.7 requires a single-pointer path. -->
<div class="row moves">
	<button
		type="button"
		class="ghost icon"
		aria-label="Move {name} up"
		disabled={!canMoveUp(index)}
		onclick={() => onMove(index, index - 1)}>↑</button
	>
	<button
		type="button"
		class="ghost icon"
		aria-label="Move {name} down"
		disabled={!canMoveDown(index, length)}
		onclick={() => onMove(index, index + 1)}>↓</button
	>
	<button type="button" class="ghost icon danger" aria-label="Remove {name}" onclick={onRemove}>✕</button>
</div>

<style>
	.moves {
		gap: 0;
		flex-wrap: nowrap;
	}
</style>
