<script lang="ts" module>
	export type MealFormState = {
		label: string;
		categoryId: string | undefined;
		rows: { key: number; productId: string; amount: number | null }[];
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import type { Category, MealDraft, Product } from '$lib/api';
	import { draftCost, draftMacros, servingRows } from '$lib/domain/meal';
	import { move } from '$lib/domain/order';
	import type { Kept } from '$lib/memory.svelte';
	import CategorySelect from './CategorySelect.svelte';
	import CostFigure from './CostFigure.svelte';
	import Dialog from './Dialog.svelte';
	import DraftNotice from './DraftNotice.svelte';
	import MacroLine from './MacroLine.svelte';
	import MoveButtons from './MoveButtons.svelte';
	import NumberField from './NumberField.svelte';
	import ProductPicker from './ProductPicker.svelte';

	let {
		initial,
		draft,
		products,
		categories,
		submitLabel,
		onSubmit,
	}: {
		initial: MealDraft;
		/** Where the form keeps what is typed, so it outlasts the screen. */
		draft: Kept<MealFormState | null>;
		products: readonly Product[];
		categories: readonly Category[];
		submitLabel: string;
		onSubmit: (draft: MealDraft) => Promise<unknown>;
	} = $props();

	function fresh(): MealFormState {
		const seed = untrack(() => initial);
		return {
			label: seed.label,
			categoryId: seed.categoryId,
			rows: seed.servings.map((serving, key) => ({
				key,
				productId: serving.productId,
				amount: serving.amount,
			})),
		};
	}

	function toDraft(state: MealFormState): MealDraft {
		return {
			label: state.label,
			categoryId: state.categoryId,
			servings: state.rows.map((row) => ({ productId: row.productId, amount: row.amount ?? 0 })),
		};
	}

	// Seeded once: from the kept draft when there is one, from the props otherwise. A
	// refresh mid-edit must not overwrite what is being typed.
	const box = untrack(() => draft);
	let restored = $state(
		box.value !== null && JSON.stringify(toDraft(box.value)) !== JSON.stringify(toDraft(fresh())),
	);
	box.value ??= fresh();
	const form = box.value;
	let nextKey = Math.max(-1, ...form.rows.map((row) => row.key)) + 1;
	let picking = $state(false);
	let busy = $state(false);

	const productsById = $derived(new Map(products.map((product) => [product.id, product])));
	const preview = $derived(
		servingRows(
			form.rows.map((row) => ({ productId: row.productId, amount: row.amount ?? 0 })),
			products,
		),
	);
	const valid = $derived(
		form.label.trim() !== '' && form.rows.every((row) => row.amount !== null && row.amount > 0),
	);

	function add(product: Product) {
		form.rows.push({ key: nextKey++, productId: product.id, amount: 100 });
		picking = false;
	}

	function startOver() {
		Object.assign(form, fresh());
		nextKey = form.rows.length;
		restored = false;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		try {
			await onSubmit(toDraft(form));
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
			<span>Label</span>
			<input bind:value={form.label} required />
		</label>
		<CategorySelect {categories} bind:value={form.categoryId} />
	</section>

	<section class="card stack">
		<div class="row spread">
			<h2>Servings</h2>
			<button type="button" onclick={() => (picking = true)}>Add product</button>
		</div>
		{#if form.rows.length === 0}
			<p class="empty">No products yet.</p>
		{:else}
			<ul class="list">
				{#each form.rows as row, index (row.key)}
					{@const product = productsById.get(row.productId)}
					<li class="stack entry">
						<div class="row spread">
							<strong class="grow truncate" class:muted={!product}
								>{product?.name ?? 'Product removed'}</strong
							>
							<MoveButtons
								{index}
								length={form.rows.length}
								name={product?.name ?? 'serving'}
								onMove={(from, to) => (form.rows = move(form.rows, from, to))}
								onRemove={() => form.rows.splice(index, 1)}
							/>
						</div>
						<NumberField label="Amount" suffix={product?.unit ?? 'g'} bind:value={row.amount} required />
						{#if preview[index]}
							<MacroLine macros={preview[index].macros} />
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="card stack" aria-label="Calculated totals">
		<h2>Calculated</h2>
		<MacroLine macros={draftMacros(preview)} />
		<CostFigure cost={draftCost(preview)} label="Cost" />
	</section>

	<div class="row end">
		<button class="primary" type="submit" disabled={!valid || busy}>{submitLabel}</button>
	</div>
</form>

<Dialog bind:open={picking} title="Add a product">
	<ProductPicker {products} onPick={add} />
</Dialog>

<style>
	.entry + .entry {
		border-top: 1px solid var(--border);
		padding-top: 0.75rem;
	}
</style>
