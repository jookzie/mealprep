<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import type { Meal } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import { formatKcal } from '$lib/domain/format';

	let {
		meals,
		selected,
		onToggle
	}: { meals: readonly Meal[]; selected: readonly string[]; onToggle: (mealId: string) => void } =
		$props();

	let open = $state(false);
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
					{#each meals as meal (meal.id)}
						<Command.Item value={meal.label} onSelect={() => onToggle(meal.id)}>
							<CheckIcon
								class="size-4 {selected.includes(meal.id) ? 'opacity-100' : 'opacity-0'}"
							/>
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
