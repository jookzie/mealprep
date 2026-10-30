<script lang="ts">
	import type { Cost } from '$lib/api';
	import { formatCost } from '$lib/domain/format';

	let { cost, label }: { cost: Cost | null; label?: string } = $props();
</script>

<!--
	A figure with missing prices is a floor, so it is shown as one — "≥ 11.20" — rather than
	withheld or passed off as the total (CO-3).
-->
<span class="cost">
	{#if label}<span class="label">{label}</span>{/if}
	{#if cost === null}
		<span class="unset">no price</span>
	{:else if cost.complete}
		<span class="amount">{formatCost(cost.amount)}</span>
	{:else}
		<span
			class="amount"
			title="Some products have no price, so this is a lower bound"
			aria-label="at least {formatCost(cost.amount)}: some products have no price"
			>≥ {formatCost(cost.amount)}</span
		>
	{/if}
</span>

<style>
	.cost {
		display: inline-flex;
		gap: 0.35rem;
		align-items: baseline;
		font-size: 0.85rem;
		font-variant-numeric: tabular-nums;
	}

	/* Inherits the ink it sits on, so it reads on the hero card as on the page. */
	.label,
	.unset {
		opacity: 0.72;
		font-weight: 600;
	}

	.amount {
		font-weight: 700;
	}
</style>
