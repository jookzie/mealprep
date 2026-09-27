<script lang="ts">
	import type { Nutrients, Unit } from '$lib/api';
	import { formatGrams } from '$lib/domain/format';
	import { nutrientRows } from '$lib/domain/nutrients';

	let { nutrients, unit }: { nutrients: Nutrients; unit: Unit } = $props();

	const rows = $derived(nutrientRows(nutrients));
</script>

{#if rows.length > 0}
	<details class="card">
		<summary>All nutrients per 100 {unit} ({rows.length})</summary>
		<table>
			<tbody>
				{#each rows as row (row.key)}
					<tr>
						<th scope="row" class:indent={row.depth === 1}>{row.label}</th>
						<td>{formatGrams(row.value).replace(' g', '')}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</details>
{/if}

<style>
	summary {
		cursor: pointer;
		min-height: 32px;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		margin-top: 0.5rem;
		font-size: 0.9rem;
	}

	th {
		text-align: left;
		font-weight: 400;
		padding: 0.25rem 0;
	}

	.indent {
		padding-left: 1rem;
		color: var(--muted);
	}

	td {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	tr + tr {
		border-top: 1px solid var(--border);
	}
</style>
