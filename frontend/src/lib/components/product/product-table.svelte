<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteProduct, type Product } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import SortableHead from '$lib/components/app/sortable-head.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import { type MacroKey, MACRO_LABELS } from '$lib/domain/macros';
	import type { Sort, SortKey } from '$lib/domain/sort';
	import OffCredit from './off-credit.svelte';

	// The two things anyone actually sorts a food list by are energy and protein, and
	// until now it could be sorted by neither.
	let {
		products,
		sort,
		onSort
	}: { products: Product[]; sort: Sort; onSort: (key: SortKey) => void } = $props();

	const macroColumns: { key: MacroKey; format: (value: number) => string }[] = [
		{ key: 'energyKcal', format: formatKcal },
		{ key: 'proteinG', format: formatGrams },
		{ key: 'fatG', format: formatGrams },
		{ key: 'carbohydratesG', format: formatGrams }
	];
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<SortableHead label="Product" sortKey="name" {sort} {onSort} />
			{#each macroColumns as column (column.key)}
				<SortableHead
					label={MACRO_LABELS[column.key]}
					sortKey={column.key}
					{sort}
					{onSort}
					align="right"
				/>
			{/each}
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
					<p class="text-muted-foreground text-xs">
						{#if product.brand}{product.brand} · {/if}per 100 {product.unit}
					</p>
				</Table.Cell>
				{#each macroColumns as column (column.key)}
					<Table.Cell class="text-right tabular-nums">
						{column.format(product.macros[column.key])}
					</Table.Cell>
				{/each}
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
