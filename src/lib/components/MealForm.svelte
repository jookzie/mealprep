<script lang="ts">
	import { untrack } from 'svelte';
	import type { Category, MealDraft, Product } from '$lib/api';
	import { draftCost, draftMacros, servingRows } from '$lib/domain/meal';
	import { move } from '$lib/domain/order';
	import CategorySelect from './CategorySelect.svelte';
	import CostFigure from './CostFigure.svelte';
	import Dialog from './Dialog.svelte';
	import MacroLine from './MacroLine.svelte';
	import MoveButtons from './MoveButtons.svelte';
	import NumberField from './NumberField.svelte';
	import ProductPicker from './ProductPicker.svelte';

	let {
		initial,
		products,
		categories,
		submitLabel,
		onSubmit,
	}: {
		initial: MealDraft;
		products: readonly Product[];
		categories: readonly Category[];
		submitLabel: string;
		onSubmit: (draft: MealDraft) => Promise<unknown>;
	} = $props();

	type Row = { key: number; productId: string; amount: number | null };

	const seed = untrack(() => initial);
	let nextKey = 0;
	let label = $state(seed.label);
	let categoryId = $state(seed.categoryId);
	let rows = $state<Row[]>(
		seed.servings.map((serving) => ({
			key: nextKey++,
			productId: serving.productId,
			amount: serving.amount,
		})),
	);
	let picking = $state(false);
	let busy = $state(false);

	const productsById = $derived(new Map(products.map((product) => [product.id, product])));
	const preview = $derived(
		servingRows(
			rows.map((row) => ({ productId: row.productId, amount: row.amount ?? 0 })),
			products,
		),
	);
	const valid = $derived(
		label.trim() !== '' && rows.every((row) => row.amount !== null && row.amount > 0),
	);

	function add(product: Product) {
		rows.push({ key: nextKey++, productId: product.id, amount: 100 });
		picking = false;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		try {
			await onSubmit({
				label,
				categoryId,
				servings: rows.map((row) => ({ productId: row.productId, amount: row.amount ?? 0 })),
			});
		} finally {
			busy = false;
		}
	}
</script>

<form class="stack" onsubmit={submit}>
	<section class="card stack">
		<label class="field">
			<span>Label</span>
			<input bind:value={label} required />
		</label>
		<CategorySelect {categories} bind:value={categoryId} />
	</section>

	<section class="card stack">
		<div class="row spread">
			<h2>Servings</h2>
			<button type="button" onclick={() => (picking = true)}>Add product</button>
		</div>
		{#if rows.length === 0}
			<p class="empty">No products yet.</p>
		{:else}
			<ul class="list">
				{#each rows as row, index (row.key)}
					{@const product = productsById.get(row.productId)}
					<li class="stack entry">
						<div class="row spread">
							<strong class="grow truncate" class:muted={!product}
								>{product?.name ?? 'Product removed'}</strong
							>
							<MoveButtons
								{index}
								length={rows.length}
								name={product?.name ?? 'serving'}
								onMove={(from, to) => (rows = move(rows, from, to))}
								onRemove={() => rows.splice(index, 1)}
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
