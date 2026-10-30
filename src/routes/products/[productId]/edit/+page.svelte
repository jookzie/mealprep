<script lang="ts">
	import { runMutation, updateProduct } from '$lib/api';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm, { type ProductFormState } from '$lib/components/ProductForm.svelte';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const product = $derived(data.product);

	const draft = $derived(keepDraft<ProductFormState>(`product:${product.id}:${product.updatedAt}`));
</script>

<PageHeader title="Edit product" back="/products/{product.id}" backLabel={product.name} />

<!-- Keyed on updatedAt, so a save re-seeds the form while a refresh mid-edit does not. -->
{#key product.updatedAt}
	<ProductForm
		initial={product}
		{draft}
		submitLabel="Save"
		onSubmit={(submitted) =>
			forgetOnSave(
				draft,
				runMutation(() => updateProduct(product.id, submitted), {
					success: 'Product saved',
					redirectTo: `/products/${product.id}`,
				}),
			)}
	/>
{/key}

{#if product.sourceCode}
	<OffCredit code={product.sourceCode} />
{/if}
