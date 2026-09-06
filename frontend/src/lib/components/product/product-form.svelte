<script lang="ts">
	import { untrack } from 'svelte';
	import type { ProductDraft, Unit } from '$lib/api';
	import NumberField from '$lib/components/app/number-field.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';
	import {
		duplicateKeys,
		fromPairs,
		type NutrientPair, 
		toPairs
	} from '$lib/domain/nutrient-pairs';
	import NutrientFields from './nutrient-fields.svelte';

	let {
		initial,
		submitLabel,
		onSubmit
	}: {
		initial: ProductDraft;
		submitLabel: string;
		onSubmit: (draft: ProductDraft) => Promise<void>;
	} = $props();

	// Seeded once; the page re-seeds by keying this component on the entity.
	const seed = untrack(() => initial);
	let name = $state(seed.name);
	let unit = $state<Unit>(seed.unit);
	let energyKcal = $state<number | null>(seed.macros.energyKcal);
	let proteinG = $state<number | null>(seed.macros.proteinG);
	let fatG = $state<number | null>(seed.macros.fatG);
	let carbohydratesG = $state<number | null>(seed.macros.carbohydratesG);
	let pairs = $state<NutrientPair[]>(toPairs(seed.nutrients));
	let pending = $state(false);

	// The four macros are required; every other nutrient is optional (PR-7).
	const valid = $derived(
		name.trim() !== '' &&
			energyKcal !== null &&
			proteinG !== null &&
			fatG !== null &&
			carbohydratesG !== null &&
			duplicateKeys(pairs).size === 0
	);

	const unitLabel = $derived(unit === 'ml' ? 'millilitres' : 'grams');

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!valid) return;
		pending = true;
		await onSubmit({
			name: name.trim(),
			unit,
			macros: {
				energyKcal: energyKcal as number,
				proteinG: proteinG as number,
				fatG: fatG as number,
				carbohydratesG: carbohydratesG as number
			},
			nutrients: fromPairs(pairs)
		});
		pending = false;
	}
</script>

<form class="space-y-6" onsubmit={submit}>
	<div class="grid gap-4 sm:grid-cols-[1fr_10rem]">
		<div class="space-y-2">
			<Label for="name">Name</Label>
			<Input id="name" bind:value={name} placeholder="Rolled oats" required />
		</div>
		<div class="space-y-2">
			<Label for="unit">Measured in</Label>
			<Select.Root type="single" bind:value={unit}>
				<Select.Trigger id="unit" class="w-full">
					{unit === 'ml' ? 'Millilitres' : 'Grams'}
				</Select.Trigger>
				<Select.Content>
					<Select.Item value="g">Grams</Select.Item>
					<Select.Item value="ml">Millilitres</Select.Item>
				</Select.Content>
			</Select.Root>
		</div>
	</div>

	<Separator />

	<div class="space-y-3">
		<div>
			<Label>Macros <span class="text-muted-foreground">per 100 {unitLabel}</span></Label>
			<p class="text-muted-foreground mt-1 text-sm">All four are required.</p>
		</div>
		<div class="grid gap-4 sm:grid-cols-2">
			<NumberField id="energyKcal" label="Energy" suffix="kcal" bind:value={energyKcal} required />
			<NumberField id="proteinG" label="Protein" suffix="g" bind:value={proteinG} required />
			<NumberField id="fatG" label="Fat" suffix="g" bind:value={fatG} required />
			<NumberField
				id="carbohydratesG"
				label="Carbs"
				suffix="g"
				bind:value={carbohydratesG}
				required
			/>
		</div>
	</div>

	<Separator />

	<NutrientFields bind:pairs unit={unitLabel} />

	<Button type="submit" disabled={pending || !valid}>
		{pending ? 'Saving…' : submitLabel}
	</Button>
</form>
