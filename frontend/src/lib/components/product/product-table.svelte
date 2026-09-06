<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteProduct, type Product } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import OffCredit from './off-credit.svelte';

	let { products }: { products: Product[] } = $props();
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head>Product</Table.Head>
			<Table.Head class="text-right">Energy</Table.Head>
			<Table.Head class="text-right">Protein</Table.Head>
			<Table.Head class="text-right">Fat</Table.Head>
			<Table.Head class="text-right">Carbs</Table.Head>
			<Table.Head class="w-1"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each products as product (product.id)}
			<Table.Row>
				<Table.Cell>
					<div class="flex flex-wrap items-center gap-2">
						<a href="/products/{product.id}" class="font-medium hover:underline">{product.name}</a>
						<OffCredit sourceCode={product.sourceCode} />
					</div>
					<p class="text-muted-foreground text-xs">per 100 {product.unit}</p>
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatKcal(product.macros.energyKcal)}
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(product.macros.proteinG)}
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(product.macros.fatG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(product.macros.carbohydratesG)}
				</Table.Cell>
				<Table.Cell>
					<div class="flex items-center justify-end gap-1">
						<Button href="/products/{product.id}/edit" variant="ghost" size="icon" title="Edit">
							<PencilIcon class="size-4" />
						</Button>
						<DeleteAction
							name={product.name}
							size="icon"
							description="The product is removed from your list. Meals that use it will show it as removed."
							action={() =>
								deleteProduct({ path: { productId: product.id }, throwOnError: true })}
						/>
					</div>
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
