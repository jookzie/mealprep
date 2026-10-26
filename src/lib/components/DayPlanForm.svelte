<script lang="ts" module>
	export type DayPlanFormState = {
		label: string;
		categoryId: string | undefined;
		rows: (
			| { key: number; kind: 'meal'; mealId: string }
			| { key: number; kind: 'product'; productId: string; amount: number | null }
		)[];
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import type {
		Category,
		Cost,
		DayPlanDraft,
		DayPlanItemDraft,
		Macros,
		Meal,
		Product,
		Targets,
	} from '$lib/api';
	import { addCost, servingCost, ZERO_COST } from '$lib/domain/cost';
	import { formatAmount } from '$lib/domain/format';
	import { addMacros, servingMacros, ZERO_MACROS } from '$lib/domain/macros';
	import { move } from '$lib/domain/order';
	import type { Kept } from '$lib/memory.svelte';
	import CategorySelect from './CategorySelect.svelte';
	import CostFigure from './CostFigure.svelte';
	import Dialog from './Dialog.svelte';
	import DraftNotice from './DraftNotice.svelte';
	import GroupedPicker from './GroupedPicker.svelte';
	import MacroLine from './MacroLine.svelte';
	import MacroMeters from './MacroMeters.svelte';
	import MoveButtons from './MoveButtons.svelte';
	import NumberField from './NumberField.svelte';
	import ProductPicker from './ProductPicker.svelte';

	let {
		initial,
		draft,
		meals,
		products,
		categories,
		mealCategories,
		targets,
		submitLabel,
		onSubmit,
	}: {
		initial: DayPlanDraft;
		/** Where the form keeps what is typed, so it outlasts the screen. */
		draft: Kept<DayPlanFormState | null>;
		meals: readonly Meal[];
		products: readonly Product[];
		categories: readonly Category[];
		mealCategories: readonly Category[];
		targets: Targets | null;
		submitLabel: string;
		onSubmit: (draft: DayPlanDraft) => Promise<unknown>;
	} = $props();

	type Row = DayPlanFormState['rows'][number];

	function fresh(): DayPlanFormState {
		const seed = untrack(() => initial);
		return {
			label: seed.label,
			categoryId: seed.categoryId,
			rows: seed.items.map((item, key) => ({ ...item, key })),
		};
	}

	function toDraft(state: DayPlanFormState): DayPlanDraft {
		return { label: state.label, categoryId: state.categoryId, items: state.rows.map(toItem) };
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
	let pickingMeal = $state(false);
	let pickingProduct = $state(false);
	let busy = $state(false);

	const mealsById = $derived(new Map(meals.map((meal) => [meal.id, meal])));
	const productsById = $derived(new Map(products.map((product) => [product.id, product])));

	/** A meal carries its figures from the backend; a loose product scales like a serving. */
	function figures(row: Row): { name: string; detail: string; macros: Macros; cost: Cost } {
		if (row.kind === 'meal') {
			const meal = mealsById.get(row.mealId);
			return {
				name: meal?.label ?? 'Meal removed',
				detail: 'Meal',
				macros: meal?.macros ?? ZERO_MACROS,
				cost: meal?.cost ?? ZERO_COST,
			};
		}
		const product = productsById.get(row.productId);
		const amount = row.amount ?? 0;
		return {
			name: product?.name ?? 'Product removed',
			detail: formatAmount(amount, product?.unit),
			macros: product ? servingMacros(product.macros, amount) : ZERO_MACROS,
			cost: servingCost(product, amount),
		};
	}

	const computed = $derived(form.rows.map(figures));
	const totalMacros = $derived(computed.reduce((sum, row) => addMacros(sum, row.macros), ZERO_MACROS));
	const totalCost = $derived(computed.reduce((sum, row) => addCost(sum, row.cost), ZERO_COST));
	const valid = $derived(
		form.label.trim() !== '' &&
			form.rows.every((row) => row.kind === 'meal' || (row.amount !== null && row.amount > 0)),
	);

	function addMeal(meal: Meal) {
		form.rows.push({ key: nextKey++, kind: 'meal', mealId: meal.id });
		pickingMeal = false;
	}

	function addProduct(product: Product) {
		form.rows.push({ key: nextKey++, kind: 'product', productId: product.id, amount: 100 });
		pickingProduct = false;
	}

	function toItem(row: Row): DayPlanItemDraft {
		if (row.kind === 'meal') return { kind: 'meal', mealId: row.mealId };
		return { kind: 'product', productId: row.productId, amount: row.amount ?? 0 };
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
			<h2>What is eaten, in order</h2>
		</div>
		{#if form.rows.length === 0}
			<p class="empty">Nothing yet.</p>
		{:else}
			<ul class="list">
				{#each form.rows as row, index (row.key)}
					{@const shown = computed[index]}
					<li class="stack entry">
						<div class="row spread">
							<div class="grow">
								<strong class="truncate">{shown?.name}</strong>
								<p class="muted small">{row.kind === 'meal' ? 'Meal' : 'Product'}</p>
							</div>
							<MoveButtons
								{index}
								length={form.rows.length}
								name={shown?.name ?? 'item'}
								onMove={(from, to) => (form.rows = move(form.rows, from, to))}
								onRemove={() => form.rows.splice(index, 1)}
							/>
						</div>
						{#if row.kind === 'product'}
							<NumberField
								label="Amount"
								suffix={productsById.get(row.productId)?.unit ?? 'g'}
								bind:value={row.amount}
								required
							/>
						{/if}
						{#if shown}
							<MacroLine macros={shown.macros} />
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
		<div class="row">
			<button type="button" onclick={() => (pickingMeal = true)}>Add meal</button>
			<button type="button" onclick={() => (pickingProduct = true)}>Add product</button>
		</div>
	</section>

	<section class="card stack" aria-label="Calculated totals">
		<h2>Calculated</h2>
		<MacroMeters actual={totalMacros} target={targets?.macros ?? null} />
		<CostFigure cost={totalCost} label="Cost" />
	</section>

	<div class="row end">
		<button class="primary" type="submit" disabled={!valid || busy}>{submitLabel}</button>
	</div>
</form>

<Dialog bind:open={pickingMeal} title="Add a meal">
	<GroupedPicker items={meals} categories={mealCategories} placeholder="Filter meals" onPick={addMeal} />
</Dialog>

<Dialog bind:open={pickingProduct} title="Add a product">
	<ProductPicker {products} onPick={addProduct} />
</Dialog>

<style>
	.entry + .entry {
		border-top: 1px solid var(--border);
		padding-top: 0.75rem;
	}
</style>
