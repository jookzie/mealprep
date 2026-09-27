<script lang="ts">
	import CostFigure from '$lib/components/CostFigure.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ScanButton from '$lib/components/ScanButton.svelte';
	import { productCost } from '$lib/domain/cost';
	import { nextSort, type SortKey, sortRows } from '$lib/domain/sort';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let query = $state('');
	let sort = $state(nextSort({ key: 'energyKcal', direction: 'asc' }, 'name'));

	const rows = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const matching = data.products.filter((product) =>
			`${product.name} ${product.brand ?? ''}`.toLowerCase().includes(needle),
		);
		return sortRows(matching, sort);
	});

	const SORTS: { key: SortKey; label: string }[] = [
		{ key: 'name', label: 'Name' },
		{ key: 'energyKcal', label: 'Energy' },
		{ key: 'proteinG', label: 'Protein' },
		{ key: 'fatG', label: 'Fat' },
		{ key: 'carbohydratesG', label: 'Carbs' },
	];
</script>

<PageHeader title="Products" subtitle="Everything per 100 g or ml">
	{#snippet actions()}
		<a class="button" href="/products/search">Search catalogue</a>
		<ScanButton />
		<a class="button primary" href="/products/new">New</a>
	{/snippet}
</PageHeader>

<div class="row">
	<input class="grow" type="search" placeholder="Filter by name or brand" bind:value={query} />
	<select
		class="sort"
		aria-label="Sort by"
		value={sort.key}
		onchange={(event) => (sort = nextSort(sort, event.currentTarget.value as SortKey))}
	>
		{#each SORTS as option (option.key)}
			<option value={option.key}>{option.label}</option>
		{/each}
	</select>
	<button
		class="icon"
		aria-label="Reverse order"
		onclick={() => (sort = nextSort(sort, sort.key))}>{sort.direction === 'asc' ? '↑' : '↓'}</button
	>
</div>

{#if data.products.length === 0}
	<p class="empty">No products yet. Search the catalogue or enter one by hand.</p>
{:else if rows.length === 0}
	<p class="empty">Nothing matches.</p>
{:else}
	<ul class="list">
		{#each rows as product (product.id)}
			<li>
				<a class="card list-link stack" href="/products/{product.id}">
					<div class="row spread">
						<div class="grow">
							<strong>{product.name}</strong>
							{#if product.brand}<span class="muted small"> · {product.brand}</span>{/if}
						</div>
						<span class="pill">per 100 {product.unit}</span>
					</div>
					<div class="row spread">
						<MacroLine macros={product.macros} />
						<CostFigure cost={productCost(product)} />
					</div>
				</a>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.sort {
		width: auto;
	}
</style>
