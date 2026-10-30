<script lang="ts">
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ScanButton from '$lib/components/ScanButton.svelte';
	import { itemLabel } from '$lib/domain/day-plan';
	import { formatAmount } from '$lib/domain/format';
	import { dayOfMonth, headingDateLabel, weekdayLabel } from '$lib/domain/week';
	import { formatKilograms, formatRate } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const plan = $derived(data.today.dayPlan);
	const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
</script>

<PageHeader
	eyebrow={headingDateLabel(data.today.date)}
	title={plan?.label ?? (data.today.dayPlanId ? 'Plan removed' : 'Nothing planned')}
>
	{#snippet actions()}
		<ScanButton compact />
	{/snippet}
</PageHeader>

{#if plan}
	<MacroMeters actual={plan.macros} target={data.targets?.macros ?? null} caption="kcal planned">
		{#snippet aside()}
			<span class="cost"><CostFigure cost={plan.cost} /></span>
		{/snippet}
	</MacroMeters>
	{#if !data.targets}
		<p class="muted small"><a href="/you">Set daily targets</a> to see how the day compares.</p>
	{/if}

	<div class="row spread">
		<h2>In order</h2>
		<a class="button outline compact" href="/day-plans/{plan.id}">Open plan</a>
	</div>

	{#if plan.items.length === 0}
		<p class="empty">Nothing in this plan yet.</p>
	{:else}
		<ol class="items">
			{#each plan.items as item, index (index)}
				{@const macros = item.kind === 'meal' ? item.meal.macros : item.product.macros}
				<li>
					<a
						class="item"
						href={item.kind === 'meal' ? `/meals/${item.meal.id}` : `/products/${item.product.id}`}
					>
						<span class="display index">{index + 1}</span>
						<span class="grow body">
							<strong class="truncate" class:muted={item.kind === 'product' && !item.product.name}
								>{itemLabel(item)}</strong
							>
							<span class="muted small">
								{#if item.kind === 'meal'}
									Meal · {item.meal.servings.length}
									{item.meal.servings.length === 1 ? 'product' : 'products'}
								{:else}
									{formatAmount(item.product.amount, item.product.unit)}
								{/if}
							</span>
						</span>
						<span class="kcal figure">{whole.format(macros.energyKcal)}</span>
					</a>
				</li>
			{/each}
		</ol>
	{/if}
{:else}
	<section class="card stack open-day">
		<p>
			{#if data.today.dayPlanId}
				The plan this day pointed at has been deleted, so today reads as unplanned.
			{:else}
				No day plan is assigned to today.
			{/if}
		</p>
		<a class="button primary" href="/calendar?day={data.today.date}">Pick a plan for today</a>
	</section>
{/if}

<a class="card weight" href="/weight">
	<span class="grow">
		<span class="small label">Weight trend</span>
		<span class="display trend figure">
			{data.weight.trend === undefined ? '—' : formatKilograms(data.weight.trend)}
		</span>
	</span>
	{#if data.weight.ratePerWeek !== undefined}
		<span class="rate">{formatRate(data.weight.ratePerWeek)}</span>
	{:else}
		<span class="muted small">Log a weigh-in</span>
	{/if}
	<ChevronRight size={20} aria-hidden="true" />
</a>

<section class="stack">
	<h2>Coming up</h2>
	<ol class="ahead">
		{#each data.ahead as day (day.date)}
			<li>
				<a class="day" class:unplanned={!day.dayPlan} href="/calendar?day={day.date}">
					<span class="date">
						<span class="small">{weekdayLabel(day.date)}</span>
						<span class="display">{dayOfMonth(day.date)}</span>
					</span>
					<span class="grow truncate label">{day.dayPlan?.label ?? 'Unplanned'}</span>
					{#if day.dayPlan}
						<span class="figure small">{whole.format(day.dayPlan.macros.energyKcal)} kcal</span>
					{/if}
				</a>
			</li>
		{/each}
	</ol>
</section>

<style>
	.cost {
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: rgb(255 255 255 / 0.12);
		color: var(--hero-text);
	}

	.compact {
		min-height: 40px;
		font-size: 0.9rem;
	}

	.items,
	.ahead {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.item {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 16px;
		background: var(--surface);
		border-radius: var(--radius);
		text-decoration: none;
	}

	.index {
		font-size: 1.6rem;
		width: 1.4rem;
	}

	.body {
		display: flex;
		flex-direction: column;
	}

	.kcal {
		font-weight: 700;
		padding: 0.35rem 0.6rem;
		border-radius: 12px;
		background: var(--raised);
	}

	.open-day {
		align-items: flex-start;
	}

	.weight {
		display: flex;
		align-items: center;
		gap: 12px;
		text-decoration: none;
		background: transparent;
		border: 2px solid var(--text);
	}

	.weight .grow {
		display: flex;
		flex-direction: column;
	}

	.label {
		font-weight: 700;
	}

	.trend {
		font-size: 1.75rem;
		line-height: 1.1;
	}

	.rate {
		font-weight: 700;
		font-size: 0.85rem;
		padding: 0.25rem 0.65rem;
		border-radius: 10px;
		background: var(--carbs);
		color: var(--on-macro);
		white-space: nowrap;
	}

	.day {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 8px 16px 8px 8px;
		border-radius: var(--radius);
		background: var(--surface);
		text-decoration: none;
	}

	.day.unplanned {
		background: transparent;
		border: 2px dashed var(--border);
	}

	.date {
		width: 48px;
		height: 48px;
		border-radius: 16px;
		background: var(--raised);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		line-height: 1.1;
	}

	.date .small {
		font-weight: 600;
		color: var(--muted);
	}

	.date .display {
		font-size: 1.15rem;
	}

	.label {
		font-weight: 700;
	}

	.unplanned .label {
		color: var(--muted);
	}
</style>
