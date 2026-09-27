<script lang="ts">
	import { runMutation, updateProduct } from '$lib/api';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm from '$lib/components/ProductForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const product = $derived(data.product);
</script>

<PageHeader title="Edit product" back="/products/{product.id}" />

<!-- Keyed on updatedAt, so a save re-seeds the form while a refresh mid-edit does not. -->
{#key product.updatedAt}
	<ProductForm
		initial={product}
		submitLabel="Save"
		onSubmit={(draft) =>
			runMutation(() => updateProduct(product.id, draft), {
				success: 'Product saved',
				redirectTo: `/products/${product.id}`,
			})}
	/>
{/key}

{#if product.sourceCode}
	<OffCredit code={product.sourceCode} />
{/if}
