<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import { Button } from '$lib/components/ui/button';
	import {
		type IsoDate,
		periodLabel,
		shiftWeeks,
		startOfIsoWeek,
		todayIso,
		WEEKS_IN_VIEW
	} from '$lib/domain/week';

	// The calendar shows a block of weeks, so it pages by the whole block rather than
	// by one week: stepping a week at a time would leave three quarters of the view
	// unchanged and make the movement hard to follow.
	let { weekStart }: { weekStart: IsoDate } = $props();

	const thisBlock = $derived(startOfIsoWeek(todayIso()));
</script>

<div class="flex flex-wrap items-center gap-2">
	<Button
		href="?week={shiftWeeks(weekStart, -WEEKS_IN_VIEW)}"
		variant="outline"
		size="icon"
		title="Previous {WEEKS_IN_VIEW} weeks"
	>
		<ChevronLeftIcon class="size-4" />
	</Button>
	<Button
		href="?week={shiftWeeks(weekStart, WEEKS_IN_VIEW)}"
		variant="outline"
		size="icon"
		title="Next {WEEKS_IN_VIEW} weeks"
	>
		<ChevronRightIcon class="size-4" />
	</Button>
	<span class="px-1 text-sm font-medium tabular-nums">{periodLabel(weekStart)}</span>
	{#if weekStart !== thisBlock}
		<Button href="?week={thisBlock}" variant="ghost" size="sm">This week</Button>
	{/if}
</div>
