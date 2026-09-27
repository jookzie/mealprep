<script lang="ts">
	import { goto } from '$app/navigation';
	import { assignCalendarDay, type DayPlan, runMutation, unassignCalendarDay } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import GroupedPicker from '$lib/components/GroupedPicker.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { buildPlannedPeriod, type PlannedDay } from '$lib/domain/calendar';
	import { groupByCategory, nonEmptyGroups } from '$lib/domain/category';
	import { formatKcal } from '$lib/domain/format';
	import {
		datesMatchingWeekdays,
		dayMonthLabel,
		fullDateLabel,
		isCurrentWeek,
		isToday,
		periodLabel,
		periodRange,
		shiftWeeks,
		WEEKS_IN_VIEW,
		weekdayLabel,
		weekLabel,
		weekRange,
	} from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

	const period = $derived(
		buildPlannedPeriod({
			weekStart: data.weekStart,
			days: data.days,
			dayPlans: data.dayPlans,
			targets: data.targets,
		}),
	);

	let assigning = $state<PlannedDay | null>(null);
	let assignOpen = $state(false);

	let bulkOpen = $state(false);
	let bulkPlanId = $state('');
	let bulkWeek = $state('all');
	let bulkWeekdays = $state<number[]>([0, 1, 2, 3, 4]);
	let bulkBusy = $state(false);

	const planGroups = $derived(nonEmptyGroups(groupByCategory(data.dayPlans, data.categories)));

	// Every date a bulk assignment would touch, named before anything is written.
	const bulkDates = $derived.by(() => {
		const { from, to } = bulkWeek === 'all' ? periodRange(data.weekStart) : weekRange(bulkWeek);
		return datesMatchingWeekdays(from, to, bulkWeekdays);
	});

	function turn(weeks: number) {
		goto(`?week=${shiftWeeks(data.weekStart, weeks)}`, { noScroll: true, keepFocus: true });
	}

	function openDay(day: PlannedDay) {
		assigning = day;
		assignOpen = true;
	}

	async function assign(plan: DayPlan) {
		const day = assigning;
		assignOpen = false;
		if (!day) return;
		await runMutation(() => assignCalendarDay(day.date, plan.id), {
			success: `${plan.label} on ${fullDateLabel(day.date)}`,
		});
	}

	async function clear() {
		const day = assigning;
		assignOpen = false;
		if (!day) return;
		await runMutation(() => unassignCalendarDay(day.date), {
			success: `${fullDateLabel(day.date)} cleared`,
		});
	}

	// One write per day, in sequence: SQLite has a single writer, so concurrency would only
	// contend for its lock.
	async function applyBulk() {
		const planId = bulkPlanId;
		const dates = bulkDates;
		bulkBusy = true;
		await runMutation(
			async () => {
				for (const date of dates) await assignCalendarDay(date, planId);
			},
			{ success: `Assigned to ${dates.length} ${dates.length === 1 ? 'day' : 'days'}` },
		);
		bulkBusy = false;
		bulkOpen = false;
	}
</script>

