<script lang="ts">
	import type { PlannedPeriod } from '$lib/domain/calendar';
	import { weekdayLabel } from '$lib/domain/week';
	import WeekRow from './week-row.svelte';

	/*
	 * Four true seven-track weeks, stacked. Every shipped weekly planner uses a real
	 * week row; cards reflowing into a ragged block stop the row being a week at all.
	 *
	 * Below the breakpoint the whole thing becomes a stacked day list, so the weekday
	 * header is hidden there — it only labels columns that exist.
	 */
	let {
		period,
		onAssign,
		onUnassign
	}: {
		period: PlannedPeriod;
		onAssign: (date: string) => void;
		onUnassign: (date: string) => void;
	} = $props();
</script>

<div class="space-y-4">
	<div class="hidden gap-2 lg:grid lg:grid-cols-[8.5rem_minmax(0,1fr)]">
		<div></div>
		<div class="grid grid-cols-[repeat(7,minmax(0,1fr))] gap-2">
			{#each period.weeks[0].days as day (day.date)}
				<div class="text-muted-foreground px-3 text-xs font-medium">{weekdayLabel(day.date)}</div>
			{/each}
		</div>
	</div>

	{#each period.weeks as week (week.weekStart)}
		<WeekRow {week} {onAssign} {onUnassign} />
	{/each}
</div>
