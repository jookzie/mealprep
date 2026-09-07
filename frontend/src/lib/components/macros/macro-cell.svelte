<script lang="ts">
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import { tint } from '$lib/domain/heatmap';
	import type { MacroKey } from '$lib/domain/macros';
	import { MACRO_HUE } from './hues';

	/*
	 * One macro cell in an entity table, shaded against the rest of its column.
	 *
	 * The tint is the macro's own hue, so a strong cell reads as "a lot of protein"
	 * rather than as a verdict — the same rule the meters follow, and the reason no
	 * column ever shades toward red. The figure is printed regardless, so the colour
	 * is a second channel and never the only one.
	 */
	let { value, macro, intensity }: { value: number; macro: MacroKey; intensity: number } = $props();
</script>

<Table.Cell
	class="text-right tabular-nums"
	style="background-color: color-mix(in oklch, {MACRO_HUE[macro]} {tint(intensity) * 100}%, transparent)"
>
	{macro === 'energyKcal' ? formatKcal(value) : formatGrams(value)}
</Table.Cell>
