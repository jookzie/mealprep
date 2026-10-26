<script lang="ts">
	import { deleteWeightEntry, runMutation, setWeightEntry } from '$lib/api';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import NumberField from '$lib/components/NumberField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import WeightChart from '$lib/components/WeightChart.svelte';
	import { fullDateLabel, todayIso } from '$lib/domain/week';
	import { formatKilograms, formatRate, RANGES, type Range, rangeLabel, visible } from '$lib/domain/weight';
	import { keep } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const kilograms = keep<number | null>('weight:kilograms', null);
	const date = keep('weight:date', todayIso());
	const range = keep<Range>('weight:range', 90);
	let busy = $state(false);

	// The date is kept only alongside a weight not yet saved. An app left open overnight
	// would otherwise offer yesterday, and file the morning's weigh-in under it.
	if (kilograms.value === null) date.value = todayIso();

	const points = $derived(visible(data.series.points, range.value));
	const history = $derived(
		[...data.series.points].reverse().filter((point) => point.kilograms !== undefined),
	);
	const today = $derived(data.series.points.find((point) => point.date === todayIso()));

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const weight = kilograms.value;
		if (weight === null) return;
		busy = true;
		const saved = await runMutation(() => setWeightEntry(date.value, weight), {
			success: 'Weight saved',
		});
		busy = false;
		if (saved) kilograms.value = null;
	}
</script>

<PageHeader title="Weight" subtitle="Weigh in daily; read the trend, not the scale" />

<form class="card stack" onsubmit={submit}>
	<div class="grid-2">
		<NumberField label="Weight" suffix="kg" bind:value={kilograms.value} required />
		<label class="field">
			<span>Date</span>
			<input type="date" max={todayIso()} bind:value={date.value} />
		</label>
	</div>
	{#if today?.kilograms !== undefined && date.value === todayIso()}
		<p class="small muted">
			Today already reads {formatKilograms(today.kilograms)}. Saving replaces it.
		</p>
	{/if}
	<div class="row end">
		<button class="primary" type="submit" disabled={busy || kilograms.value === null}>Save</button>
	</div>
</form>

{#if data.series.points.length === 0}
	<p class="empty">No weigh-ins yet. The trend needs a week or so before it says anything.</p>
{:else}
	<section class="card stack">
		<div class="row spread">
			<div>
				<strong>{data.series.trend === undefined ? '—' : formatKilograms(data.series.trend)}</strong>
				<span class="muted small"> trend</span>
			</div>
			<div class="rate small">
				{#if data.series.ratePerWeek === undefined}
					<span class="muted">a week of weigh-ins gives a rate</span>
				{:else}
					{formatRate(data.series.ratePerWeek)}
				{/if}
			</div>
		</div>

		<WeightChart {points} />

		<div class="row ranges" role="group" aria-label="Range">
			{#each RANGES as option (option)}
				<button
					class:selected={range.value === option}
					aria-pressed={range.value === option}
					onclick={() => (range.value = option)}>{rangeLabel(option)}</button
				>
			{/each}
		</div>
	</section>

	<h2 class="small muted history-heading">History</h2>
	<ul class="list">
		{#each history as point (point.date)}
			<li>
				<div class="card row spread">
					<div>
						<strong>{formatKilograms(point.kilograms ?? 0)}</strong>
						<span class="muted small"> · {fullDateLabel(point.date)}</span>
					</div>
					<div class="row">
						<span class="muted small">trend {formatKilograms(point.trend)}</span>
						<DeleteButton
							what="the weigh-in for {fullDateLabel(point.date)}"
							consequence="The trend is recalculated without it."
							onConfirm={() =>
								runMutation(() => deleteWeightEntry(point.date), { success: 'Weigh-in deleted' })}
						/>
					</div>
				</div>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.rate {
		font-variant-numeric: tabular-nums;
	}

	.ranges button {
		flex: 1;
	}

	.ranges button.selected {
		border-color: var(--accent);
		color: var(--accent);
	}

	.history-heading {
		margin: 1rem 0 0.5rem;
		font-weight: 600;
	}
</style>
