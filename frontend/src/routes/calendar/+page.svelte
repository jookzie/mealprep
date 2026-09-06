<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { page } from '$app/state';
	import { assignCalendarDay, runMutation, unassignCalendarDay } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import AssignPlanDialog from '$lib/components/calendar/assign-plan-dialog.svelte';
	import WeekGrid from '$lib/components/calendar/week-grid.svelte';
	import WeekNav from '$lib/components/calendar/week-nav.svelte';
	import WeekSummary from '$lib/components/calendar/week-summary.svelte';
	import { Button } from '$lib/components/ui/button';

	let { data } = $props();

	let assigning = $state<string | null>(null);
	let dialogOpen = $state(false);

	const returnTo = $derived(`${page.url.pathname}${page.url.search}`);

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
</script>

<PageHeader
	title="Calendar"
	description="A day holds one plan. Assigning to a day that already has one replaces it."
>
	{#snippet actions()}
		<Button href="/day-plans/new?returnTo={encodeURIComponent(returnTo)}" variant="outline">
			<PlusIcon class="size-4" />
			New day plan
		</Button>
	{/snippet}
</PageHeader>

<WeekNav weekStart={data.week.weekStart} />

<WeekGrid week={data.week} onAssign={openAssign} onUnassign={clear} />

<WeekSummary week={data.week} />

<AssignPlanDialog
	bind:open={dialogOpen}
	date={assigning}
	dayPlans={data.dayPlans}
	{returnTo}
	onChoose={choose}
/>
