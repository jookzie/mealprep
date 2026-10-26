<script lang="ts">
	import { createProduct, runMutation } from '$lib/api';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm, { type ProductFormState } from '$lib/components/ProductForm.svelte';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';

	const draft = $derived(keepDraft<ProductFormState>('product:new'));
</script>

<PageHeader title="New product" back="/products" />

<ProductForm
	{draft}
	submitLabel="Create product"
	onSubmit={(submitted) =>
		forgetOnSave(
			draft,
			runMutation(() => createProduct(submitted), {
				success: 'Product created',
				redirectTo: (product) => `/products/${product.id}`,
			}),
		)}
/>
