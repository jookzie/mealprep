<script lang="ts">
	import type { PlannedWeek } from '$lib/domain/calendar';
	import DayCell from './day-cell.svelte';

	/*
	 * A true seven-track week. Every shipped weekly planner uses one, and seven cards
	 * reflowing into a ragged 4+3 block stops the row being a week at all — Thursday
	 * lands under Monday and the shape carries no meaning.
	 *
	 * Below the breakpoint it becomes a stacked day list rather than a squeezed grid.
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
</script>

<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-[repeat(7,minmax(0,1fr))]">
	{#each week.days as day (day.date)}
		<DayCell {day} {onAssign} {onUnassign} />
	{/each}
</div>
