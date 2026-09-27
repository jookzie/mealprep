<script lang="ts">
	import { deleteProduct, runMutation } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import NutrientPanel from '$lib/components/NutrientPanel.svelte';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { productCost } from '$lib/domain/cost';
	import { formatUnitName } from '$lib/domain/format';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const product = $derived(data.product);
</script>

<PageHeader title={product.name} subtitle={product.brand} back="/products">
	{#snippet actions()}
		<a class="button" href="/products/{product.id}/edit">Edit</a>
		<DeleteButton
			what={product.name}
			consequence="Meals and day plans that use it keep the entry and show it as removed."
			onConfirm={() =>
				runMutation(() => deleteProduct(product.id), {
					success: 'Product deleted',
					redirectTo: '/products',
				})}
		/>
	{/snippet}
</PageHeader>

<section class="card stack">
	<div class="row spread">
		<h2>Per 100 {formatUnitName(product.unit)}</h2>
		<CostFigure cost={productCost(product)} label="Cost" />
	</div>
	<MacroMeters actual={product.macros} target={null} />
</section>

<NutrientPanel nutrients={product.nutrients} unit={product.unit} />

{#if product.sourceCode}
	<OffCredit code={product.sourceCode} />
{:else}
	<p class="muted small">Entered by hand.</p>
{/if}
