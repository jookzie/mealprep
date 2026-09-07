<script lang="ts">
	import CalendarPlusIcon from '@lucide/svelte/icons/calendar-plus';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { page } from '$app/state';
	import { assignCalendarDay, runMutation, unassignCalendarDay } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import ApplyPlanDialog from '$lib/components/calendar/apply-plan-dialog.svelte';
	import AssignPlanDialog from '$lib/components/calendar/assign-plan-dialog.svelte';
	import WeekGrid from '$lib/components/calendar/week-grid.svelte';
	import WeekNav from '$lib/components/calendar/week-nav.svelte';
	import PeriodSummary from '$lib/components/calendar/period-summary.svelte';
	import { Button } from '$lib/components/ui/button';

	let { data } = $props();

	let assigning = $state<string | null>(null);
	let dialogOpen = $state(false);
	let applyOpen = $state(false);

	const returnTo = $derived(`${page.url.pathname}${page.url.search}`);
	const plannedDates = $derived(
		data.period.weeks
			.flatMap((week) => week.days)
			.filter((day) => day.dayPlanId !== null)
			.map((day) => day.date)
	);

	function openAssign(date: string) {
		assigning = date;
		dialogOpen = true;
	}

	async function choose(dayPlanId: string) {
		const date = assigning;
		dialogOpen = false;
		if (!date) return;
		await runMutation(
			() => assignCalendarDay({ path: { date }, body: { dayPlanId }, throwOnError: true }),
			{ success: 'Day plan assigned' }
		);
	}

	async function clear(date: string) {
		await runMutation(() => unassignCalendarDay({ path: { date }, throwOnError: true }), {
			success: 'Day cleared'
		});
	}

	/*
	 * There is no bulk endpoint, so a range assignment is a sequence of writes. They run
	 * one after another rather than at once: the backend holds a single SQLite writer,
	 * and firing seven concurrent PUTs at it only contends for the same lock.
	 */
	async function applyToDates(dayPlanId: string, dates: string[]) {
		await runMutation(
			async () => {
				for (const date of dates) {
					await assignCalendarDay({ path: { date }, body: { dayPlanId }, throwOnError: true });
				}
			},
			{ success: `Plan applied to ${dates.length} ${dates.length === 1 ? 'day' : 'days'}` }
		);
	}
</script>

<PageHeader
	title="Calendar"
	description="Four weeks at a time. A day holds one plan, and assigning over one replaces it."
>
	{#snippet actions()}
		<Button variant="outline" onclick={() => (applyOpen = true)} disabled={data.dayPlans.length === 0}>
			<CalendarPlusIcon class="size-4" />
			Apply a plan
		</Button>
		<Button href="/day-plans/new?returnTo={encodeURIComponent(returnTo)}">
			<PlusIcon class="size-4" />
			New day plan
		</Button>
	{/snippet}
</PageHeader>

<WeekNav weekStart={data.period.weekStart} />

<!-- Totals above the grid: the figure being steered by comes before the days it is
     made of, as every planner that shows one puts it. -->
<PeriodSummary period={data.period} />

<WeekGrid period={data.period} onAssign={openAssign} onUnassign={clear} />

<AssignPlanDialog
	bind:open={dialogOpen}
	date={assigning}
	dayPlans={data.dayPlans}
	{returnTo}
	onChoose={choose}
/>

<ApplyPlanDialog
	bind:open={applyOpen}
	weekStart={data.period.weekStart}
	dayPlans={data.dayPlans}
	{plannedDates}
	onApply={applyToDates}
/>
