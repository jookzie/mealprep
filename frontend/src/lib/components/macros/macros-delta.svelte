<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatSigned } from '$lib/domain/format';
	import { deviation } from '$lib/domain/macros';

	// Deviations are shown and nothing else: the app never blocks or alters a plan
	// because of them (TG-3), so these carry no warning colour.
	let { actual, target }: { actual: Macros; target: Macros } = $props();

	const delta = $derived(deviation(actual, target));
	const chips = $derived([
		{ label: 'Energy', value: formatSigned(delta.energyKcal, 'kcal') },
		{ label: 'Protein', value: formatSigned(delta.proteinG, 'g') },
		{ label: 'Fat', value: formatSigned(delta.fatG, 'g') },
		{ label: 'Carbs', value: formatSigned(delta.carbohydratesG, 'g') }
	]);
</script>

<ul class="flex flex-wrap gap-2">
	{#each chips as chip (chip.label)}
		<li class="bg-muted text-muted-foreground rounded-md px-2 py-1 text-xs">
			<span>{chip.label}</span>
			<span class="text-foreground ml-1 tabular-nums">{chip.value}</span>
		</li>
	{/each}
</ul>
