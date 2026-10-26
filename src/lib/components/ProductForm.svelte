<script lang="ts" module>
	import type { Unit } from '$lib/api';
	import type { NutrientPair } from '$lib/domain/nutrient-pairs';

	export type ProductFormState = {
		name: string;
		brand: string;
		unit: Unit;
		energyKcal: number | null;
		proteinG: number | null;
		fatG: number | null;
		carbohydratesG: number | null;
		cost: number | null;
		pairs: NutrientPair[];
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import type { ProductDraft } from '$lib/api';
	import type { ProductSeed } from '$lib/domain/catalogue';
	import {
		duplicateKeys,
		fromPairs,
		newPair,
		toPairs,
	} from '$lib/domain/nutrient-pairs';
	import { implausibilities } from '$lib/domain/plausibility';
	import type { Kept } from '$lib/memory.svelte';
	import DraftNotice from './DraftNotice.svelte';
	import NumberField from './NumberField.svelte';

	let {
		initial,
		draft,
		submitLabel,
		onSubmit,
	}: {
		initial?: ProductSeed;
		/** Where the form keeps what is typed, so it outlasts the screen. */
		draft: Kept<ProductFormState | null>;
		submitLabel: string;
		onSubmit: (draft: ProductDraft) => Promise<unknown>;
	} = $props();

	function fresh(): ProductFormState {
		const seed = untrack(() => initial);
		return {
			name: seed?.name ?? '',
			brand: seed?.brand ?? '',
			unit: seed?.unit ?? 'g',
			energyKcal: seed?.macros.energyKcal ?? null,
			proteinG: seed?.macros.proteinG ?? null,
			fatG: seed?.macros.fatG ?? null,
			carbohydratesG: seed?.macros.carbohydratesG ?? null,
			cost: seed?.cost ?? null,
			pairs: toPairs(seed?.nutrients),
		};
	}

	/** What the user typed, without the row ids that differ between two seedings. */
	function typed(state: ProductFormState): string {
		return JSON.stringify({ ...state, pairs: state.pairs.map((pair) => [pair.key, pair.value]) });
	}

	// Seeded once: from the kept draft when there is one, from the props otherwise. A
	// refresh mid-edit must not overwrite what is being typed.
	const box = untrack(() => draft);
	let restored = $state(box.value !== null && typed(box.value) !== typed(fresh()));
	box.value ??= fresh();
	const form = box.value;
	let busy = $state(false);

	function startOver() {
		Object.assign(form, fresh());
		restored = false;
	}

	const duplicates = $derived(duplicateKeys(form.pairs));
	// Figures that cannot all be right, re-checked as the user corrects them.
	const doubts = $derived(
		implausibilities({
			unit: form.unit,
			macros: {
				energyKcal: form.energyKcal,
				fatG: form.fatG,
				proteinG: form.proteinG,
				carbohydratesG: form.carbohydratesG,
			},
			nutrients: fromPairs(form.pairs),
		}),
	);
	const complete = $derived(
		form.name.trim() !== '' &&
			form.energyKcal !== null &&
			form.proteinG !== null &&
			form.fatG !== null &&
			form.carbohydratesG !== null &&
			duplicates.size === 0,
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const { energyKcal, proteinG, fatG, carbohydratesG } = form;
		if (energyKcal === null || proteinG === null || fatG === null || carbohydratesG === null) {
			return;
		}
		busy = true;
		try {
			await onSubmit({
				name: form.name,
				brand: form.brand.trim() === '' ? undefined : form.brand,
				unit: form.unit,
				macros: { energyKcal, fatG, proteinG, carbohydratesG },
				nutrients: fromPairs(form.pairs),
				cost: form.cost ?? undefined,
			});
		} finally {
			busy = false;
		}
	}
</script>

<form class="stack" onsubmit={submit}>
	{#if restored}
		<DraftNotice onStartOver={startOver} />
	{/if}

	<section class="card stack">
		<label class="field">
			<span>Name</span>
			<input bind:value={form.name} required />
		</label>
		<label class="field">
			<span>Brand (optional)</span>
			<input bind:value={form.brand} />
		</label>
		<label class="field">
			<span>Figures are per 100</span>
			<select bind:value={form.unit}>
				<option value="g">grams</option>
				<option value="ml">millilitres</option>
			</select>
		</label>
	</section>

	<section class="card stack">
		<h2>Per 100 {form.unit}</h2>
		<div class="grid-2">
			<NumberField label="Energy" suffix="kcal" bind:value={form.energyKcal} required />
			<NumberField label="Protein" suffix="g" bind:value={form.proteinG} required />
			<NumberField label="Fat" suffix="g" bind:value={form.fatG} required />
			<NumberField label="Carbs" suffix="g" bind:value={form.carbohydratesG} required />
		</div>
		<NumberField label="Cost per 100 {form.unit}" suffix="optional" bind:value={form.cost} />
	</section>

	<section class="card stack">
		<div class="row spread">
			<h2>Other nutrients</h2>
			<button type="button" onclick={() => form.pairs.push(newPair())}>Add</button>
		</div>
		{#each form.pairs as pair, index (pair.id)}
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
					aria-label="Amount per 100 {form.unit}"
					bind:value={pair.value}
				/>
				<button
					type="button"
					class="ghost icon danger"
					aria-label="Remove nutrient"
					onclick={() => form.pairs.splice(index, 1)}>✕</button
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
