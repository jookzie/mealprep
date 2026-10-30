<script lang="ts">
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Plus from '@lucide/svelte/icons/plus';
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { assignCalendarDay, type DayPlan, runMutation, unassignCalendarDay } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import GroupedPicker from '$lib/components/GroupedPicker.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SectionSwitch from '$lib/components/SectionSwitch.svelte';
	import { assignmentEffects, buildPlannedPeriod } from '$lib/domain/calendar';
	import { categoryName, groupByCategory, nonEmptyGroups } from '$lib/domain/category';
	import { itemLabel } from '$lib/domain/day-plan';
	import { formatAmount, formatSigned } from '$lib/domain/format';
	import { MACRO_LABELS, MACRO_SERIES } from '$lib/domain/macros';
	import {
		datesMatchingWeekdays,
		dayMonthLabel,
		dayOfMonth,
		headingDateLabel,
		isToday,
		periodLabel,
		periodRange,
		shiftWeeks,
		todayIso,
		WEEKS_IN_VIEW,
		weekdayLabel,
		weekLabel,
		weekRange,
	} from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const WEEKDAYS = [
		{ short: 'M', name: 'Monday' },
		{ short: 'T', name: 'Tuesday' },
		{ short: 'W', name: 'Wednesday' },
		{ short: 'T', name: 'Thursday' },
		{ short: 'F', name: 'Friday' },
		{ short: 'S', name: 'Saturday' },
		{ short: 'S', name: 'Sunday' },
	];
	const HUES = { proteinG: 'var(--protein)', fatG: 'var(--fat)', carbohydratesG: 'var(--carbs)' };
	const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

	const period = $derived(
		buildPlannedPeriod({
			weekStart: data.weekStart,
			days: data.days,
			dayPlans: data.joinedPlans,
			targets: data.targets,
		}),
	);
	const allDays = $derived(period.weeks.flatMap((week) => week.days));
	const selected = $derived(allDays.find((day) => day.date === data.day) ?? allDays[0]);
	const plan = $derived(selected.dayPlan);

	let strip = $state<HTMLElement>();
	// The picked day is brought into view in the strip, whichever end of the block it is at.
	$effect(() => {
		void data.day;
		tick().then(() =>
			strip
				?.querySelector('[aria-pressed="true"]')
				?.scrollIntoView({ inline: 'center', block: 'nearest' }),
		);
	});

	function pick(date: string) {
		goto(`?week=${data.weekStart}&day=${date}`, { replaceState: true, noScroll: true, keepFocus: true });
	}

	function turn(weeks: number) {
		const weekStart = shiftWeeks(data.weekStart, weeks);
		goto(`?week=${weekStart}`, { noScroll: true, keepFocus: true });
	}

	let assignOpen = $state(false);

	async function assign(chosen: DayPlan) {
		const date = selected.date;
		assignOpen = false;
		await runMutation(() => assignCalendarDay(date, chosen.id), {
			success: `${chosen.label} on ${dayMonthLabel(date)}`,
		});
	}

	async function clear() {
		const date = selected.date;
		assignOpen = false;
		await runMutation(() => unassignCalendarDay(date), {
			success: `${dayMonthLabel(date)} cleared`,
		});
	}

	let bulkOpen = $state(false);
	let bulkPlanId = $state('');
	let bulkWeek = $state('all');
	let bulkWeekdays = $state<number[]>([0, 1, 2, 3, 4]);
	let bulkBusy = $state(false);

	const planGroups = $derived(nonEmptyGroups(groupByCategory(data.dayPlans, data.categories)));
	const bulkPlans = $derived(planGroups.flatMap((group) => group.items));

	// Every date a bulk assignment would touch, and what it does to each, named before
	// anything is written (CL-4).
	const bulkDates = $derived.by(() => {
		const { from, to } = bulkWeek === 'all' ? periodRange(data.weekStart) : weekRange(bulkWeek);
		return datesMatchingWeekdays(from, to, bulkWeekdays);
	});
	const bulkEffects = $derived(
		bulkPlanId === '' ? [] : assignmentEffects(bulkDates, data.days, bulkPlanId),
	);
	const counts = $derived({
		same: bulkEffects.filter((entry) => entry.effect === 'same').length,
		replace: bulkEffects.filter((entry) => entry.effect === 'replace').length,
		new: bulkEffects.filter((entry) => entry.effect === 'new').length,
	});

	function toggleWeekday(index: number) {
		bulkWeekdays = bulkWeekdays.includes(index)
			? bulkWeekdays.filter((day) => day !== index)
			: [...bulkWeekdays, index].sort();
	}

	// One write per day, in sequence: SQLite has a single writer, so concurrency would only
	// contend for its lock.
	async function applyBulk() {
		const planId = bulkPlanId;
		const dates = bulkEffects.filter((entry) => entry.effect !== 'same').map((entry) => entry.date);
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

<PageHeader title="Plan" eyebrow={periodLabel(data.weekStart)}>
	{#snippet actions()}
		<button class="outline" onclick={() => (bulkOpen = true)} disabled={data.dayPlans.length === 0}>
			<Plus size={18} strokeWidth={2.6} aria-hidden="true" />
			Apply to days
		</button>
	{/snippet}
</PageHeader>

<SectionSwitch
	label="Plan"
	current="/calendar"
	links={[
		{ href: '/calendar', label: 'Calendar' },
		{ href: '/day-plans', label: 'Day plans' },
	]}
/>

<div class="paging">
	<button class="icon" aria-label="Earlier {WEEKS_IN_VIEW} weeks" onclick={() => turn(-WEEKS_IN_VIEW)}>
		<ChevronLeft size={20} strokeWidth={2.6} aria-hidden="true" />
	</button>
	<button class="today-button" onclick={() => goto('/calendar', { noScroll: true })}>Today</button>
	<button class="icon" aria-label="Later {WEEKS_IN_VIEW} weeks" onclick={() => turn(WEEKS_IN_VIEW)}>
		<ChevronRight size={20} strokeWidth={2.6} aria-hidden="true" />
	</button>
</div>

<div class="chips strip" bind:this={strip} role="group" aria-label="Days in view">
	{#each allDays as day (day.date)}
		<button
			class="pill-day"
			class:today={isToday(day.date)}
			class:unplanned={!day.dayPlan}
			aria-pressed={day.date === selected.date}
			aria-label="{headingDateLabel(day.date)}: {day.dayPlan?.label ?? 'unplanned'}"
			onclick={() => pick(day.date)}
		>
			<span class="small">{weekdayLabel(day.date)}</span>
			<span class="display num">{dayOfMonth(day.date)}</span>
			<span class="dot" aria-hidden="true"></span>
		</button>
	{/each}
</div>

<section class="day-card" class:empty-day={!plan}>
	<div class="row spread top">
		<div class="grow">
			<p class="small strong">{headingDateLabel(selected.date)}</p>
			<h2 class="label">
				{plan?.label ?? (selected.dayPlanId ? 'Plan removed' : 'Unplanned')}
			</h2>
			{#if plan}
				<p class="small">{categoryName(plan, data.categories) ?? ''}</p>
			{/if}
		</div>
		<button class="primary" onclick={() => (assignOpen = true)} disabled={data.dayPlans.length === 0}>
			{plan ? 'Swap' : 'Pick a plan'}
		</button>
	</div>

	{#if plan}
		<div class="row tags">
			<span class="tag ink figure">{whole.format(plan.macros.energyKcal)} kcal</span>
			{#if data.targets}
				{@const delta = plan.macros.energyKcal - data.targets.macros.energyKcal}
				<span class="tag figure">
					{formatSigned(Math.abs(delta), 'kcal').replace(' kcal', '')}
					{delta >= 0 ? 'over' : 'under'}
				</span>
			{/if}
			<span class="tag soft"><CostFigure cost={plan.cost} /></span>
		</div>
		<div class="mini">
			{#each MACRO_SERIES as key (key)}
				<div style:background={HUES[key]}>
					<span class="small strong">{MACRO_LABELS[key]}</span>
					<span class="display figure">
						{whole.format(plan.macros[key])}{#if data.targets}<small
								>/{whole.format(data.targets.macros[key])}</small
							>{/if}
					</span>
				</div>
			{/each}
		</div>
	{:else if data.dayPlans.length === 0}
		<p class="small">
			There are no day plans yet. <a href="/day-plans/new">Make the first one</a>.
		</p>
	{/if}
</section>

{#if plan && plan.items.length > 0}
	<ol class="items">
		{#each plan.items as item, index (index)}
			{@const macros = item.kind === 'meal' ? item.meal.macros : item.product.macros}
			<li>
				<span class="display index">{index + 1}</span>
				<span class="grow truncate strong" class:muted={item.kind === 'product' && !item.product.name}
					>{itemLabel(item)}</span
				>
				{#if item.kind === 'product'}
					<span class="muted small">{formatAmount(item.product.amount, item.product.unit)}</span>
				{/if}
				<span class="figure strong kcal">{whole.format(macros.energyKcal)}</span>
			</li>
		{/each}
	</ol>
	<a class="button outline open" href="/day-plans/{plan.id}">Open {plan.label}</a>
{/if}

<section class="stack summary">
	<div class="row spread">
		<h2>Per planned day</h2>
		<span class="muted small strong">{period.plannedCount} of {period.dayCount} days planned</span>
	</div>
	{#if period.averagePerPlannedDay}
		<MacroMeters actual={period.averagePerPlannedDay} target={period.targetPerDay} caption="kcal a day, on average">
			{#snippet aside()}
				<span class="avg-cost"><CostFigure cost={period.averageCostPerPlannedDay} /></span>
			{/snippet}
		</MacroMeters>
		<p class="small muted">
			All planned days together: <CostFigure cost={period.totalCost} />. Unplanned days count in
			no average.
		</p>
	{:else}
		<p class="muted">No day in view has a plan yet.</p>
	{/if}
</section>

<Dialog bind:open={assignOpen} title={headingDateLabel(selected.date)} subtitle="Pick what this day eats">
	<GroupedPicker
		items={data.dayPlans}
		categories={data.categories}
		selectedId={selected.dayPlanId ?? undefined}
		placeholder="Filter day plans"
		onPick={assign}
	/>
	{#if selected.dayPlanId}
		<button class="outline wide" onclick={clear}>Leave this day unplanned</button>
	{/if}
</Dialog>

<Dialog bind:open={bulkOpen} title="Apply to days" tall>
	<div class="stack bulk">
		<fieldset class="stack">
			<legend class="strong small">1 · Which plan</legend>
			<div class="chips">
				{#each bulkPlans as option (option.id)}
					<button
						type="button"
						class="chip choice"
						aria-pressed={bulkPlanId === option.id}
						onclick={() => (bulkPlanId = option.id)}>{option.label}</button
					>
				{/each}
			</div>
		</fieldset>

		<fieldset class="stack">
			<legend class="strong small">2 · On which weekdays</legend>
			<div class="weekdays">
				{#each WEEKDAYS as weekday, index (weekday.name)}
					<button
						type="button"
						class="weekday"
						aria-label={weekday.name}
						aria-pressed={bulkWeekdays.includes(index)}
						onclick={() => toggleWeekday(index)}>{weekday.short}</button
					>
				{/each}
			</div>
		</fieldset>

		<fieldset class="stack">
			<legend class="strong small">3 · Over which weeks</legend>
			<div class="chips">
				<button type="button" class="chip" aria-pressed={bulkWeek === 'all'} onclick={() => (bulkWeek = 'all')}>
					All {WEEKS_IN_VIEW} weeks
				</button>
				{#each period.weeks as week (week.weekStart)}
					<button
						type="button"
						class="chip"
						aria-pressed={bulkWeek === week.weekStart}
						onclick={() => (bulkWeek = week.weekStart)}>{weekLabel(week.weekStart)}</button
					>
				{/each}
			</div>
		</fieldset>

		<div class="preview stack" aria-live="polite">
			{#if bulkDates.length === 0}
				<p class="strong">No days match.</p>
			{:else if bulkPlanId === ''}
				<p class="strong">Touches {bulkDates.length} {bulkDates.length === 1 ? 'day' : 'days'} — pick a plan.</p>
			{:else}
				<p class="display touches">Touches {bulkDates.length} {bulkDates.length === 1 ? 'day' : 'days'}</p>
				<ul class="dates">
					{#each bulkEffects as entry (entry.date)}
						<li class={entry.effect}>
							<span class="small">{weekdayLabel(entry.date)}</span>
							<span class="strong">{dayMonthLabel(entry.date)}</span>
						</li>
					{/each}
				</ul>
				<div class="row legend small strong">
					<span><i class="same"></i>Already on it · {counts.same}</span>
					<span><i class="replace"></i>Replaced · {counts.replace}</span>
					<span><i class="new"></i>Newly planned · {counts.new}</span>
				</div>
				{#if bulkEffects.some((entry) => entry.effect !== 'same' && entry.date < todayIso())}
					<p class="small">Some of these days are already behind you.</p>
				{/if}
			{/if}
		</div>

		<button
			class="primary wide"
			disabled={bulkPlanId === '' || counts.replace + counts.new === 0 || bulkBusy}
			onclick={applyBulk}
		>
			{#if bulkPlanId !== '' && counts.replace + counts.new > 0}
				Assign to {counts.replace + counts.new} {counts.replace + counts.new === 1 ? 'day' : 'days'}
			{:else}
				Assign
			{/if}
		</button>
	</div>
</Dialog>

<style>
	.strong {
		font-weight: 700;
	}

	.paging {
		display: flex;
		gap: 0.5rem;
	}

	.today-button {
		flex: 1;
	}

	.strip {
		padding-bottom: 2px;
	}

	.pill-day {
		flex: 0 0 auto;
		width: 54px;
		height: 80px;
		padding: 0;
		border-radius: 20px;
		flex-direction: column;
		gap: 2px;
		background: var(--surface);
		font-weight: 600;
	}

	.pill-day.today {
		background: var(--protein);
		color: var(--on-macro);
	}

	.pill-day.unplanned {
		background: transparent;
		border: 2px dashed var(--border);
	}

	.pill-day[aria-pressed='true'] {
		background: var(--accent);
		color: var(--accent-text);
		border-color: transparent;
	}

	.pill-day .small {
		opacity: 0.8;
	}

	.num {
		font-size: 1.35rem;
		line-height: 1;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 4px;
		background: currentColor;
	}

	.unplanned .dot {
		background: transparent;
	}

	.day-card {
		background: var(--carbs);
		color: var(--on-macro);
		border-radius: var(--radius-lg);
		padding: 1.1rem;
		display: flex;
		flex-direction: column;
		gap: 0.9rem;
	}

	.day-card.empty-day {
		background: transparent;
		color: var(--text);
		border: 2px dashed var(--border);
	}

	.top {
		flex-wrap: nowrap;
		align-items: flex-start;
	}

	.label {
		font-size: 1.85rem;
		line-height: 1.05;
	}

	.tags {
		gap: 0.4rem;
	}

	.tag {
		font-weight: 700;
		padding: 0.35rem 0.8rem;
		border-radius: 999px;
		background: #ffffff;
		color: #111111;
	}

	.tag.ink {
		background: #111111;
		color: #ffffff;
	}

	.tag.soft {
		background: rgb(255 255 255 / 0.55);
	}

	.mini {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 8px;
	}

	.mini > div {
		border-radius: 16px;
		padding: 0.6rem 0.7rem;
		display: flex;
		flex-direction: column;
	}

	.day-card .mini > :nth-child(3) {
		background: #ffffff !important;
	}

	.mini .display {
		font-size: 1.35rem;
	}

	.mini small {
		font-size: 0.8rem;
	}

	.items {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.items li {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0.65rem 0;
		border-bottom: 1.5px solid var(--border);
	}

	.index {
		width: 1.2rem;
		color: var(--muted);
	}

	.kcal {
		min-width: 2.75rem;
		text-align: right;
	}

	.open {
		align-self: flex-start;
	}

	.summary {
		margin-top: 0.5rem;
	}

	.avg-cost {
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: rgb(127 127 127 / 0.2);
	}

	.wide {
		width: 100%;
		min-height: 52px;
	}

	.bulk {
		gap: 1.1rem;
	}

	.choice[aria-pressed='true'] {
		background: var(--protein);
		color: var(--on-macro);
		border-color: var(--text);
	}

	.weekdays {
		display: grid;
		grid-template-columns: repeat(7, minmax(0, 1fr));
		gap: 6px;
	}

	.weekday {
		min-height: 48px;
		padding: 0;
		border-radius: 16px;
		font-weight: 800;
	}

	.weekday[aria-pressed='true'] {
		background: var(--accent);
		color: var(--accent-text);
	}

	.preview {
		background: var(--surface);
		border-radius: var(--radius);
		padding: 0.9rem;
		gap: 0.7rem;
	}

	.touches {
		font-size: 1.3rem;
	}

	.dates {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 6px;
	}

	.dates li {
		min-height: 52px;
		border-radius: 14px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		line-height: 1.15;
		color: #111111;
	}

	.same {
		background: var(--raised);
		border: 1.5px solid var(--border);
	}

	.dates li.same {
		color: var(--text);
	}

	.replace {
		background: var(--protein);
	}

	.new {
		background: var(--carbs);
	}

	.legend {
		gap: 0.5rem 0.9rem;
	}

	.legend span {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
	}

	.legend i {
		width: 12px;
		height: 12px;
		border-radius: 4px;
		display: inline-block;
	}
</style>
