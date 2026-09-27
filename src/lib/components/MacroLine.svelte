<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal } from '$lib/domain/format';

	let { macros }: { macros: Macros } = $props();
</script>

<span class="macro-line">
	<strong>{formatKcal(macros.energyKcal)}</strong>
	<span><i style:background="var(--protein)"></i>P {formatGrams(macros.proteinG)}</span>
	<span><i style:background="var(--fat)"></i>F {formatGrams(macros.fatG)}</span>
	<span><i style:background="var(--carbs)"></i>C {formatGrams(macros.carbohydratesG)}</span>
</span>

<style>
	.macro-line {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.2rem 0.7rem;
		font-size: 0.85rem;
		font-variant-numeric: tabular-nums;
	}

	i {
		display: inline-block;
		width: 0.55rem;
		height: 0.55rem;
		border-radius: 50%;
		margin-right: 0.25rem;
	}
</style>
