<script lang="ts">
	import type { PlannedWeek } from '$lib/domain/calendar';
	import { formatKcal } from '$lib/domain/format';
	import { isCurrentWeek, weekLabel } from '$lib/domain/week';
	import DayCell from './day-cell.svelte';

	/*
	 * One week of the block: its own seven columns, with a spine on the left carrying
	 * the figures for that week alone. Each week keeps its own honest denominator —
	 * "3 of 7 days planned" — so a sparse week is never averaged into looking full.
	 */
	let {
		week,
		onAssign,
		onUnassign
	}: {
		week: PlannedWeek;
		onAssign: (date: string) => void;
		onUnassign: (date: string) => void;
	} = $props();

	const current = $derived(isCurrentWeek(week.weekStart));
</script>

<section class="grid gap-2 lg:grid-cols-[8.5rem_minmax(0,1fr)]">
	<div class="flex flex-row items-baseline gap-3 lg:flex-col lg:items-start lg:gap-0.5 lg:pt-2">
		<h3 class="text-sm font-medium tabular-nums">
			{weekLabel(week.weekStart)}
			{#if current}
				<span class="text-primary ml-1 text-xs font-normal">this week</span>
			{/if}
		</h3>
		<p class="text-muted-foreground text-xs tabular-nums">
			{week.plannedCount} of 7 planned
		</p>
		{#if week.averagePerPlannedDay}
			<p class="text-muted-foreground text-xs tabular-nums">
				{formatKcal(week.averagePerPlannedDay.energyKcal)} a day
			</p>
		{/if}
	</div>

	<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-[repeat(7,minmax(0,1fr))]">
		{#each week.days as day (day.date)}
			<DayCell {day} {onAssign} {onUnassign} />
		{/each}
	</div>
</section>
