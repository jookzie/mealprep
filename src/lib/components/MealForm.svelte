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
	import AmountField from './AmountField.svelte';
	import MoveButtons from './MoveButtons.svelte';
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
	const totals = $derived(draftMacros(preview));
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

<form class="stack editor" onsubmit={submit}>
	{#if restored}
		<DraftNotice onStartOver={startOver} />
	{/if}

	<label class="field">
		<span>Name</span>
		<input class="title-input" bind:value={form.label} required />
	</label>
	<CategorySelect {categories} bind:value={form.categoryId} />

	<div class="row spread">
		<h2>Servings</h2>
		<span class="muted small">In eating order</span>
	</div>
	{#if form.rows.length === 0}
		<p class="empty">No products yet.</p>
	{:else}
		<ol class="editor-rows">
			{#each form.rows as row, index (row.key)}
				{@const product = productsById.get(row.productId)}
				<li class="editor-row">
					<div class="grow">
						<strong class="truncate" class:muted={!product}>{product?.name ?? 'Product removed'}</strong>
						<span class="detail">{Math.round(preview[index]?.macros.energyKcal ?? 0)} kcal</span>
					</div>
					<AmountField
						label="Amount of {product?.name ?? 'product'}"
						unit={product?.unit ?? 'g'}
						bind:value={row.amount}
					/>
					<MoveButtons
						{index}
						length={form.rows.length}
						name={product?.name ?? 'serving'}
						onMove={(from, to) => (form.rows = move(form.rows, from, to))}
						onRemove={() => form.rows.splice(index, 1)}
					/>
				</li>
			{/each}
		</ol>
	{/if}
	<button type="button" class="add" onclick={() => (picking = true)}>+ Add a product</button>

	<div class="save-bar">
		<div class="grow totals">
			<span class="display">{Math.round(totals.energyKcal)} kcal</span>
			<span class="small">
				P {Math.round(totals.proteinG)} · F {Math.round(totals.fatG)} · C {Math.round(totals.carbohydratesG)}
				· <CostFigure cost={draftCost(preview)} />
			</span>
		</div>
		<button class="confirm" type="submit" disabled={!valid || busy}>{submitLabel}</button>
	</div>
</form>

<Dialog bind:open={picking} title="Add a product" tall>
	<ProductPicker {products} onPick={add} />
</Dialog>

<style>
	.editor {
		gap: 1rem;
	}

	.add {
		min-height: 52px;
		border: 2px dashed var(--text);
		background: transparent;
	}
</style>
