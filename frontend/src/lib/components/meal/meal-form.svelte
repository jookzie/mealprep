<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { untrack } from 'svelte';
	import type { MealDraft, Product } from '$lib/api';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';
	import { draftMacros, servingRows } from '$lib/domain/meal';
	import ServingFields from './serving-fields.svelte';

	let {
		initial,
		products,
		submitLabel,
		onSubmit
	}: {
		initial: MealDraft;
		products: Product[];
		submitLabel: string;
		onSubmit: (draft: MealDraft) => Promise<void>;
	} = $props();

	type Row = { productId: string; amount: number | null };

	// Seeded once; the page re-seeds by keying this component on the entity.
	const seed = untrack(() => initial);
	let label = $state(seed.label);
	let rows = $state<Row[]>(seed.servings.map((serving) => ({ ...serving })));
	let pending = $state(false);

	const complete = $derived(
		rows.filter((row): row is { productId: string; amount: number } =>
			row.productId !== '' && row.amount !== null && row.amount > 0
		)
	);

	// A preview of an unsaved draft only. Once saved, the meal carries the API's own
	// figure and the detail view shows that instead.
	const preview = $derived(draftMacros(servingRows(complete, products)));

	const valid = $derived(label.trim() !== '' && complete.length === rows.length && rows.length > 0);

	function add() {
		rows = [...rows, { productId: '', amount: null }];
	}

	function remove(index: number) {
		rows = rows.filter((_, i) => i !== index);
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!valid) return;
		pending = true;
		await onSubmit({ label: label.trim(), servings: complete });
		pending = false;
	}
</script>

<form class="space-y-6" onsubmit={submit}>
	<div class="space-y-2">
		<Label for="label">Label</Label>
		<Input id="label" bind:value={label} placeholder="Porridge with berries" required />
	</div>

	<Separator />

	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<Label>Servings</Label>
			<Button type="button" variant="outline" size="sm" onclick={add} disabled={products.length === 0}>
				<PlusIcon class="size-4" />
				Add serving
			</Button>
		</div>

		{#if products.length === 0}
			<p class="text-muted-foreground text-sm">
				There are no products yet. <a href="/products" class="underline underline-offset-4">
					Add one first</a
				>.
			</p>
		{:else if rows.length === 0}
			<p class="text-muted-foreground text-sm">
				A meal is a set of products with serving sizes. Add the first one.
			</p>
		{:else}
			<ul class="space-y-2">
				{#each rows as _row, index (index)}
					<ServingFields
						{products}
						{index}
						bind:productId={rows[index].productId}
						bind:amount={rows[index].amount}
						onRemove={() => remove(index)}
					/>
				{/each}
			</ul>
		{/if}
	</div>

	{#if rows.length > 0}
		<div class="bg-muted/40 rounded-lg p-4">
			<p class="text-muted-foreground mb-2 text-xs">Draft total</p>
			<MacrosSummary macros={preview} />
		</div>
	{/if}

	<Button type="submit" disabled={pending || !valid}>
		{pending ? 'Saving…' : submitLabel}
	</Button>
</form>
