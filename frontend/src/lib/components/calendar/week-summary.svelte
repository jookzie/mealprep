<script lang="ts">
	import MacrosDelta from '$lib/components/macros/macros-delta.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import TargetProgress from '$lib/components/macros/target-progress.svelte';
	import * as Card from '$lib/components/ui/card';
	import type { PlannedWeek } from '$lib/domain/calendar';

	let { week }: { week: PlannedWeek } = $props();
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Week total</Card.Title>
		<Card.Description>
			{week.plannedCount} of 7 days planned.
			{#if week.targetTotal}
				Compared against the daily target for those {week.plannedCount}
				{week.plannedCount === 1 ? 'day' : 'days'}.
			{/if}
		</Card.Description>
	</Card.Header>
	<Card.Content class="space-y-6">
		<MacrosSummary macros={week.total} variant="grid" />
		{#if week.targetTotal}
			<div class="space-y-4">
				<TargetProgress actual={week.total} target={week.targetTotal} />
				<MacrosDelta actual={week.total} target={week.targetTotal} />
			</div>
		{:else}
			<p class="text-muted-foreground text-sm">
				<a href="/targets" class="underline underline-offset-4">Set your daily targets</a>
				to see the week measured against them.
			</p>
		{/if}
	</Card.Content>
</Card.Root>
