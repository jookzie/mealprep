<script lang="ts">
	import Pencil from '@lucide/svelte/icons/pencil';
	import { deleteProduct, runMutation } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import NutrientPanel from '$lib/components/NutrientPanel.svelte';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { productCost } from '$lib/domain/cost';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const product = $derived(data.product);
</script>

<PageHeader
	title={product.name}
	eyebrow={[product.brand, product.sourceCode ? 'Open Food Facts' : 'Added by hand'].filter(Boolean).join(' · ')}
	back="/products"
	backLabel="Library"
>
	{#snippet actions()}
		<DeleteButton
			what={product.name}
			consequence="Meals and day plans that use it keep the entry and show it as removed."
			onConfirm={() =>
				runMutation(() => deleteProduct(product.id), {
					success: 'Product deleted',
					redirectTo: '/products',
				})}
		/>
		<a class="button primary" href="/products/{product.id}/edit">
			<Pencil size={16} strokeWidth={2.6} aria-hidden="true" />
			Edit
		</a>
	{/snippet}
</PageHeader>

<MacroMeters actual={product.macros} target={null} caption="kcal per 100 {product.unit}">
	{#snippet aside()}
		<span class="cost"><CostFigure cost={productCost(product)} label="per 100 {product.unit}" /></span>
	{/snippet}
</MacroMeters>

<NutrientPanel nutrients={product.nutrients} unit={product.unit} />

{#if product.sourceCode}
	<OffCredit code={product.sourceCode} />
{/if}

<style>
	.cost {
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: rgb(127 127 127 / 0.22);
	}
</style>
