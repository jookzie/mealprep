<script lang="ts">
	import type { Nutrients, Unit } from '$lib/api';
	import { formatGrams } from '$lib/domain/format';
	import { nutrientRows } from '$lib/domain/nutrients';

	let { nutrients, unit }: { nutrients: Nutrients; unit: Unit } = $props();

	const rows = $derived(nutrientRows(nutrients));
</script>

{#if rows.length > 0}
	<details class="panel">
		<summary>
			<span class="display">Full label per 100 {unit}</span>
			<span class="muted small">{rows.length} more</span>
		</summary>
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
	.panel {
		border: 2px solid var(--text);
		border-radius: var(--radius);
		padding: 0.25rem 1rem 0.5rem;
	}

	summary {
		cursor: pointer;
		min-height: 44px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		list-style: none;
	}

	summary::-webkit-details-marker {
		display: none;
	}

	summary .display {
		font-size: 1.15rem;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		margin-top: 0.5rem;
		font-size: 0.9rem;
	}

	th {
		text-align: left;
		font-weight: 700;
		padding: 0.4rem 0;
	}

	.indent {
		padding-left: 1rem;
		color: var(--muted);
	}

	.indent {
		font-weight: 500;
	}

	td {
		text-align: right;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}

	tr + tr {
		border-top: 1.5px solid var(--border);
	}
</style>
