<script lang="ts">
	import { goto } from '$app/navigation';
	import { createProduct, type ProductDraft, runMutation } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import ProductForm from '$lib/components/product/product-form.svelte';
	import * as Card from '$lib/components/ui/card';

	const empty: ProductDraft = {
		name: '',
		unit: 'g',
		macros: { energyKcal: 0, proteinG: 0, fatG: 0, carbohydratesG: 0 },
		nutrients: {}
	};

	// The new product's id only exists once the call has returned, so the redirect
	// happens here rather than through runMutation's redirectTo.
	async function create(draft: ProductDraft) {
		const created = await runMutation(() => createProduct({ body: draft, throwOnError: true }), {
			success: 'Product created'
		});
		if (created) await goto(`/products/${created.data.product.id}`);
	}
</script>

<PageHeader
	title="New product"
	description="Nutrients are per 100 g, or per 100 ml when the product is a liquid."
/>

<Card.Root>
	<Card.Content class="pt-6">
		<ProductForm initial={empty} submitLabel="Create product" onSubmit={create} />
	</Card.Content>
</Card.Root>
