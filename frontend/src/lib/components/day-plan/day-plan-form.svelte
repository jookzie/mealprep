<script lang="ts">
	import { untrack } from 'svelte';
	import type { DayPlanDraft, Meal } from '$lib/api';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import MealPicker from '$lib/components/meal/meal-picker.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';
	import { resolveMeals } from '$lib/domain/day-plan';
	import { sumMacros } from '$lib/domain/macros';
	import DayPlanMealList from './day-plan-meal-list.svelte';

	let {
		initial,
		meals,
		submitLabel,
		onSubmit
	}: {
		initial: DayPlanDraft;
		meals: Meal[];
		submitLabel: string;
		onSubmit: (draft: DayPlanDraft) => Promise<void>;
	} = $props();

	// Seeded once; the page re-seeds by keying this component on the entity.
	const seed = untrack(() => initial);
	let label = $state(seed.label);
	let mealIds = $state<string[]>([...seed.mealIds]);
	let pending = $state(false);

	// Preview only. A saved plan carries the API's own figure.
	const preview = $derived(
		sumMacros(
			resolveMeals(mealIds, meals)
				.filter((meal) => meal !== undefined)
				.map((meal) => meal.macros)
		)
	);

	const valid = $derived(label.trim() !== '' && mealIds.length > 0);

	// A day plan may legitimately hold the same meal twice, so this adds rather than
	// toggling off an existing entry.
	function add(mealId: string) {
		mealIds = [...mealIds, mealId];
	}

	function remove(index: number) {
		mealIds = mealIds.filter((_, i) => i !== index);
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!valid) return;
		pending = true;
		await onSubmit({ label: label.trim(), mealIds });
		pending = false;
	}
</script>

<form class="space-y-6" onsubmit={submit}>
	<div class="space-y-2">
		<Label for="label">Label</Label>
		<Input id="label" bind:value={label} placeholder="Training day" required />
	</div>

	<Separator />

	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<Label>Meals</Label>
			<MealPicker {meals} selected={mealIds} onToggle={add} />
		</div>

		{#if meals.length === 0}
			<p class="text-muted-foreground text-sm">
				There are no meals yet. <a href="/meals" class="underline underline-offset-4">
					Create one first</a
				>.
			</p>
		{:else if mealIds.length === 0}
			<p class="text-muted-foreground text-sm">A day plan is a group of meals. Add the first one.</p>
		{:else}
			<DayPlanMealList {mealIds} {meals} onRemove={remove} />
		{/if}
	</div>

	{#if mealIds.length > 0}
		<div class="bg-muted/40 rounded-lg p-4">
			<p class="text-muted-foreground mb-2 text-xs">Draft total</p>
			<MacrosSummary macros={preview} />
		</div>
	{/if}

	<Button type="submit" disabled={pending || !valid}>
		{pending ? 'Saving…' : submitLabel}
	</Button>
</form>
