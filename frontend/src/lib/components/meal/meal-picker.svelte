<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import type { Meal } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import { formatKcal } from '$lib/domain/format';
	import { byRecency } from '$lib/domain/sort';

	let {
		meals,
		open = $bindable(false),
		onChoose
	}: { meals: readonly Meal[]; open?: boolean; onChoose: (mealId: string) => void } = $props();

	// A day plan may legitimately hold the same meal twice, so there is nothing to tick
	// off here: choosing adds, every time.
	const ordered = $derived(byRecency(meals));

	function choose(mealId: string) {
		onChoose(mealId);
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="outline" size="sm">
				<PlusIcon class="size-4" />
				Add meal
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-80 p-0" align="start">
		<Command.Root>
			<Command.Input placeholder="Search meals…" />
			<Command.List>
				<Command.Empty>No meal found.</Command.Empty>
				<Command.Group>
					{#each ordered as meal (meal.id)}
						<Command.Item value={meal.label} onSelect={() => choose(meal.id)}>
							<span class="flex-1 truncate">{meal.label}</span>
							<span class="text-muted-foreground text-xs tabular-nums">
								{formatKcal(meal.macros.energyKcal)}
							</span>
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
