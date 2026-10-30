<script lang="ts">
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import X from '@lucide/svelte/icons/x';
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
<div class="moves">
	<div class="arrows">
		<button
			type="button"
			aria-label="Move {name} up"
			disabled={!canMoveUp(index)}
			onclick={() => onMove(index, index - 1)}
		>
			<ChevronUp size={14} strokeWidth={3} aria-hidden="true" />
		</button>
		<button
			type="button"
			aria-label="Move {name} down"
			disabled={!canMoveDown(index, length)}
			onclick={() => onMove(index, index + 1)}
		>
			<ChevronDown size={14} strokeWidth={3} aria-hidden="true" />
		</button>
	</div>
	<button type="button" class="remove" aria-label="Remove {name}" onclick={onRemove}>
		<X size={18} strokeWidth={2.6} aria-hidden="true" />
	</button>
</div>

<style>
	.moves {
		display: flex;
		align-items: center;
		gap: 2px;
		flex-shrink: 0;
	}

	.arrows {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.arrows button {
		width: 34px;
		min-height: 0;
		height: 24px;
		padding: 0;
		border-radius: 8px;
		background: var(--raised);
	}

	.remove {
		width: 36px;
		min-width: 0;
		padding: 0;
		background: transparent;
		color: var(--muted);
	}
</style>
