<script lang="ts">
	import type { Macros } from '$lib/api';
	import type { PlannedWeek } from '$lib/domain/calendar';
	import DayCell from './day-cell.svelte';

	let {
		week,
		onAssign,
		onUnassign
	}: {
		week: PlannedWeek;
		onAssign: (date: string) => void;
		onUnassign: (date: string) => void;
	} = $props();

	const target: Macros | null = $derived(week.targetPerDay);
</script>

<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
	{#each week.days as day (day.date)}
		<DayCell {day} {target} {onAssign} {onUnassign} />
	{/each}
</div>
