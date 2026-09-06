<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import { Button } from '$lib/components/ui/button';
	import { type IsoDate, shiftWeeks, startOfIsoWeek, todayIso, weekLabel } from '$lib/domain/week';

	let { weekStart }: { weekStart: IsoDate } = $props();

	const thisWeek = $derived(startOfIsoWeek(todayIso()));
</script>

<div class="flex flex-wrap items-center gap-2">
	<Button href="?week={shiftWeeks(weekStart, -1)}" variant="outline" size="icon" title="Previous week">
		<ChevronLeftIcon class="size-4" />
	</Button>
	<Button href="?week={shiftWeeks(weekStart, 1)}" variant="outline" size="icon" title="Next week">
		<ChevronRightIcon class="size-4" />
	</Button>
	<span class="px-1 text-sm font-medium tabular-nums">{weekLabel(weekStart)}</span>
	{#if weekStart !== thisWeek}
		<Button href="?week={thisWeek}" variant="ghost" size="sm">This week</Button>
	{/if}
</div>
