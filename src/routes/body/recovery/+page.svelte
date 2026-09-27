<script lang="ts">
	import type { MarkerSeries } from '$lib/api';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import TrendChart from '$lib/components/TrendChart.svelte';
	import {
		behaviourLabel,
		formatMarker,
		impactHelps,
		impactPhrase,
		markerChart,
		markerName,
		rankImpacts,
		statusMeaning,
		statusPhrase,
	} from '$lib/domain/health';
	import { RANGES, type Range, rangeLabel, visible } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let range = $state<Range>(90);

	const series = $derived(
		[data.recovery.hrv, data.recovery.restingHeartRate].filter(
			(entry): entry is MarkerSeries => entry !== undefined,
		),
	);
	const impacts = $derived(rankImpacts(data.recovery.impacts));
	const impactMarker = $derived(data.recovery.impactMarker);
</script>

<PageHeader
	title="Recovery"
	back="/body"
	subtitle="Your 7-day average against your own normal range, never anyone else's"
/>

{#if series.length === 0}
	<p class="empty">
		No HRV or resting heart rate yet. Wear the strap overnight and sync; a week gives a first
		average and two give a normal range.
	</p>
{:else}
	{#each series as entry (entry.marker)}
		{@const format = (value: number) => formatMarker(entry.marker, value)}
		<section class="card stack">
			<div>
				<div class="row spread">
					<strong>{markerName(entry.marker)}</strong>
					<span class="figure">{entry.average === undefined ? '—' : format(entry.average)}</span>
				</div>
				<p class="small muted">
					{#if entry.status}
						7-day average {statusPhrase(entry.status)}
						{#if entry.band}({Math.round(entry.band.low)}–{format(entry.band.high)}).{/if}
						{statusMeaning(entry.marker, entry.status)}
					{:else}
						Your normal range appears after two weeks of readings.
					{/if}
				</p>
			</div>

			<TrendChart
				points={visible(markerChart(entry), range)}
				{format}
				minSpan={entry.marker === 'hrv' ? 10 : 4}
				label="{markerName(entry.marker)} each morning as points, the 7-day average as a line, and your normal range as a shaded band."
				valueName="reading"
				lineName="7-day avg"
				bandName="normal"
			/>
		</section>
	{/each}

	<div class="row ranges" role="group" aria-label="Range">
		{#each RANGES as option (option)}
			<button
				class:selected={range === option}
				aria-pressed={range === option}
				onclick={() => (range = option)}>{rangeLabel(option)}</button
			>
		{/each}
	</div>

	<section class="stack">
		<h2 class="small muted heading">What goes with better mornings</h2>
		{#if impactMarker && impacts.length > 0}
			<ul class="list">
				{#each impacts as impact (impact.behaviour)}
					<li class="card impact">
						<div class="verdict">
							<span>{behaviourLabel(impact.behaviour)}</span>
							<strong class:quiet={!impact.clear}>
								{impact.clear ? (impactHelps(impact, impactMarker) ? 'better' : 'worse') : '—'}
							</strong>
						</div>
						<p class="small muted">
							{impactPhrase(impact, impactMarker)} · {impact.withDays} mornings with,
							{impact.withoutDays} without
						</p>
					</li>
				{/each}
			</ul>
			<p class="small muted">
				Compared over the last 90 days. These are associations in your own data, not causes.
			</p>
		{:else}
			<p class="small muted">
				Each habit needs five mornings with it and five without in the last 90 days before it
				is compared: sleep length, bedtime, workouts, and what the plan said you ate.
			</p>
		{/if}
	</section>
{/if}

<style>
	.figure {
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	.ranges button {
		flex: 1;
	}

	.ranges button.selected {
		border-color: var(--accent);
		color: var(--accent);
	}

	.heading {
		font-weight: 600;
	}

	/* The verdict keeps its column however long the behaviour's name wraps. */
	.verdict {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 0.75rem;
		align-items: baseline;
	}

	.impact p {
		margin-top: 0.15rem;
	}

	.quiet {
		color: var(--muted);
		font-weight: 400;
	}
</style>
