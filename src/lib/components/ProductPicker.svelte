<script lang="ts">
	import type { Product } from '$lib/api';
	import { byRecency } from '$lib/domain/sort';
	import MacroLine from './MacroLine.svelte';

	let { products, onPick }: { products: readonly Product[]; onPick: (product: Product) => void } =
		$props();

	let query = $state('');

	const matches = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const recent = byRecency(products);
		if (needle === '') return recent;
		return recent.filter((product) =>
			`${product.name} ${product.brand ?? ''}`.toLowerCase().includes(needle),
		);
	});
</script>

<div class="stack">
	<!-- svelte-ignore a11y_autofocus -->
	<input type="search" placeholder="Filter products" bind:value={query} autofocus />
	{#if matches.length === 0}
		<p class="empty">No products match. Add one from the Products tab.</p>
	{:else}
		<ul class="list picker">
			{#each matches as product (product.id)}
				<li>
					<button type="button" class="pick" onclick={() => onPick(product)}>
						<span class="truncate"
							><strong>{product.name}</strong>
							{#if product.brand}<span class="muted small">{product.brand}</span>{/if}</span
						>
						<span class="muted small">per 100 {product.unit}: <MacroLine macros={product.macros} /></span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.picker {
		max-height: 55dvh;
		overflow-y: auto;
	}

	.pick {
		width: 100%;
		flex-direction: column;
		align-items: flex-start;
		text-align: left;
		background: var(--surface);
	}
</style>
