<script lang="ts">
	import * as Table from '$lib/components/ui/table';
	import { formatAmount, formatGrams, formatKcal } from '$lib/domain/format';
	import type { ServingRow } from '$lib/domain/meal';

	let { rows }: { rows: ServingRow[] } = $props();
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head>Product</Table.Head>
			<Table.Head class="text-right">Serving</Table.Head>
			<Table.Head class="text-right">Energy</Table.Head>
			<Table.Head class="text-right">Protein</Table.Head>
			<Table.Head class="text-right">Fat</Table.Head>
			<Table.Head class="text-right">Carbs</Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		<!-- A meal may legitimately hold the same product twice, so the id alone is not
		     an identity for a row. -->
		{#each rows as row, index (`${row.productId}-${index}`)}
			<Table.Row>
				<Table.Cell>
					{#if row.product}
						<a href="/products/{row.product.id}" class="font-medium hover:underline">
							{row.product.name}
						</a>
					{:else}
						<!-- The product was deleted; the serving still exists on the meal. -->
						<span class="text-muted-foreground italic">Product removed</span>
					{/if}
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatAmount(row.amount, row.unit)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatKcal(row.macros.energyKcal)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(row.macros.proteinG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(row.macros.fatG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(row.macros.carbohydratesG)}
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
