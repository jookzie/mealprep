<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import MacroStrip from '$lib/components/macros/macro-strip.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import type { PlannedDay } from '$lib/domain/calendar';
	import { dayMonthLabel, isToday, weekdayLabel } from '$lib/domain/week';

	/*
	 * One column of the week. It lists the plan's meals rather than only its label,
	 * because the items in the cell are what make a week grid scannable at all — the
	 * label alone tells you a day is planned but not what it holds.
	 *
	 * The replace and clear actions live behind the row menu so seven cells do not
	 * carry fourteen buttons.
	 */
	let {
		day,
		onAssign,
		onUnassign
	}: {
		day: PlannedDay;
		onAssign: (date: string) => void;
		onUnassign: (date: string) => void;
	} = $props();

	const today = $derived(isToday(day.date));
</script>

<div
	class="flex min-h-44 flex-col rounded-lg border {today
		? 'border-primary/70 bg-primary/[0.03]'
		: 'bg-card'}"
>
	<!-- A fixed height so the seven headers line up whether or not the day has a menu. -->
	<div class="flex h-10 items-center justify-between gap-1 border-b px-3">
		<div class="flex items-baseline gap-1.5">
			<span class="text-sm font-medium">{weekdayLabel(day.date)}</span>
			<span class="text-muted-foreground text-xs tabular-nums">{dayMonthLabel(day.date)}</span>
		</div>
		{#if day.dayPlanId}
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="size-6"
							aria-label="Change {weekdayLabel(day.date)}"
						>
							<EllipsisIcon class="size-4" />
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end">
					<DropdownMenu.Item onSelect={() => onAssign(day.date)}>Replace plan</DropdownMenu.Item>
					<DropdownMenu.Item onSelect={() => onUnassign(day.date)}>Clear day</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/if}
	</div>

	{#if day.dayPlan && day.macros}
		<div class="flex flex-1 flex-col gap-2 p-3">
			<a href="/day-plans/{day.dayPlan.id}" class="text-sm font-medium hover:underline">
				{day.dayPlan.label}
			</a>
			<MacroStrip macros={day.macros} />
			{#if day.dayPlan.meals.length > 0}
				<ul class="text-muted-foreground mt-auto space-y-0.5 text-xs">
					{#each day.dayPlan.meals as meal (meal.id)}
						<li class="truncate">{meal.label}</li>
					{/each}
				</ul>
			{/if}
		</div>
	{:else if day.dayPlanId}
		<!-- Assigned to a plan that has since been deleted. The id is kept so the day can
		     still be cleared. -->
		<div class="flex flex-1 flex-col gap-2 p-3">
			<p class="text-muted-foreground text-sm italic">Plan removed</p>
			<Button variant="outline" size="sm" class="mt-auto" onclick={() => onAssign(day.date)}>
				Assign a plan
			</Button>
		</div>
	{:else}
		<!-- An unplanned day says so, and is excluded from the week's denominator. It is
		     never drawn as 0 kcal, which would read as a shortfall against the target. -->
		<div class="flex flex-1 flex-col p-3">
			<p class="text-muted-foreground/70 text-xs">Unplanned</p>
			<Button
				variant="ghost"
				size="sm"
				class="text-muted-foreground mt-auto w-full justify-start"
				onclick={() => onAssign(day.date)}
			>
				<PlusIcon class="size-4" />
				Assign
			</Button>
		</div>
	{/if}
</div>
