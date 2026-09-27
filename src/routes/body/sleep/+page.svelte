<script lang="ts">
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SleepChart from '$lib/components/SleepChart.svelte';
	import { formatClock, formatDuration, formatSpread, sleepDays } from '$lib/domain/health';
	import { RANGES, type Range, rangeLabel, visible } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let range = $state<Range>(30);

	const sleep = $derived(data.sleep);
	const days = $derived(visible(sleepDays(sleep.nights), range));
</script>

<PageHeader
	title="Sleep"
	back="/body"
	subtitle="How long, and how regularly: both matter on their own"
/>

{#if sleep.nights.length === 0}
	<p class="empty">No nights yet. Sleep with the strap on and sync.</p>
{:else}
	<section class="card stack">
		<div class="grid-2 figures">
			<div>
				<span class="small muted">Asleep, last 7 nights</span>
				<strong>{sleep.averageMinutes === undefined ? '—' : formatDuration(sleep.averageMinutes)}</strong>
			</div>
			<div>
				<span class="small muted">Typical night</span>
				<strong>
					{sleep.typicalBedMinute === undefined || sleep.typicalWakeMinute === undefined
						? '—'
						: `${formatClock(sleep.typicalBedMinute)}–${formatClock(sleep.typicalWakeMinute)}`}
				</strong>
			</div>
		</div>
		<p class="small muted">
			{#if sleep.bedtimeSpreadMinutes !== undefined && sleep.wakeSpreadMinutes !== undefined}
				Over four weeks your bedtime varies by {formatSpread(sleep.bedtimeSpreadMinutes)} and your
				wake time by {formatSpread(sleep.wakeSpreadMinutes)}. Level bar ends below mean a steadier
				rhythm; irregular timing goes with poorer recovery even at the same total.
			{:else}
				Regularity needs two weeks of nights before it says anything.
			{/if}
		</p>

		<SleepChart {days} />

		<div class="row ranges" role="group" aria-label="Range">
			{#each RANGES as option (option)}
				<button
					class:selected={range === option}
					aria-pressed={range === option}
					onclick={() => (range = option)}>{rangeLabel(option)}</button
				>
			{/each}
		</div>
	</section>
{/if}

<style>
	.figures > div {
		display: flex;
		flex-direction: column;
	}

	.figures strong {
		font-variant-numeric: tabular-nums;
	}

	.ranges button {
		flex: 1;
	}

	.ranges button.selected {
		border-color: var(--accent);
		color: var(--accent);
	}
</style>
