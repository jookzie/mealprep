<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal, formatSigned } from '$lib/domain/format';
	import { MACRO_KEYS, MACRO_LABELS, type MacroKey, ratio } from '$lib/domain/macros';

	let { actual, target }: { actual: Macros; target: Macros | null } = $props();

	/** The track runs to 150% of the target, so the target mark sits at two thirds on every row. */
	const TRACK = 1.5;

	const HUES: Record<MacroKey, string> = {
		energyKcal: 'var(--energy)',
		proteinG: 'var(--protein)',
		fatG: 'var(--fat)',
		carbohydratesG: 'var(--carbs)',
	};

	function figure(key: MacroKey, value: number): string {
		return key === 'energyKcal' ? formatKcal(value) : formatGrams(value);
	}

	function percent(fraction: number): string {
		return `${(Math.min(Math.max(fraction, 0), TRACK) / TRACK) * 100}%`;
	}
</script>

<div class="meters">
	{#each MACRO_KEYS as key (key)}
		{@const value = actual[key]}
		{@const goal = target?.[key] ?? null}
		{@const fraction = goal === null ? 0 : ratio(value, goal)}
		<div class="meter-row">
			<div class="row spread small">
				<span>{MACRO_LABELS[key]}</span>
				<span class="figures">
					{figure(key, value)}
					{#if goal !== null}
						<span class="muted">/ {figure(key, goal)}</span>
						<span class="muted">({formatSigned(value - goal, key === 'energyKcal' ? 'kcal' : 'g')})</span>
					{/if}
				</span>
			</div>
			{#if goal !== null}
				<div
					class="track"
					role="meter"
					aria-label={MACRO_LABELS[key]}
					aria-valuemin={0}
					aria-valuemax={goal * TRACK}
					aria-valuenow={value}
					aria-valuetext="{figure(key, value)} of {figure(key, goal)}"
				>
					<div class="fill" style:width={percent(Math.min(fraction, 1))} style:background-color={HUES[key]}></div>
					{#if fraction > 1}
						<div
							class="fill over"
							style:left={percent(1)}
							style:width="calc({percent(fraction)} - {percent(1)})"
							style:background-color={HUES[key]}
						></div>
					{/if}
					<div class="mark" style:left={percent(1)}></div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.meters {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	.figures {
		font-variant-numeric: tabular-nums;
	}

	.track {
		position: relative;
		height: 0.6rem;
		margin-top: 0.2rem;
		border-radius: 999px;
		background: var(--surface-2);
		overflow: hidden;
	}

	.fill {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 0;
	}

	/* Over target is the same hue under a hatch: a material, never a warning colour. */
	.over {
		background-image: repeating-linear-gradient(
			45deg,
			rgb(255 255 255 / 0.55) 0 3px,
			transparent 3px 6px
		);
	}

	.mark {
		position: absolute;
		top: -2px;
		bottom: -2px;
		width: 2px;
		background: var(--text);
	}
</style>