<PageHeader title="Calendar" subtitle={periodLabel(data.weekStart)}>
	{#snippet actions()}
		<button class="primary" onclick={() => (bulkOpen = true)} disabled={data.dayPlans.length === 0}
			>Apply plan</button
		>
	{/snippet}
</PageHeader>

<div class="row spread">
	<button onclick={() => turn(-WEEKS_IN_VIEW)}>← Earlier</button>
	<button onclick={() => goto('/calendar', { noScroll: true })}>Today</button>
	<button onclick={() => turn(WEEKS_IN_VIEW)}>Later →</button>
</div>

<section class="card stack">
	<div class="row spread">
		<h2>Per planned day</h2>
		<span class="muted small">{period.plannedCount} of {period.dayCount} days planned</span>
	</div>
	{#if period.averagePerPlannedDay}
		<MacroMeters actual={period.averagePerPlannedDay} target={period.targetPerDay} />
		<div class="row spread">
			<CostFigure cost={period.averageCostPerPlannedDay} label="Per day" />
			<CostFigure cost={period.totalCost} label="All planned days" />
		</div>
	{:else}
		<p class="muted">No day in view has a plan. Tap a day to assign one.</p>
	{/if}
</section>

{#each period.weeks as week (week.weekStart)}
	<section class="stack" aria-label={weekLabel(week.weekStart)}>
		<div class="row spread">
			<h2 class:current={isCurrentWeek(week.weekStart)}>{weekLabel(week.weekStart)}</h2>
			<span class="muted small">
				{week.plannedCount}/7 planned
				{#if week.averagePerPlannedDay}· {formatKcal(week.averagePerPlannedDay.energyKcal)} a day{/if}
			</span>
		</div>
		<ol class="week">
			{#each week.days as day (day.date)}
				<li>
					<button
						class="day"
						class:today={isToday(day.date)}
						class:unplanned={!day.dayPlan}
						onclick={() => openDay(day)}
					>
						<span class="small"><strong>{weekdayLabel(day.date)}</strong> {dayMonthLabel(day.date)}</span>
						{#if day.dayPlan}
							<span class="label truncate">{day.dayPlan.label}</span>
							<MacroLine macros={day.dayPlan.macros} />
						{:else if day.dayPlanId}
							<span class="muted small">Plan removed</span>
						{:else}
							<span class="muted small">Unplanned</span>
						{/if}
					</button>
				</li>
			{/each}
		</ol>
	</section>
{/each}

<Dialog bind:open={assignOpen} title={assigning ? fullDateLabel(assigning.date) : 'Assign a plan'}>
	{#if data.dayPlans.length === 0}
		<p class="muted">There are no day plans to assign yet.</p>
		<a class="button primary" href="/day-plans/new">New day plan</a>
	{:else}
		<GroupedPicker
			items={data.dayPlans}
			categories={data.categories}
			selectedId={assigning?.dayPlanId ?? undefined}
			placeholder="Filter day plans"
			onPick={assign}
		/>
	{/if}
	{#if assigning?.dayPlanId}
		<button class="danger" onclick={clear}>Clear this day</button>
	{/if}
</Dialog>

<Dialog bind:open={bulkOpen} title="Apply a plan to several days">
	<label class="field">
		<span>Day plan</span>
		<select bind:value={bulkPlanId}>
			<option value="" disabled>Choose a plan</option>
			{#each planGroups as group (group.key)}
				{#if planGroups.length > 1}
					<optgroup label={group.label}>
						{#each group.items as plan (plan.id)}
							<option value={plan.id}>{plan.label}</option>
						{/each}
					</optgroup>
				{:else}
					{#each group.items as plan (plan.id)}
						<option value={plan.id}>{plan.label}</option>
					{/each}
				{/if}
			{/each}
		</select>
	</label>
	<label class="field">
		<span>Weeks</span>
		<select bind:value={bulkWeek}>
			<option value="all">All {WEEKS_IN_VIEW} weeks in view</option>
			{#each period.weeks as week (week.weekStart)}
				<option value={week.weekStart}>{weekLabel(week.weekStart)}</option>
			{/each}
		</select>
	</label>
	<fieldset class="weekdays">
		<legend class="small muted">Days of the week</legend>
		{#each WEEKDAYS as name, index (name)}
			<label class="weekday">
				<input type="checkbox" value={index} bind:group={bulkWeekdays} />
				{name}
			</label>
		{/each}
	</fieldset>
	<p class="small">
		{#if bulkDates.length === 0}
			No days match.
		{:else}
			Touches {bulkDates.length} {bulkDates.length === 1 ? 'day' : 'days'}: {bulkDates
				.map(dayMonthLabel)
				.join(', ')}. A day that already has a plan gets this one instead.
		{/if}
	</p>
	<div class="row end">
		<button onclick={() => (bulkOpen = false)}>Cancel</button>
		<button
			class="primary"
			disabled={bulkPlanId === '' || bulkDates.length === 0 || bulkBusy}
			onclick={applyBulk}>Apply</button
		>
	</div>
</Dialog>

<style>
	.week {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: 1fr;
		gap: 0.4rem;
	}

	@media (min-width: 60rem) {
		.week {
			grid-template-columns: repeat(7, minmax(0, 1fr));
		}
	}

	.day {
		width: 100%;
		height: 100%;
		flex-direction: column;
		align-items: flex-start;
		justify-content: flex-start;
		text-align: left;
		background: var(--surface);
		gap: 0.15rem;
	}

	.day.unplanned {
		background: var(--surface-2);
		border-style: dashed;
	}

	.day.today {
		border-color: var(--accent);
		box-shadow: inset 0 0 0 1px var(--accent);
	}

	.label {
		max-width: 100%;
		font-weight: 600;
	}

	.current::after {
		content: ' · this week';
		color: var(--accent);
		font-weight: 400;
		font-size: 0.85rem;
	}

	.weekdays {
		border: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem 0.75rem;
	}

	.weekday {
		display: inline-flex;
		align-items: center;
		gap: 0.3rem;
		min-height: 44px;
	}
</style>
