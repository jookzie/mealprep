<script lang="ts">
	import { untrack } from 'svelte';
	import { runMutation, setTargets } from '$lib/api';
	import NumberField from '$lib/components/NumberField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { formatKcal } from '$lib/domain/format';
	import { impliedEnergyKcal } from '$lib/domain/macros';
	import { formatKilograms, formatRate } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const seed = untrack(() => data.targets?.macros);
	let energyKcal = $state<number | null>(seed?.energyKcal ?? null);
	let proteinG = $state<number | null>(seed?.proteinG ?? null);
	let fatG = $state<number | null>(seed?.fatG ?? null);
	let carbohydratesG = $state<number | null>(seed?.carbohydratesG ?? null);
	let busy = $state(false);

	// Four independent numbers can quietly disagree; saying so informs and blocks nothing.
	const implied = $derived(
		proteinG === null || fatG === null || carbohydratesG === null
			? null
			: impliedEnergyKcal({ energyKcal: 0, proteinG, fatG, carbohydratesG }),
	);
	const disagrees = $derived(
		implied !== null && energyKcal !== null && Math.abs(implied - energyKcal) > energyKcal * 0.05,
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (energyKcal === null || proteinG === null || fatG === null || carbohydratesG === null) {
			return;
		}
		const macros = { energyKcal, proteinG, fatG, carbohydratesG };
		busy = true;
		await runMutation(() => setTargets(macros), { success: 'Targets saved' });
		busy = false;
	}
</script>

<PageHeader title="Daily targets" subtitle="What plans and the calendar are measured against" />

<form class="card stack" onsubmit={submit}>
	<div class="grid-2">
		<NumberField label="Energy" suffix="kcal" bind:value={energyKcal} required />
		<NumberField label="Protein" suffix="g" bind:value={proteinG} required />
		<NumberField label="Fat" suffix="g" bind:value={fatG} required />
		<NumberField label="Carbs" suffix="g" bind:value={carbohydratesG} required />
	</div>
	{#if disagrees && implied !== null && energyKcal !== null}
		<p class="small muted">
			Protein, fat and carbs add up to {formatKcal(implied)}, while the energy target is
			{formatKcal(energyKcal)}. Nothing is blocked by this.
		</p>
	{/if}
	<div class="row end">
		<button
			class="primary"
			type="submit"
			disabled={busy ||
				energyKcal === null ||
				proteinG === null ||
				fatG === null ||
				carbohydratesG === null}>Save</button
		>
	</div>
</form>

<!--
	Weight is the other thing plans are measured against, so it sits with the targets rather
	than taking a sixth tab: both platforms' guidelines cap a bottom bar at five.
-->
<a class="card list-link row spread" href="/weight">
	<div>
		<strong>Weight</strong>
		<span class="muted small">
			{#if data.weight.trend === undefined}
				· nothing logged yet
			{:else}
				· trend {formatKilograms(data.weight.trend)}
			{/if}
		</span>
	</div>
	<span class="small muted">
		{data.weight.ratePerWeek === undefined ? '' : formatRate(data.weight.ratePerWeek)} →
	</span>
</a>
