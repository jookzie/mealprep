<script lang="ts">
	import type { DayPlan } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';
	import { formatKcal } from '$lib/domain/format';
	import { byRecency } from '$lib/domain/sort';
	import {
		type IsoDate,
		datesMatchingWeekdays,
		periodWeekStarts,
		weekDates,
		weekLabel,
		weekRange,
		weekdayLabel
	} from '$lib/domain/week';

	/*
	 * Assigning one plan to Monday, Wednesday and Friday is otherwise five separate
	 * dialogs. Paprika's menus do the same thing — apply, choose where it starts, and
	 * the rest fill in — and it is a far better answer to repetitive assignment than
	 * drag-and-drop would be.
	 *
	 * There is no bulk endpoint, so this is a loop over PUT /calendar/{date} behind one
	 * confirmation. The dates it will touch are named before anything is written,
	 * because a day already holding a plan is replaced.
	 */
	let {
		open = $bindable(),
		weekStart,
		dayPlans,
		plannedDates,
		onApply
	}: {
		open: boolean;
		weekStart: IsoDate;
		dayPlans: readonly DayPlan[];
		/** Dates in this week that already hold a plan, so replacement can be spelled out. */
		plannedDates: readonly IsoDate[];
		onApply: (dayPlanId: string, dates: IsoDate[]) => Promise<void>;
	} = $props();

	const ordered = $derived(byRecency(dayPlans));
	// Weekday buttons are labelled from the first visible week, but the labels are the
	// same for every week in the block.
	const dates = $derived(weekDates(weekStart));
	const weekStarts = $derived(periodWeekStarts(weekStart));

	let dayPlanId = $state('');
	let weekdays = $state<string[]>([]);
	// Which of the visible weeks to touch: 'all', or one week's start date. The dialog
	// is opened from a four-week view, so applying to only the first would silently
	// ignore three quarters of what is on screen.
	let scope = $state<string>('all');
	let pending = $state(false);

	const selected = $derived(ordered.find((plan) => plan.id === dayPlanId));
	const scopeStarts = $derived(scope === 'all' ? weekStarts : [scope]);
	const targets = $derived(
		scopeStarts.flatMap((start) => {
			const { from, to } = weekRange(start);
			return datesMatchingWeekdays(from, to, weekdays.map(Number));
		})
	);
	const replacing = $derived(targets.filter((date) => plannedDates.includes(date)).length);
	const valid = $derived(dayPlanId !== '' && targets.length > 0);

	async function apply() {
		if (!valid) return;
		pending = true;
		await onApply(dayPlanId, targets);
		pending = false;
		open = false;
		weekdays = [];
		scope = 'all';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Apply a plan across the weeks</Dialog.Title>
			<Dialog.Description>
				Pick a plan and the days it belongs on. A day holds one plan, so any day you choose that
				already has one is replaced.
			</Dialog.Description>
		</Dialog.Header>

		{#if ordered.length === 0}
			<p class="text-muted-foreground text-sm">There are no day plans yet.</p>
		{:else}
			<div class="space-y-4">
				<div class="space-y-2">
					<Label for="apply-plan">Day plan</Label>
					<Select.Root type="single" bind:value={dayPlanId}>
						<Select.Trigger id="apply-plan" class="w-full">
							{selected ? selected.label : 'Choose a day plan…'}
						</Select.Trigger>
						<Select.Content>
							{#each ordered as plan (plan.id)}
								<Select.Item value={plan.id} label={plan.label}>
									<span class="flex-1 truncate">{plan.label}</span>
									<span class="text-muted-foreground ml-3 text-xs tabular-nums">
										{formatKcal(plan.macros.energyKcal)}
									</span>
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="apply-scope">Weeks</Label>
					<Select.Root type="single" bind:value={scope}>
						<Select.Trigger id="apply-scope" class="w-full">
							{scope === 'all'
								? `All ${weekStarts.length} weeks`
								: weekLabel(scope)}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="all" label="All {weekStarts.length} weeks">
								All {weekStarts.length} weeks
							</Select.Item>
							{#each weekStarts as start (start)}
								<Select.Item value={start} label={weekLabel(start)}>{weekLabel(start)}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label>Days</Label>
					<ToggleGroup.Root type="multiple" variant="outline" bind:value={weekdays} class="w-full">
						{#each dates as date, index (date)}
							<ToggleGroup.Item value={String(index)} class="flex-1">
								{weekdayLabel(date)}
							</ToggleGroup.Item>
						{/each}
					</ToggleGroup.Root>
				</div>

				<p class="text-muted-foreground text-sm" aria-live="polite">
					{#if targets.length === 0}
						Choose at least one day.
					{:else}
						{targets.length}
						{targets.length === 1 ? 'day' : 'days'} will be assigned{replacing > 0
							? `, replacing ${replacing} that already ${replacing === 1 ? 'has' : 'have'} a plan`
							: ''}.
					{/if}
				</p>
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (open = false)} disabled={pending}>Cancel</Button>
			<Button onclick={apply} disabled={!valid || pending}>
				{pending ? 'Applying…' : 'Apply plan'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
