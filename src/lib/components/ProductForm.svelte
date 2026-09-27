<script lang="ts">
	import { untrack } from 'svelte';
	import type { ProductDraft, Unit } from '$lib/api';
	import type { ProductSeed } from '$lib/domain/catalogue';
	import {
		duplicateKeys,
		fromPairs,
		type NutrientPair,
		newPair,
		toPairs,
	} from '$lib/domain/nutrient-pairs';
	import { implausibilities } from '$lib/domain/plausibility';
	import NumberField from './NumberField.svelte';

	let {
		initial,
		submitLabel,
		onSubmit,
	}: {
		initial?: ProductSeed;
		submitLabel: string;
		onSubmit: (draft: ProductDraft) => Promise<unknown>;
	} = $props();

	// Seeded once from the props: a refresh mid-edit must not overwrite what is being typed.
	const seed = untrack(() => initial);
	let name = $state(seed?.name ?? '');
	let brand = $state(seed?.brand ?? '');
	let unit = $state<Unit>(seed?.unit ?? 'g');
	let energyKcal = $state<number | null>(seed?.macros.energyKcal ?? null);
	let proteinG = $state<number | null>(seed?.macros.proteinG ?? null);
	let fatG = $state<number | null>(seed?.macros.fatG ?? null);
	let carbohydratesG = $state<number | null>(seed?.macros.carbohydratesG ?? null);
	let cost = $state<number | null>(seed?.cost ?? null);
	let pairs = $state<NutrientPair[]>(toPairs(seed?.nutrients));
	let busy = $state(false);

	const duplicates = $derived(duplicateKeys(pairs));
	// Figures that cannot all be right, re-checked as the user corrects them.
	const doubts = $derived(
		implausibilities({
			unit,
			macros: { energyKcal, fatG, proteinG, carbohydratesG },
			nutrients: fromPairs(pairs),
		}),
	);
	const complete = $derived(
		name.trim() !== '' &&
			energyKcal !== null &&
			proteinG !== null &&
			fatG !== null &&
			carbohydratesG !== null &&
			duplicates.size === 0,
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (energyKcal === null || proteinG === null || fatG === null || carbohydratesG === null) {
			return;
		}
		busy = true;
		try {
			await onSubmit({
				name,
				brand: brand.trim() === '' ? undefined : brand,
				unit,
				macros: { energyKcal, fatG, proteinG, carbohydratesG },
				nutrients: fromPairs(pairs),
				cost: cost ?? undefined,
			});
		} finally {
			busy = false;
		}
	}
</script>

<form class="stack" onsubmit={submit}>
	<section class="card stack">
		<label class="field">
			<span>Name</span>
			<input bind:value={name} required />
		</label>
		<label class="field">
			<span>Brand (optional)</span>
			<input bind:value={brand} />
		</label>
		<label class="field">
			<span>Figures are per 100</span>
			<select bind:value={unit}>
				<option value="g">grams</option>
				<option value="ml">millilitres</option>
			</select>
		</label>
	</section>

	<section class="card stack">
		<h2>Per 100 {unit}</h2>
		<div class="grid-2">
			<NumberField label="Energy" suffix="kcal" bind:value={energyKcal} required />
			<NumberField label="Protein" suffix="g" bind:value={proteinG} required />
			<NumberField label="Fat" suffix="g" bind:value={fatG} required />
			<NumberField label="Carbs" suffix="g" bind:value={carbohydratesG} required />
		</div>
		<NumberField label="Cost per 100 {unit}" suffix="optional" bind:value={cost} />
	</section>

	<section class="card stack">
		<div class="row spread">
			<h2>Other nutrients</h2>
			<button type="button" onclick={() => pairs.push(newPair())}>Add</button>
		</div>
		{#each pairs as pair, index (pair.id)}
			<div class="row nutrient">
				<input
					class="grow"
					placeholder="Name, e.g. fiber"
					aria-label="Nutrient name"
					aria-invalid={duplicates.has(pair.key.trim().toLowerCase())}
					bind:value={pair.key}
				/>
				<input
					class="amount"
					inputmode="decimal"
					placeholder="per 100"
					aria-label="Amount per 100 {unit}"
					bind:value={pair.value}
				/>
				<button
					type="button"
					class="ghost icon danger"
					aria-label="Remove nutrient"
					onclick={() => pairs.splice(index, 1)}>✕</button
				>
			</div>
		{/each}
		{#if duplicates.size > 0}
			<p class="error small">Each nutrient can appear only once.</p>
		{/if}
	</section>

	{#if doubts.length > 0}
		<section class="card stack doubts" aria-live="polite">
			<h2>Check against the package</h2>
			<ul class="small">
				{#each doubts as doubt (doubt)}
					<li>{doubt}</li>
				{/each}
			</ul>
		</section>
	{/if}

	<div class="row end">
		<button class="primary" type="submit" disabled={!complete || busy}>{submitLabel}</button>
	</div>
</form>

<style>
	.doubts {
		border-color: var(--warning);
	}

	.doubts h2 {
		color: var(--warning);
	}

	.doubts ul {
		margin: 0;
		padding-left: 1.1rem;
	}

	.nutrient {
		flex-wrap: nowrap;
	}

	.amount {
		width: 6.5rem;
		flex: none;
	}
</style>
