<script lang="ts">
	import { deleteWeightEntry, runMutation, setWeightEntry } from '$lib/api';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import WeightChart from '$lib/components/WeightChart.svelte';
	import { formatDecimal, parseDecimal } from '$lib/domain/number';
	import { fullDateLabel, isToday, todayIso } from '$lib/domain/week';
	import { formatKilograms, formatRate, RANGES, type Range, rangeLabel, visible } from '$lib/domain/weight';
	import { keep } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const kilograms = keep<number | null>('weight:kilograms', null);
	const date = keep('weight:date', todayIso());
	const range = keep<Range>('weight:range', 90);
	let busy = $state(false);
	let text = $state(formatDecimal(kilograms.value));

	// The date is kept only alongside a weight not yet saved. An app left open overnight
	// would otherwise offer yesterday, and file the morning's weigh-in under it.
	if (kilograms.value === null) date.value = todayIso();

	const points = $derived(visible(data.series.points, range.value));
	const history = $derived(
		[...data.series.points].reverse().filter((point) => point.kilograms !== undefined),
	);
	const onDate = $derived(data.series.points.find((point) => point.date === date.value));

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const weight = kilograms.value;
		if (weight === null) return;
		busy = true;
		const saved = await runMutation(() => setWeightEntry(date.value, weight), {
			success: 'Weight saved',
		});
		busy = false;
		if (saved) {
			kilograms.value = null;
			text = '';
		}
	}
</script>

<PageHeader title="Weight" back="/you" backLabel="You" eyebrow="Weigh in daily; read the trend, not the scale" />

{#if data.series.points.length > 0}
	<div class="headline">
		<div>
			<span class="small strong">Trend</span>
			<span class="display trend figure">
				{data.series.trend === undefined ? '—' : formatKilograms(data.series.trend)}
			</span>
		</div>
		{#if data.series.ratePerWeek === undefined}
			<span class="muted small">A week of weigh-ins gives a rate</span>
		{:else}
			<span class="rate">{formatRate(data.series.ratePerWeek)}</span>
		{/if}
	</div>

	<div class="segmented" role="radiogroup" aria-label="Range">
		{#each RANGES as option (option)}
			<button
				type="button"
				role="radio"
				aria-checked={range.value === option}
				onclick={() => (range.value = option)}>{rangeLabel(option)}</button
			>
		{/each}
	</div>

	<WeightChart {points} />
{/if}

<form class="log" onsubmit={submit}>
	<div class="grow fields">
		<label class="date-field">
			<span class="small strong">{isToday(date.value) ? 'Today' : fullDateLabel(date.value)}</span>
			<input
				type="date"
				class="date"
				aria-label="Date"
				max={todayIso()}
				bind:value={date.value}
			/>
		</label>
		<label class="kg">
			<input
				type="text"
				inputmode="decimal"
				autocomplete="off"
				placeholder="80.0"
				aria-label="Weight in kilograms"
				value={text}
				oninput={(event) => {
					text = event.currentTarget.value;
					kilograms.value = parseDecimal(text);
				}}
			/>
			<span class="strong">kg</span>
		</label>
	</div>
	<button class="primary" type="submit" disabled={busy || kilograms.value === null}>Log</button>
</form>
{#if onDate?.kilograms !== undefined}
	<p class="small muted">
		{isToday(date.value) ? 'Today' : fullDateLabel(date.value)} already reads
		{formatKilograms(onDate.kilograms)}. Logging replaces it.
	</p>
{/if}

{#if data.series.points.length === 0}
	<p class="empty">No weigh-ins yet. The trend needs a week or so before it says anything.</p>
{:else}
	<h2>History</h2>
	<ul class="history">
		{#each history as point (point.date)}
			<li>
				<span class="grow">
					<strong class="figure">{formatKilograms(point.kilograms ?? 0)}</strong>
					<span class="muted small">{fullDateLabel(point.date)}</span>
				</span>
				<span class="muted small figure">trend {formatKilograms(point.trend)}</span>
				<DeleteButton
					what="the weigh-in for {fullDateLabel(point.date)}"
					consequence="The trend is recalculated without it."
					onConfirm={() =>
						runMutation(() => deleteWeightEntry(point.date), { success: 'Weigh-in deleted' })}
				/>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.strong {
		font-weight: 700;
	}

	.headline {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 0.75rem;
	}

	.headline > div {
		display: flex;
		flex-direction: column;
	}

	.trend {
		font-size: 3.25rem;
		line-height: 1;
	}

	.rate {
		font-weight: 700;
		padding: 0.35rem 0.75rem;
		border-radius: 14px;
		background: var(--carbs);
		color: var(--on-macro);
		white-space: nowrap;
	}

	.log {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 0.75rem 0.75rem 1.1rem;
		border-radius: var(--radius-lg);
		background: var(--surface);
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.date-field {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	/* The native date picker opens from a small chip; the figure is what the row is for. */
	.date {
		width: auto;
		min-height: 32px;
		padding: 0.1rem 0.5rem;
		border-radius: 10px;
		font-size: 0.8rem;
		border-width: 1.5px;
	}

	.kg {
		display: flex;
		align-items: baseline;
		gap: 0.3rem;
	}

	.kg input {
		width: 6.5rem;
		min-height: 0;
		padding: 0;
		border: none;
		background: transparent;
		font-family: var(--font-display);
		font-size: 2rem;
		font-weight: 800;
	}

	.log .primary {
		min-height: 52px;
		padding: 0 1.4rem;
	}

	.history {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.history li {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.4rem 0;
		border-bottom: 1.5px solid var(--border);
	}

	.history .grow {
		display: flex;
		flex-direction: column;
	}
</style>
