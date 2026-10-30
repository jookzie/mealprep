<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatKcal } from '$lib/domain/format';

	let { macros, energy = true }: { macros: Macros; energy?: boolean } = $props();

	const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
</script>

<!-- One line of figures: energy in ink, then a tag per macro in its own pastel. -->
<span class="macro-line">
	{#if energy}<strong>{formatKcal(macros.energyKcal)}</strong>{/if}
	<span class="tag protein" title="Protein">P {whole.format(macros.proteinG)}</span>
	<span class="tag fat" title="Fat">F {whole.format(macros.fatG)}</span>
	<span class="tag carbs" title="Carbohydrates">C {whole.format(macros.carbohydratesG)}</span>
</span>

<style>
	.macro-line {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.3rem 0.4rem;
		font-size: 0.8rem;
		font-variant-numeric: tabular-nums;
	}

	strong {
		margin-right: 0.2rem;
		font-size: 0.875rem;
	}

	.tag {
		font-weight: 700;
		padding: 0.1rem 0.5rem;
		border-radius: 8px;
		color: var(--on-macro);
	}

	.protein {
		background: var(--protein);
	}

	.fat {
		background: var(--fat);
	}

	.carbs {
		background: var(--carbs);
	}
</style>
