<script lang="ts">
	import type { Cost } from '$lib/api';
	import { formatCost } from '$lib/domain/format';

	let { cost, label }: { cost: Cost | null; label?: string } = $props();
</script>

<!-- A figure with missing prices is a floor, so it is shown with its caveat rather than withheld. -->
<span class="cost small">
	{#if label}<span class="muted">{label}</span>{/if}
	{#if cost === null}
		<span class="muted">unset</span>
	{:else}
		<span class="amount">{formatCost(cost.amount)}</span>{#if !cost.complete}<abbr
				title="Some products are missing cost values, so this is a lower bound"
				aria-label="incomplete: some products are missing cost values">*</abbr
			>{/if}
	{/if}
</span>

<style>
	.cost {
		display: inline-flex;
		gap: 0.3rem;
		align-items: baseline;
		font-variant-numeric: tabular-nums;
	}

	.amount {
		border-bottom: 2px solid var(--cost);
	}

	abbr {
		text-decoration: none;
		color: var(--muted);
	}
</style>
