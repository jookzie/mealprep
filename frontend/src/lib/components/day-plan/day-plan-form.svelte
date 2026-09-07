<script lang="ts">
	import { untrack } from 'svelte';
	import type { DayPlanDraft, Macros, Meal } from '$lib/api';
	import CalculatedTotals from '$lib/components/macros/calculated-totals.svelte';
	import MealPicker from '$lib/components/meal/meal-picker.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { sumMacros } from '$lib/domain/macros';
	import { move, moveAnnouncement } from '$lib/domain/order';
	import DayPlanMealList from './day-plan-meal-list.svelte';

	let {
		initial,
		meals,
		targets,
		submitLabel,
		onSubmit
	}: {
		initial: DayPlanDraft;
		meals: Meal[];
		targets: Macros | null;
		submitLabel: string;
		onSubmit: (draft: DayPlanDraft) => Promise<void>;
	} = $props();

	// The same meal may appear twice in a plan, so a row needs an identity of its own —
	// the meal id does not distinguish the two entries, and the index cannot be the key
	// once rows reorder.
	type Row = { id: string; mealId: string };

	function row(mealId: string): Row {
		return { id: crypto.randomUUID(), mealId };
	}

	// Seeded once; the page re-seeds by keying this component on the entity.
	const seed = untrack(() => initial);
	let label = $state(seed.label);
	let rows = $state<Row[]>(seed.mealIds.map(row));
	let pending = $state(false);
	let announcement = $state('');

	const resolved = $derived(
		rows.map((entry) => ({
			...entry,
			meal: meals.find((meal) => meal.id === entry.mealId)
		}))
	);

	// Preview only. A saved plan carries the API's own figure. A row whose meal has
	// been deleted contributes nothing rather than a zero it cannot vouch for.
	const preview = $derived(
		sumMacros(
			resolved
				.map((entry) => entry.meal?.macros)
				.filter((macros) => macros !== undefined)
		)
	);

	const valid = $derived(label.trim() !== '' && rows.length > 0);

	function add(mealId: string) {
		rows = [...rows, row(mealId)];
	}

	function remove(id: string) {
		rows = rows.filter((entry) => entry.id !== id);
	}

	function moveRow(from: number, to: number) {
		const name = resolved[from].meal?.label ?? 'Meal';
		rows = move(rows, from, to);
		announcement = moveAnnouncement(name, to, rows.length);
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!valid) return;
		pending = true;
		await onSubmit({ label: label.trim(), mealIds: rows.map((entry) => entry.mealId) });
		pending = false;
	}
</script>

<form onsubmit={submit}>
	<div class="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_20rem]">
		<div class="space-y-6">
			<div class="space-y-2">
				<Label for="label">Label</Label>
				<Input id="label" bind:value={label} placeholder="Training day" required />
			</div>

			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<Label>Meals</Label>
					<MealPicker {meals} onChoose={add} />
				</div>

				{#if meals.length === 0}
					<p class="text-muted-foreground text-sm">
						There are no meals yet. <a href="/meals" class="underline underline-offset-4">
							Create one first</a
						>.
					</p>
				{:else if rows.length === 0}
					<p class="text-muted-foreground text-sm">
						A day plan is a group of meals in the order you eat them. Add the first one.
					</p>
				{:else}
					<DayPlanMealList rows={resolved} onRemove={remove} onMove={moveRow} />
				{/if}

				<p class="sr-only" aria-live="polite">{announcement}</p>
			</div>

			<Button type="submit" disabled={pending || !valid}>
				{pending ? 'Saving…' : submitLabel}
			</Button>
		</div>

		<aside class="lg:sticky lg:top-6">
			<CalculatedTotals macros={preview} target={targets} empty={rows.length === 0} />
		</aside>
	</div>
</form>
