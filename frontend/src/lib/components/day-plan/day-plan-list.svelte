<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { type DayPlan, deleteDayPlan, type Macros } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import MacroRadar from '$lib/components/macros/macro-radar.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { formatGrams, formatKcal, formatSigned } from '$lib/domain/format';
	import { MACRO_KEYS, MACRO_LABELS } from '$lib/domain/macros';
	import { MACRO_HUE } from '$lib/components/macros/hues';

	/*
	 * A plan's identity is its four figures and the shape they make, so both are on the
	 * card: the numbers to read and the radar to compare against the other plans at a
	 * glance. Every axis shares one denominator across the whole list, which is what
	 * makes the shapes comparable to each other and not only to themselves.
	 */
	let {
		dayPlans,
		denominator,
		relativeTo
	}: { dayPlans: DayPlan[]; denominator: Macros; relativeTo: 'target' | 'largest' } = $props();

	function value(macros: Macros, key: (typeof MACRO_KEYS)[number]): string {
		return key === 'energyKcal' ? formatKcal(macros[key]) : formatGrams(macros[key]);
	}
</script>

<!-- Two across at most: the figures and the radar sit side by side, and squeezing a
     third column makes the deltas collide with the chart. -->
<div class="grid gap-4 xl:grid-cols-2">
	{#each dayPlans as plan (plan.id)}
		<Card.Root>
			<Card.Header>
				<Card.Title>
					<a href="/day-plans/{plan.id}" class="hover:underline">{plan.label}</a>
				</Card.Title>
				<Card.Description>
					{#if plan.meals.length === 0}
						No meals
					{:else}
						{plan.meals.map((meal) => meal.label).join(' · ')}
					{/if}
				</Card.Description>
				<Card.Action>
					<div class="flex items-center gap-1">
						<Button href="/day-plans/{plan.id}/edit" variant="ghost" size="icon" title="Edit">
							<PencilIcon class="size-4" />
						</Button>
						<DeleteAction
							name={plan.label}
							size="icon"
							description="The plan is removed. Calendar days holding it will show it as removed."
							action={() => deleteDayPlan({ path: { dayPlanId: plan.id }, throwOnError: true })}
						/>
					</div>
				</Card.Action>
			</Card.Header>
			<Card.Content class="flex items-center justify-between gap-4">
				<dl class="min-w-0 flex-1 space-y-1.5">
					{#each MACRO_KEYS as key (key)}
						<div class="flex items-baseline justify-between gap-3 text-sm">
							<dt class="text-muted-foreground flex items-center gap-1.5 whitespace-nowrap">
								<span
									class="size-1.5 rounded-full"
									style:background-color={MACRO_HUE[key]}
									aria-hidden="true"
								></span>
								{MACRO_LABELS[key]}
							</dt>
							<dd class="flex items-baseline gap-2 whitespace-nowrap">
								<span class="font-medium tabular-nums">{value(plan.macros, key)}</span>
								{#if relativeTo === 'target'}
									<!-- Against the daily target, shown and never acted on (TG-3). -->
									<span class="text-muted-foreground text-xs tabular-nums">
										{formatSigned(
											plan.macros[key] - denominator[key],
											key === 'energyKcal' ? 'kcal' : 'g'
										)}
									</span>
								{/if}
							</dd>
						</div>
					{/each}
				</dl>
				<div class="shrink-0">
					<MacroRadar macros={plan.macros} {denominator} {relativeTo} size={124} />
				</div>
			</Card.Content>
		</Card.Root>
	{/each}
</div>
