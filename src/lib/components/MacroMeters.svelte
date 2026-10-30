<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Macros } from '$lib/api';
	import { formatSigned } from '$lib/domain/format';
	import { MACRO_LABELS, MACRO_SERIES, ratio } from '$lib/domain/macros';

	let {
		actual,
		target,
		caption = 'kcal',
		aside,
	}: {
		actual: Macros;
		/** Null where there is nothing to measure against, such as one product or one meal. */
		target: Macros | null;
		/** Under the energy figure: what it is ("kcal planned", "kcal per 100 g"). */
		caption?: string;
		/** The right of the energy card, for a figure that belongs beside it — cost, say. */
		aside?: Snippet;
	} = $props();

	const HUES = { proteinG: 'var(--protein)', fatG: 'var(--fat)', carbohydratesG: 'var(--carbs)' };

	const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
	const oneDecimal = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

	// The bar's full width is the target, so an over-target day fills it and the excess is
	// said in words and drawn as a hatch over the full bar: a material, never a warning colour.
	function fill(value: number, goal: number): string {
		return `${Math.min(1, Math.max(0, ratio(value, goal))) * 100}%`;
	}
</script>

<!--
	Planned against target is a meter per macro, never a ring: a ring encodes 0–100% and has
	no honest way to show 130%, which TG-3 requires. Energy is the hero card, the sum of the
	three pastel blocks beneath it rather than a fourth sibling.
-->
<div class="meters">
	<div class="energy">
		<div class="figure-block">
			<span class="display big figure">{whole.format(actual.energyKcal)}</span>
			<span class="caption">{caption}</span>
		</div>
		<div class="side">
			{#if target}
				{@const delta = actual.energyKcal - target.energyKcal}
				<span class="delta figure">{formatSigned(delta, 'kcal').replace(' kcal', '')}</span>
				<span class="caption small">target {whole.format(target.energyKcal)}</span>
			{/if}
			{@render aside?.()}
		</div>
	</div>

	<div class="blocks">
		{#each MACRO_SERIES as key (key)}
			{@const value = actual[key]}
			{@const goal = target?.[key] ?? null}
			<div class="block" style:background={HUES[key]}>
				<span class="label">{MACRO_LABELS[key]}</span>
				<span class="display value figure">{oneDecimal.format(value)}<small>g</small></span>
				{#if goal !== null}
					{@const share = ratio(value, goal)}
					<div
						class="track"
						role="meter"
						aria-label={MACRO_LABELS[key]}
						aria-valuemin={0}
						aria-valuemax={goal}
						aria-valuenow={value}
						aria-valuetext="{oneDecimal.format(value)} g of {oneDecimal.format(goal)} g"
					>
						<span class="bar" class:over={share > 1} style:width={fill(value, goal)}></span>
					</div>
					<span class="of figure">of {oneDecimal.format(goal)} g · {whole.format(share * 100)}%</span>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.meters {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.energy {
		background: var(--hero);
		color: var(--hero-text);
		border-radius: var(--radius-lg);
		padding: 1.1rem 1.25rem;
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: 0.75rem;
	}

	.figure-block,
	.side {
		display: flex;
		flex-direction: column;
	}

	.side {
		align-items: flex-end;
		gap: 0.35rem;
	}

	.big {
		font-size: 3.25rem;
		line-height: 0.95;
		letter-spacing: -0.04em;
	}

	.caption {
		color: var(--hero-muted);
		font-weight: 600;
	}

	.delta {
		font-weight: 800;
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: var(--bg);
		color: var(--text);
	}

	.blocks {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 10px;
	}

	.block {
		color: var(--on-macro);
		border-radius: var(--radius);
		padding: 0.85rem 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
		min-width: 0;
	}

	.label {
		font-size: 0.8rem;
		font-weight: 700;
	}

	.value {
		font-size: 1.75rem;
		line-height: 1;
		letter-spacing: -0.03em;
		overflow-wrap: anywhere;
	}

	.value small {
		font-size: 0.95rem;
	}

	.track {
		height: 10px;
		border-radius: 5px;
		background: rgb(17 17 17 / 0.14);
		overflow: hidden;
	}

	.bar {
		display: block;
		height: 100%;
		border-radius: 5px;
		background: #111111;
	}

	/* Over target is the same bar under a hatch: a material, never a warning colour. */
	.bar.over {
		background-image: repeating-linear-gradient(
			45deg,
			rgb(255 255 255 / 0.45) 0 3px,
			transparent 3px 6px
		);
	}

	.of {
		font-size: 0.75rem;
		font-weight: 600;
	}
</style>
