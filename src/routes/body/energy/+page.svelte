<script lang="ts">
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { formatKcal, formatSigned } from '$lib/domain/format';
	import { dayMonthLabel } from '$lib/domain/week';
	import { formatKilograms } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const balance = $derived(data.balance);
	const difference = $derived(
		balance.expenditureKcal === undefined || balance.measuredKcal === undefined
			? undefined
			: balance.measuredKcal - balance.expenditureKcal,
	);
</script>

<PageHeader
	title="Energy balance"
	back="/body"
	subtitle="What your weight trend says you burn, from {dayMonthLabel(balance.from)} to {dayMonthLabel(balance.to)}"
/>

<section class="card stack">
	<dl class="figures">
		<div>
			<dt>Planned intake</dt>
			<dd>{balance.plannedKcal === undefined ? '—' : `${formatKcal(balance.plannedKcal)} a day`}</dd>
			<dd class="small muted">{balance.plannedDays} of {balance.windowDays} days planned</dd>
		</div>
		<div>
			<dt>Weight trend</dt>
			<dd>
				{balance.trendChangeKg === undefined
					? '—'
					: `${balance.trendChangeKg > 0 ? '+' : balance.trendChangeKg < 0 ? '−' : '±'}${formatKilograms(Math.abs(balance.trendChangeKg))}`}
			</dd>
			<dd class="small muted">
				{balance.storedKcal === undefined
					? 'needs weigh-ins at both ends'
					: `${formatSigned(balance.storedKcal, 'kcal')} a day into storage`}
			</dd>
		</div>
		<div class="headline">
			<dt>You burned</dt>
			<dd>
				{balance.expenditureKcal === undefined ? '—' : `${formatKcal(balance.expenditureKcal)} a day`}
			</dd>
		</div>
		{#if balance.measuredKcal !== undefined}
			<div>
				<dt>Wearable says</dt>
				<dd>{formatKcal(balance.measuredKcal)} a day</dd>
				<dd class="small muted">
					{balance.measuredDays} days reported{difference === undefined
						? ''
						: ` · ${formatSigned(difference, 'kcal')} against the trend`}
				</dd>
			</div>
		{/if}
	</dl>
</section>

<section class="stack small muted">
	{#if balance.expenditureKcal === undefined}
		<p>
			The estimate needs at least 14 of the last 21 days planned in the calendar, and weigh-ins
			from before the window to near its end.
		</p>
	{/if}
	<p>
		Intake minus what the weight trend stored or released is what you burned, at about 7,700
		kcal per kilogram. Over three weeks this is usually closer than a wrist estimate, which can
		be off by a quarter or more.
	</p>
	<p>
		It assumes you ate what was planned. Days off-plan move the estimate by as much as they
		differed from the plan.
	</p>
</section>

<style>
	.figures {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.figures div {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 0 0.5rem;
	}

	dt {
		color: var(--muted);
	}

	dd {
		margin: 0;
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	dd.small {
		grid-column: 1 / -1;
		text-align: left;
	}

	.headline {
		border-top: 1px solid var(--border);
		padding-top: 0.75rem;
	}

	.headline dt,
	.headline dd {
		color: var(--text);
		font-weight: 600;
	}
</style>
