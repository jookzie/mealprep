<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import XIcon from '@lucide/svelte/icons/x';
	import type { Macros } from '$lib/api';
	import TargetProgress from '$lib/components/macros/target-progress.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import type { PlannedDay } from '$lib/domain/calendar';
	import { formatKcal } from '$lib/domain/format';
	import { dayMonthLabel, isToday, weekdayLabel } from '$lib/domain/week';

	let {
		day,
		target,
		onAssign,
		onUnassign
	}: {
		day: PlannedDay;
		target: Macros | null;
		onAssign: (date: string) => void;
		onUnassign: (date: string) => void;
	} = $props();

	const today = $derived(isToday(day.date));
</script>

<Card.Root class={today ? 'border-primary/60' : undefined}>
	<Card.Header class="gap-1 pb-3">
		<div class="flex items-baseline justify-between">
			<Card.Title class="text-sm font-medium">{weekdayLabel(day.date)}</Card.Title>
			<span class="text-muted-foreground text-xs tabular-nums">{dayMonthLabel(day.date)}</span>
		</div>
	</Card.Header>
	<Card.Content class="space-y-3">
		{#if day.dayPlan && day.macros}
			<div>
				<a href="/day-plans/{day.dayPlan.id}" class="font-medium hover:underline">
					{day.dayPlan.label}
				</a>
				<p class="text-muted-foreground text-xs tabular-nums">{formatKcal(day.macros.energyKcal)}</p>
			</div>
			{#if target}
				<TargetProgress actual={day.macros} target={target} compact />
			{/if}
			<div class="flex gap-1">
				<Button variant="ghost" size="sm" onclick={() => onAssign(day.date)}>Replace</Button>
				<Button variant="ghost" size="sm" onclick={() => onUnassign(day.date)}>
					<XIcon class="size-4" />
					Clear
				</Button>
			</div>
		{:else if day.dayPlanId}
			<!-- Assigned to a plan that has since been deleted. The id is kept so the
			     day can still be cleared. -->
			<p class="text-muted-foreground text-sm italic">Plan removed</p>
			<div class="flex gap-1">
				<Button variant="ghost" size="sm" onclick={() => onAssign(day.date)}>Assign</Button>
				<Button variant="ghost" size="sm" onclick={() => onUnassign(day.date)}>
					<XIcon class="size-4" />
					Clear
				</Button>
			</div>
		{:else}
			<Button variant="ghost" size="sm" class="w-full justify-start" onclick={() => onAssign(day.date)}>
				<PlusIcon class="size-4" />
				Assign a plan
			</Button>
		{/if}
	</Card.Content>
</Card.Root>
