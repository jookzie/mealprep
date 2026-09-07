<script lang="ts">
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import MacroMeters from '$lib/components/macros/macro-meters.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';
	import type { PlannedPeriod } from '$lib/domain/calendar';
	import { formatKcal } from '$lib/domain/format';

	/*
	 * The block's totals, in a strip above the grid rather than below it — the figure
	 * you are steering by belongs where you see it before you start editing days.
	 *
	 * The comparison defaults to the average over the days actually planned, because
	 * the target the user set is a daily figure. A four-week total cannot be read
	 * against a daily target at all, so it is the secondary framing.
	 */
	let { period }: { period: PlannedPeriod } = $props();

	let framing = $state<'average' | 'total'>('average');

	const shown = $derived(framing === 'average' ? period.averagePerPlannedDay : period.total);
	const against = $derived(framing === 'average' ? period.targetPerDay : period.targetTotal);
</script>

<div class="bg-card rounded-lg border">
	<div class="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3">
		<div>
			<h2 class="text-sm font-medium">
				{period.plannedCount} of {period.dayCount} days planned
			</h2>
			<p class="text-muted-foreground text-xs tabular-nums">
				{formatKcal(period.total.energyKcal)} across {period.weeks.length} weeks
			</p>
		</div>

		<div class="flex flex-wrap items-center gap-3">
			{#if period.targetPerDay}
				<!-- The target profile is reachable from the planner, which is where you
				     notice it needs changing. -->
				<a
					href="/targets"
					class="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs tabular-nums"
				>
					Target {formatKcal(period.targetPerDay.energyKcal)} a day
					<ChevronRightIcon class="size-3.5" />
				</a>
			{/if}
			<ToggleGroup.Root
				type="single"
				size="sm"
				variant="outline"
				value={framing}
				onValueChange={(value) => {
					if (value) framing = value as 'average' | 'total';
				}}
			>
				<ToggleGroup.Item value="average" aria-label="Per planned day">Per day</ToggleGroup.Item>
				<ToggleGroup.Item value="total" aria-label="Block total">All {period.weeks.length} weeks</ToggleGroup.Item>
			</ToggleGroup.Root>
		</div>
	</div>

	<div class="p-4">
		{#if period.plannedCount === 0}
			<p class="text-muted-foreground text-sm">
				Nothing is planned in these weeks yet. Assign a day plan to a day to see it measured.
			</p>
		{:else if shown && against}
			<div class="grid items-start gap-6 lg:grid-cols-2">
				<MacrosSummary macros={shown} variant="pair" />
				<MacroMeters actual={shown} target={against} />
			</div>
		{:else if shown}
			<div class="space-y-3">
				<MacrosSummary macros={shown} variant="grid" />
				<p class="text-muted-foreground text-sm">
					<a href="/targets" class="underline underline-offset-4">Set your daily targets</a>
					to see these weeks measured against them.
				</p>
			</div>
		{/if}
	</div>
</div>
