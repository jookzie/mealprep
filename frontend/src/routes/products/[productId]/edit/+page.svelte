<script lang="ts">
	import { type ProductDraft, runMutation, updateProduct } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import OffCredit from '$lib/components/product/off-credit.svelte';
	import ProductForm from '$lib/components/product/product-form.svelte';
	import * as Card from '$lib/components/ui/card';

	let { data } = $props();

	// Every field the draft omits is cleared by the PUT, so the draft carries all of
	// them — brand included, or an edit would silently discard what the import captured.
	const initial = $derived<ProductDraft>({
		name: data.product.name,
		brand: data.product.brand,
		unit: data.product.unit,
		macros: data.product.macros,
		nutrients: data.product.nutrients
	});

	async function save(draft: ProductDraft) {
		await runMutation(
			() =>
				updateProduct({
					path: { productId: data.product.id },
					body: draft,
					throwOnError: true
				}),
			{ success: 'Product saved', redirectTo: `/products/${data.product.id}` }
		);
	}
</script>

<PageHeader title="Edit {data.product.name}" description="Every field is replaced on save." />

<Card.Root>
	<Card.Content class="space-y-4 pt-6">
		<!-- An import lands here, so the credit has to travel with it (§6). -->
		<OffCredit sourceCode={data.product.sourceCode} variant="full" />
		{#key data.product.updatedAt}
			<ProductForm {initial} submitLabel="Save product" onSubmit={save} />
		{/key}
	</Card.Content>
</Card.Root>
