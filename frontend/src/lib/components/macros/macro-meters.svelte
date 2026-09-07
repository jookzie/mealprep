<script lang="ts">
	import type { Macros } from '$lib/api';
	import { MACRO_KEYS, MACRO_LABELS } from '$lib/domain/macros';
	import { MACRO_HUE, MACRO_UNIT } from './hues';
	import MacroMeter from './macro-meter.svelte';

	/*
	 * The four meters, in the one order they are ever shown in. One component behind
	 * every target comparison in the app — the day plan, the week strip, the calendar —
	 * so no two screens can draw the same comparison differently.
	 */
	let {
		actual,
		target,
		mode = 'planned'
	}: { actual: Macros; target: Macros; mode?: 'planned' | 'remaining' } = $props();
</script>

<ul class="space-y-3">
	{#each MACRO_KEYS as key (key)}
		<li>
			<MacroMeter
				label={MACRO_LABELS[key]}
				hue={MACRO_HUE[key]}
				actual={actual[key]}
				target={target[key]}
				unit={MACRO_UNIT[key]}
				{mode}
			/>
		</li>
	{/each}
</ul>
