<script lang="ts">
	import { createProduct, runMutation } from '$lib/api';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm from '$lib/components/ProductForm.svelte';
</script>

<PageHeader title="New product" back="/products" />

<ProductForm
	submitLabel="Create product"
	onSubmit={(draft) =>
		runMutation(() => createProduct(draft), {
			success: 'Product created',
			redirectTo: (product) => `/products/${product.id}`,
		})}
/>
