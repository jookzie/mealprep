<script lang="ts">
	import { untrack } from 'svelte';
	import type { Macros, MealDraft, Product } from '$lib/api';
	import CalculatedTotals from '$lib/components/macros/calculated-totals.svelte';
	import ProductPicker from '$lib/components/product/product-picker.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { draftMacros, servingRows } from '$lib/domain/meal';
	import { move, moveAnnouncement } from '$lib/domain/order';
	import ServingFields from './serving-fields.svelte';

	let {
		initial,
		products,
		targets,
		submitLabel,
		onSubmit
	}: {
		initial: MealDraft;
		products: Product[];
		targets: Macros | null;
		submitLabel: string;
		onSubmit: (draft: MealDraft) => Promise<void>;
	} = $props();

	/*
	 * A row carries its own id. The array is reorderable, so identity matters and the
	 * index cannot be the key: moving a row would otherwise re-associate every input
	 * below it with different data.
	 */
	type Row = { id: string; productId: string; amount: number | null };

	function row(productId: string, amount: number | null): Row {
		return { id: crypto.randomUUID(), productId, amount };
	}

	// Seeded once; the page re-seeds by keying this component on the entity.
	const seed = untrack(() => initial);
	let label = $state(seed.label);
	let rows = $state<Row[]>(seed.servings.map((serving) => row(serving.productId, serving.amount)));
	let pending = $state(false);

	// The add control: picking a product is what creates the row, rather than an empty
	// row appearing first and needing a second interaction to fill.
	let adding = $state('');
	let addOpen = $state(false);
	let focusRowId = $state<string | null>(null);
	let announcement = $state('');

	const complete = $derived(
		rows.filter(
			(candidate): candidate is Row & { amount: number } =>
				candidate.productId !== '' && candidate.amount !== null && candidate.amount > 0
		)
	);

	// A preview of an unsaved draft only. Once saved, the meal carries the API's own
	// figure and the detail view shows that instead.
	const preview = $derived(
		draftMacros(
			servingRows(
				complete.map((entry) => ({ productId: entry.productId, amount: entry.amount })),
				products
			)
		)
	);

	const valid = $derived(label.trim() !== '' && complete.length === rows.length && rows.length > 0);

	$effect(() => {
		// Picking in the add control appends the row; the effect is the one place a
		// selection turns into a mutation of the list.
		if (adding === '') return;
		const created = row(adding, null);
		rows = [...rows, created];
		focusRowId = created.id;
		adding = '';
	});

	function remove(id: string) {
		rows = rows.filter((candidate) => candidate.id !== id);
	}

	function moveRow(from: number, to: number) {
		const name = products.find((p) => p.id === rows[from].productId)?.name ?? 'Serving';
		rows = move(rows, from, to);
		announcement = moveAnnouncement(name, to, rows.length);
	}

	/*
	 * Enter in an amount field reopens the picker, so a six-product meal is one
	 * uninterrupted typing run rather than a reach for the mouse between every row.
	 */
	function commit() {
		focusRowId = null;
		addOpen = true;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!valid) return;
		pending = true;
		await onSubmit({
			label: label.trim(),
			servings: complete.map((entry) => ({ productId: entry.productId, amount: entry.amount }))
		});
		pending = false;
	}
</script>

<form onsubmit={submit}>
	<div class="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_20rem]">
		<div class="space-y-6">
			<div class="space-y-2">
				<Label for="label">Label</Label>
				<Input id="label" bind:value={label} placeholder="Porridge with berries" required />
			</div>

			<div class="space-y-3">
				<Label>Servings</Label>

				{#if products.length === 0}
					<p class="text-muted-foreground text-sm">
						There are no products yet. <a href="/products" class="underline underline-offset-4">
							Add one first</a
						>.
					</p>
				{:else}
					{#if rows.length > 0}
						<ul class="space-y-2">
							{#each rows as entry, index (entry.id)}
								<ServingFields
									{products}
									rowId={entry.id}
									{index}
									length={rows.length}
									focusAmount={focusRowId === entry.id}
									bind:productId={rows[index].productId}
									bind:amount={rows[index].amount}
									onRemove={() => remove(entry.id)}
									onMove={(to) => moveRow(index, to)}
									onCommit={commit}
								/>
							{/each}
						</ul>
					{:else}
						<p class="text-muted-foreground text-sm">
							A meal is a set of products with serving sizes. Pick the first one.
						</p>
					{/if}

					<div class="max-w-sm">
						<ProductPicker
							{products}
							bind:value={adding}
							bind:open={addOpen}
							id="add-serving"
							placeholder="Add a product…"
						/>
					</div>
				{/if}

				<!-- A reorder is otherwise invisible to anyone not watching the rows move. -->
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
