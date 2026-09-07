<script lang="ts">
	import type { DayPlan } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import * as Dialog from '$lib/components/ui/dialog';
	import { formatKcal } from '$lib/domain/format';
	import { byRecency } from '$lib/domain/sort';
	import { fullDateLabel } from '$lib/domain/week';

	let {
		open = $bindable(),
		date,
		dayPlans,
		returnTo,
		onChoose
	}: {
		open: boolean;
		date: string | null;
		dayPlans: readonly DayPlan[];
		returnTo: string;
		onChoose: (dayPlanId: string) => void;
	} = $props();

	// Most recently touched first. Alphabetical puts the plan you built a minute ago
	// wherever its label happens to fall, which is never where you are looking.
	const ordered = $derived(byRecency(dayPlans));
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0">
		<Dialog.Header class="px-4 pt-4">
			<Dialog.Title>Assign a day plan</Dialog.Title>
			<Dialog.Description>
				{date ? fullDateLabel(date) : ''} · a day holds one plan, so this replaces anything already
				there.
			</Dialog.Description>
		</Dialog.Header>
		{#if dayPlans.length === 0}
			<div class="space-y-3 px-4 pb-4">
				<p class="text-muted-foreground text-sm">There are no day plans yet.</p>
				<Button href="/day-plans/new?returnTo={encodeURIComponent(returnTo)}">
					Create a day plan
				</Button>
			</div>
		{:else}
			<Command.Root class="rounded-t-none border-t">
				<Command.Input placeholder="Search day plans…" />
				<Command.List>
					<Command.Empty>No day plan found.</Command.Empty>
					<Command.Group>
						{#each ordered as plan (plan.id)}
							<Command.Item value={plan.label} onSelect={() => onChoose(plan.id)}>
								<span class="flex-1 truncate">{plan.label}</span>
								<span class="text-muted-foreground text-xs tabular-nums">
									{formatKcal(plan.macros.energyKcal)}
								</span>
							</Command.Item>
						{/each}
					</Command.Group>
				</Command.List>
			</Command.Root>
		{/if}
	</Dialog.Content>
</Dialog.Root>
