<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal } from '$lib/domain/format';

	// The four macros, shown the same way everywhere they appear. The rest of a
	// product's nutrients live behind an expansion (PR-9), never here.
	let { macros, variant = 'inline' }: { macros: Macros; variant?: 'inline' | 'grid' } = $props();

	const figures = $derived([
		{ label: 'Energy', value: formatKcal(macros.energyKcal) },
		{ label: 'Protein', value: formatGrams(macros.proteinG) },
		{ label: 'Fat', value: formatGrams(macros.fatG) },
		{ label: 'Carbs', value: formatGrams(macros.carbohydratesG) }
	]);
</script>

{#if variant === 'grid'}
	<dl class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		{#each figures as figure (figure.label)}
			<div class="bg-muted/40 rounded-lg px-3 py-2">
				<dt class="text-muted-foreground text-xs">{figure.label}</dt>
				<dd class="mt-0.5 font-medium tabular-nums">{figure.value}</dd>
			</div>
		{/each}
	</dl>
{:else}
	<dl class="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
		{#each figures as figure (figure.label)}
			<div class="flex items-center gap-1.5">
				<dt class="text-xs">{figure.label}</dt>
				<dd class="text-foreground tabular-nums">{figure.value}</dd>
			</div>
		{/each}
	</dl>
{/if}
