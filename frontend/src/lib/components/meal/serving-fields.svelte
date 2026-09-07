<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Product } from '$lib/api';
	import MoveButtons from '$lib/components/app/move-buttons.svelte';
	import NumberField from '$lib/components/app/number-field.svelte';
	import ProductPicker from '$lib/components/product/product-picker.svelte';
	import { Button } from '$lib/components/ui/button';
	import { formatKcal } from '$lib/domain/format';
	import { servingMacros } from '$lib/domain/macros';

	let {
		products,
		productId = $bindable(),
		amount = $bindable(),
		rowId,
		index,
		length,
		focusAmount = false,
		onRemove,
		onMove,
		onCommit
	}: {
		products: readonly Product[];
		productId: string;
		amount: number | null;
		rowId: string;
		index: number;
		length: number;
		/** Set on the row the picker just created, so typing continues in the amount. */
		focusAmount?: boolean;
		onRemove: () => void;
		onMove: (to: number) => void;
		onCommit: () => void;
	} = $props();

	let amountRef = $state<HTMLInputElement | null>(null);

	const product = $derived(products.find((candidate) => candidate.id === productId));
	const energy = $derived(
		product && amount !== null ? formatKcal(servingMacros(product.macros, amount).energyKcal) : '—'
	);

	$effect(() => {
		// An escape hatch by design: focus is a DOM action with no derived form, and it
		// has to happen once, on the row the add control has just created.
		if (focusAmount) amountRef?.focus();
	});
</script>

<li class="flex flex-wrap items-end gap-2 sm:flex-nowrap">
	<div class="pb-1.5">
		<MoveButtons {index} {length} name={product?.name ?? 'serving'} {onMove} />
	</div>
	<div class="min-w-48 flex-1">
		<ProductPicker {products} bind:value={productId} id="serving-{rowId}-product" />
	</div>
	<div class="w-28">
		<NumberField
			bind:ref={amountRef}
			id="serving-{rowId}-amount"
			ariaLabel="Amount of {product?.name ?? 'serving'}"
			suffix={product?.unit ?? 'g'}
			bind:value={amount}
			placeholder="0"
			{onCommit}
		/>
	</div>
	<p class="text-muted-foreground w-24 pb-2 text-right text-sm tabular-nums">{energy}</p>
	<Button
		type="button"
		variant="ghost"
		size="icon"
		onclick={onRemove}
		aria-label="Remove {product?.name ?? 'serving'}"
	>
		<XIcon class="size-4" />
	</Button>
</li>
