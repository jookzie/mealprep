<script lang="ts">
	import { deleteMeasurement, type MeasurementKind, runMutation, setMeasurement } from '$lib/api';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import NumberField from '$lib/components/NumberField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import TrendChart from '$lib/components/TrendChart.svelte';
	import {
		adiposityLabel,
		formatChange,
		formatMeasurement,
		KINDS,
		kindLabel,
		kindUnit,
		latestByKind,
		measurementChange,
		measurementChart,
		ofKind,
		recompositionNote,
		trendChange,
		waistToHeight,
	} from '$lib/domain/measurement';
	import { dayMonthLabel, fullDateLabel, todayIso } from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let kind = $state<MeasurementKind>('waist');
	let value = $state<number | null>(null);
	let date = $state(todayIso());
	let busy = $state(false);
	/** The kind being read; null follows whichever was last written or waist first. */
	let chosen = $state<MeasurementKind | null>(null);

	const recorded = $derived(latestByKind(data.measurements));
	const shown = $derived(chosen ?? recorded[0]?.kind ?? 'waist');
	const history = $derived(ofKind(data.measurements, shown).toReversed());
	const points = $derived(measurementChart(data.measurements, shown));
	const ratio = $derived(waistToHeight(data.measurements));
	const waistChange = $derived(measurementChange(data.measurements, 'waist'));
	const weightChange = $derived(
		waistChange ? trendChange(data.weight.points, waistChange.from, waistChange.to) : null,
	);
	const note = $derived(
		waistChange && weightChange !== null
			? recompositionNote(waistChange.change, weightChange)
			: null,
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (value === null) return;
		busy = true;
		const saved = await runMutation(() => setMeasurement(date, kind, value as number), {
			success: `${kindLabel(kind)} saved`,
		});
		busy = false;
		if (saved) {
			value = null;
			chosen = saved.kind;
		}
	}
</script>

<PageHeader
	title="Measurements"
	back="/body"
	subtitle="Same spot, same time of day; waist tells the most"
/>

<form class="card stack" onsubmit={submit}>
	<div class="grid-2">
		<label class="field">
			<span>Measurement</span>
			<select bind:value={kind}>
				{#each KINDS as option (option.kind)}
					<option value={option.kind}>{option.label}</option>
				{/each}
			</select>
		</label>
		{#key kind}
			<NumberField label="Value" suffix={kindUnit(kind)} bind:value required />
		{/key}
		<label class="field">
			<span>Date</span>
			<input type="date" max={todayIso()} bind:value={date} />
		</label>
	</div>
	<div class="row end">
		<button class="primary" type="submit" disabled={busy || value === null}>Save</button>
	</div>
</form>

{#if data.measurements.length === 0}
	<p class="empty">
		Nothing measured yet. A waist every week or two, and your height once, are enough for the
		most useful figure here.
	</p>
{:else}
	{#if ratio || waistChange}
		<section class="card stack">
			{#if ratio}
				<div class="row spread">
					<span>Waist-to-height</span>
					<strong class="figure">{ratio.ratio.toFixed(2)}</strong>
				</div>
				<p class="small muted">
					NICE reads {adiposityLabel(ratio.band)} as {ratio.band} central fat. The ratio needs no
					age or sex tables, and follows the fat that matters most for health better than BMI.
				</p>
			{/if}
			{#if waistChange}
				<div class="row spread">
					<span>Last 4 weeks</span>
					<span class="figure small">
						waist {formatChange(waistChange.change, 'cm')}
						{#if weightChange !== null}· weight trend {formatChange(weightChange, 'kg')}{/if}
					</span>
				</div>
				{#if note}
					<p class="small muted">{note}</p>
				{/if}
			{/if}
		</section>
	{/if}

	<section class="card stack">
		<div class="row kinds" role="group" aria-label="Measurement shown">
			{#each recorded as latest (latest.kind)}
				<button
					class:selected={shown === latest.kind}
					aria-pressed={shown === latest.kind}
					onclick={() => (chosen = latest.kind)}>{kindLabel(latest.kind)}</button
				>
			{/each}
		</div>
		{#if points.length > 1}
			<TrendChart
				{points}
				format={(figure) => formatMeasurement(shown, figure)}
				minSpan={kindUnit(shown) === '%' ? 2 : 3}
				label="{kindLabel(shown)} as measured, with a straight line between measurements. The history below lists the same figures."
				valueName="measured"
				lineName="between"
			/>
		{:else}
			<p class="small muted">A second measurement draws the line.</p>
		{/if}
	</section>

	<ul class="list">
		{#each history as entry (entry.date)}
			<li>
				<div class="card row spread history">
					<div class="grow truncate">
						<strong>{formatMeasurement(entry.kind, entry.value)}</strong>
						<span class="muted small"> · {dayMonthLabel(entry.date)} {entry.date.slice(0, 4)}</span>
					</div>
					<DeleteButton
						what="the {kindLabel(entry.kind).toLowerCase()} for {fullDateLabel(entry.date)}"
						consequence="The chart is redrawn without it."
						onConfirm={() =>
							runMutation(() => deleteMeasurement(entry.date, entry.kind), {
								success: 'Measurement deleted',
							})}
					/>
				</div>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.figure {
		font-variant-numeric: tabular-nums;
	}

	/* One line per entry: the date is short enough that the delete button never wraps. */
	.history {
		flex-wrap: nowrap;
	}

	.kinds {
		gap: 0.35rem;
	}

	.kinds button {
		min-height: 36px;
		padding: 0.25rem 0.7rem;
	}

	.kinds button.selected {
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
