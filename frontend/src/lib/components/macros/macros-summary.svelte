<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import { MACRO_KEYS, MACRO_LABELS, type MacroKey } from '$lib/domain/macros';
	import { MACRO_HUE } from './hues';

	// The four macros, shown the same way everywhere they appear. The rest of a
	// product's nutrients live behind an expansion (PR-9), never here.
	// `pair` is the two-column form for a narrow column, where the viewport-based
	// breakpoint in `grid` would widen to four and squeeze every figure onto two lines.
	let {
		macros,
		variant = 'inline'
	}: { macros: Macros; variant?: 'inline' | 'grid' | 'pair' } = $props();

	function value(key: MacroKey): string {
		return key === 'energyKcal' ? formatKcal(macros[key]) : formatGrams(macros[key]);
	}
</script>

{#if variant === 'grid' || variant === 'pair'}
	<dl class="grid gap-2 {variant === 'pair' ? 'grid-cols-2' : 'grid-cols-2 sm:grid-cols-4'}">
		{#each MACRO_KEYS as key (key)}
			<div class="bg-muted/40 rounded-lg px-3 py-2">
				<dt class="text-muted-foreground flex items-center gap-1.5 text-xs">
					<span
						class="size-1.5 rounded-full"
						style:background-color={MACRO_HUE[key]}
						aria-hidden="true"
					></span>
					{MACRO_LABELS[key]}
				</dt>
				<dd class="mt-0.5 font-medium tabular-nums">{value(key)}</dd>
			</div>
		{/each}
	</dl>
{:else}
	<dl class="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
		{#each MACRO_KEYS as key (key)}
			<div class="flex items-center gap-1.5">
				<span
					class="size-1.5 rounded-full"
					style:background-color={MACRO_HUE[key]}
					aria-hidden="true"
				></span>
				<dt class="text-xs">{MACRO_LABELS[key]}</dt>
				<dd class="text-foreground tabular-nums">{value(key)}</dd>
			</div>
		{/each}
	</dl>
{/if}
