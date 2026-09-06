<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import { ratio } from '$lib/domain/macros';

	// Planned against target, per macro. Going over is drawn, not flagged: the bar
	// fills and then keeps a distinct over segment, in the same tone (TG-3).
	let {
		actual,
		target,
		compact = false
	}: { actual: Macros; target: Macros; compact?: boolean } = $props();

	const bars = $derived([
		{
			label: 'Energy',
			text: `${formatKcal(actual.energyKcal)} of ${formatKcal(target.energyKcal)}`,
			fraction: ratio(actual.energyKcal, target.energyKcal)
		},
		{
			label: 'Protein',
			text: `${formatGrams(actual.proteinG)} of ${formatGrams(target.proteinG)}`,
			fraction: ratio(actual.proteinG, target.proteinG)
		},
		{
			label: 'Fat',
			text: `${formatGrams(actual.fatG)} of ${formatGrams(target.fatG)}`,
			fraction: ratio(actual.fatG, target.fatG)
		},
		{
			label: 'Carbs',
			text: `${formatGrams(actual.carbohydratesG)} of ${formatGrams(target.carbohydratesG)}`,
			fraction: ratio(actual.carbohydratesG, target.carbohydratesG)
		}
	]);
</script>

<ul class="space-y-2">
	{#each bars as bar (bar.label)}
		<li class="space-y-1">
			{#if !compact}
				<div class="flex items-baseline justify-between text-sm">
					<span class="text-muted-foreground">{bar.label}</span>
					<span class="tabular-nums">{bar.text}</span>
				</div>
			{/if}
			<!-- Decorative: every figure it draws is already on the page as text, so
			     it is hidden rather than duplicated for a screen reader. -->
			<div class="bg-muted flex h-2 w-full overflow-hidden rounded-full" aria-hidden="true">
				<div
					class="bg-primary h-full"
					style:width="{Math.min(bar.fraction, 1) * 100}%"
				></div>
				{#if bar.fraction > 1}
					<div
						class="bg-primary/40 h-full"
						style:width="{Math.min(bar.fraction - 1, 1) * 100}%"
					></div>
				{/if}
			</div>
			{#if compact}
				<p class="text-muted-foreground text-xs tabular-nums">{bar.label}: {bar.text}</p>
			{/if}
		</li>
	{/each}
</ul>
