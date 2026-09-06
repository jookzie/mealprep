<script lang="ts">
	import XIcon from '@lucide/svelte/icons/x';
	import type { Product } from '$lib/api';
	import NumberField from '$lib/components/app/number-field.svelte';
	import ProductPicker from '$lib/components/product/product-picker.svelte';
	import { Button } from '$lib/components/ui/button';
	import { formatKcal } from '$lib/domain/format';
	import { servingMacros } from '$lib/domain/macros';

	let {
		products,
		productId = $bindable(),
		amount = $bindable(),
		index,
		onRemove
	}: {
		products: readonly Product[];
		productId: string;
		amount: number | null;
		index: number;
		onRemove: () => void;
	} = $props();

	const product = $derived(products.find((candidate) => candidate.id === productId));
	const energy = $derived(
		product && amount !== null ? formatKcal(servingMacros(product.macros, amount).energyKcal) : '—'
	);
</script>

<li class="flex flex-wrap items-end gap-2 sm:flex-nowrap">
	<div class="min-w-48 flex-1">
		<ProductPicker {products} bind:value={productId} id="serving-{index}-product" />
	</div>
	<div class="w-32">
		<NumberField
			id="serving-{index}-amount"
			suffix={product?.unit ?? 'g'}
			bind:value={amount}
			min={0}
		/>
	</div>
	<p class="text-muted-foreground w-24 pb-2 text-right text-sm tabular-nums">{energy}</p>
	<Button type="button" variant="ghost" size="icon" onclick={onRemove} aria-label="Remove serving">
		<XIcon class="size-4" />
	</Button>
</li>
