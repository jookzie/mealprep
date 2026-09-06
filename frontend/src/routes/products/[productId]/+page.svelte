<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteProduct } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import NutrientPanel from '$lib/components/macros/nutrient-panel.svelte';
	import OffCredit from '$lib/components/product/off-credit.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	let { data } = $props();
</script>

<PageHeader title={data.product.name} description="Per 100 {data.product.unit}.">
	{#snippet actions()}
		<Button href="/products/{data.product.id}/edit" variant="outline">
			<PencilIcon class="size-4" />
			Edit
		</Button>
		<DeleteAction
			name={data.product.name}
			variant="outline"
			size="default"
			description="The product is removed from your list. Meals that use it will show it as removed."
			action={() => deleteProduct({ path: { productId: data.product.id }, throwOnError: true })}
			result={{ redirectTo: '/products' }}
		/>
	{/snippet}
</PageHeader>

<Card.Root>
	<Card.Header>
		<Card.Title>Macros</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-6">
		<MacrosSummary macros={data.product.macros} variant="grid" />
		<NutrientPanel nutrients={data.product.nutrients} unit={data.product.unit} />
		<OffCredit sourceCode={data.product.sourceCode} variant="full" />
	</Card.Content>
</Card.Root>
