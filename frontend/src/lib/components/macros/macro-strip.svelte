<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatKcal } from '$lib/domain/format';
	import { MACRO_SERIES } from '$lib/domain/macros';
	import { MACRO_HUE } from './hues';

	/*
	 * The compact form, for a calendar cell. Energy is the figure the eye goes to, so it
	 * is set large and the three macros follow as one short line. Four meters and four
	 * text lines per day would be fifty-six rows of chrome across a week.
	 *
	 * The letters are the direct labels: colour is never the only channel carrying which
	 * macro a figure is.
	 */
	let { macros }: { macros: Macros } = $props();

	const letters: Record<(typeof MACRO_SERIES)[number], string> = {
		proteinG: 'P',
		fatG: 'F',
		carbohydratesG: 'C'
	};

	const round = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
</script>

<div class="space-y-0.5">
	<p class="text-base font-medium tabular-nums">{formatKcal(macros.energyKcal)}</p>
	<dl class="text-muted-foreground flex items-center gap-2.5 text-xs">
		{#each MACRO_SERIES as key (key)}
			<div class="flex items-center gap-1">
				<dt style:color={MACRO_HUE[key]} class="font-medium">{letters[key]}</dt>
				<dd class="tabular-nums">{round.format(macros[key])}</dd>
			</div>
		{/each}
	</dl>
</div>
