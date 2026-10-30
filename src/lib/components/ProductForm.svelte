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
	import X from '@lucide/svelte/icons/x';
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

<form class="stack editor" onsubmit={submit}>
	{#if restored}
		<DraftNotice onStartOver={startOver} />
	{/if}

	<label class="field">
		<span>Name</span>
		<input class="title-input" bind:value={form.name} required />
	</label>
	<div class="row identity">
		<label class="field grow">
			<span>Brand (optional)</span>
			<input bind:value={form.brand} />
		</label>
		<div class="field">
			<span id="unit-label">Per 100</span>
			<div class="segmented unit" role="radiogroup" aria-labelledby="unit-label">
				<button type="button" role="radio" aria-checked={form.unit === 'g'} onclick={() => (form.unit = 'g')}
					>g</button
				>
				<button type="button" role="radio" aria-checked={form.unit === 'ml'} onclick={() => (form.unit = 'ml')}
					>ml</button
				>
			</div>
		</div>
	</div>

	<h2>Per 100 {form.unit}</h2>
	<div class="grid-2">
		<NumberField label="Energy" suffix="kcal" tone="hero" bind:value={form.energyKcal} required />
		<NumberField label="Protein" suffix="g" tone="protein" bind:value={form.proteinG} required />
		<NumberField label="Fat" suffix="g" tone="fat" bind:value={form.fatG} required />
		<NumberField label="Carbs" suffix="g" tone="carbs" bind:value={form.carbohydratesG} required />
	</div>

	{#if doubts.length > 0}
		<section class="notice stack doubts" aria-live="polite">
			<strong>Worth a look at the package</strong>
			<ul class="small">
				{#each doubts as doubt (doubt)}
					<li>{doubt}</li>
				{/each}
			</ul>
			<span class="small">You can still save.</span>
		</section>
	{/if}

	<section class="card stack">
		<div class="row spread">
			<h2>More nutrients</h2>
			<button type="button" class="outline compact" onclick={() => form.pairs.push(newPair())}>+ Add</button>
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
					class="ghost icon"
					aria-label="Remove nutrient"
					onclick={() => form.pairs.splice(index, 1)}><X size={18} strokeWidth={2.6} aria-hidden="true" /></button
				>
			</div>
		{:else}
			<p class="muted small">Fibre, sugars, salt, vitamins — whatever the label lists.</p>
		{/each}
		{#if duplicates.size > 0}
			<p class="error small">Each nutrient can appear only once.</p>
		{/if}
	</section>

	<div class="save-bar">
		<div class="grow totals">
			<NumberField label="Price per 100 {form.unit}" suffix="optional" bind:value={form.cost} />
		</div>
		<button class="confirm" type="submit" disabled={!complete || busy}>{submitLabel}</button>
	</div>
</form>

<style>
	.editor {
		gap: 1rem;
	}

	.identity {
		flex-wrap: nowrap;
		align-items: flex-end;
	}

	.unit {
		width: 7rem;
	}

	.doubts ul {
		margin: 0;
		padding-left: 1.1rem;
		list-style: disc;
	}

	.nutrient {
		flex-wrap: nowrap;
	}

	.amount {
		width: 6.5rem;
		flex: none;
	}

	/* The price sits in the save bar, on the ink: it is the one optional figure left. */
	.save-bar :global(.field) {
		gap: 0;
	}

	.save-bar :global(.field > span) {
		font-size: 0.75rem;
		opacity: 0.75;
	}

	.save-bar :global(input) {
		min-height: 0;
		padding: 0;
		border: none;
		background: transparent;
		color: var(--hero-text);
		font-family: var(--font-display);
		font-size: 1.35rem;
		font-weight: 800;
	}
</style>
