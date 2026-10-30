<script lang="ts">
	import Globe from '@lucide/svelte/icons/globe';
	import Plus from '@lucide/svelte/icons/plus';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import LibraryTools from '$lib/components/LibraryTools.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ScanButton from '$lib/components/ScanButton.svelte';
	import SectionSwitch from '$lib/components/SectionSwitch.svelte';
	import { productCost } from '$lib/domain/cost';
	import { nextSort, type SortKey, sortRows } from '$lib/domain/sort';
	import { keep } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Origin = 'all' | 'catalogue' | 'hand' | 'unpriced';

	const query = keep('products:query', '');
	const sort = keep('products:sort', nextSort({ key: 'energyKcal', direction: 'asc' }, 'name'));
	const origin = keep<Origin>('products:origin', 'all');

	const ORIGINS: { key: Origin; label: string }[] = [
		{ key: 'all', label: 'All' },
		{ key: 'catalogue', label: 'Open Food Facts' },
		{ key: 'hand', label: 'By hand' },
		{ key: 'unpriced', label: 'No price' },
	];

	// The origin narrows the list for finding things; nothing downstream depends on it (PR-5).
	function matchesOrigin(product: (typeof data.products)[number]): boolean {
		if (origin.value === 'catalogue') return product.sourceCode !== undefined;
		if (origin.value === 'hand') return product.sourceCode === undefined;
		if (origin.value === 'unpriced') return product.cost === undefined;
		return true;
	}

	const rows = $derived.by(() => {
		const needle = query.value.trim().toLowerCase();
		const matching = data.products.filter(
			(product) =>
				matchesOrigin(product) &&
				`${product.name} ${product.brand ?? ''}`.toLowerCase().includes(needle),
		);
		return sortRows(matching, sort.value);
	});

	const SORTS: { key: SortKey; label: string }[] = [
		{ key: 'name', label: 'Name' },
		{ key: 'energyKcal', label: 'Energy' },
		{ key: 'proteinG', label: 'Protein' },
		{ key: 'fatG', label: 'Fat' },
		{ key: 'carbohydratesG', label: 'Carbs' },
	];
</script>

<PageHeader title="Library">
	{#snippet actions()}
		<a class="button primary" href="/products/new">
			<Plus size={18} strokeWidth={2.6} aria-hidden="true" />
			New product
		</a>
	{/snippet}
</PageHeader>

<SectionSwitch
	label="Library"
	current="/products"
	links={[
		{ href: '/meals', label: 'Meals' },
		{ href: '/products', label: `Products · ${data.products.length}` },
	]}
/>

<LibraryTools placeholder="Search your products" sorts={SORTS} bind:query={query.value} bind:sort={sort.value}>
	{#snippet extra()}
		<ScanButton compact />
	{/snippet}
</LibraryTools>

<div class="chips" role="group" aria-label="Show">
	{#each ORIGINS as option (option.key)}
		<button class="chip" aria-pressed={origin.value === option.key} onclick={() => (origin.value = option.key)}
			>{option.label}</button
		>
	{/each}
</div>

<a class="button outline catalogue" href="/products/search{query.value.trim() ? `?q=${encodeURIComponent(query.value.trim())}` : ''}">
	<Globe size={18} strokeWidth={2.4} aria-hidden="true" />
	Search Open Food Facts{query.value.trim() ? ` for “${query.value.trim()}”` : ''}
</a>

{#if data.products.length === 0}
	<p class="empty">No products yet. Search the catalogue, scan a barcode, or enter one by hand.</p>
{:else if rows.length === 0}
	<p class="empty">Nothing matches.</p>
{:else}
	<ul class="list">
		{#each rows as product (product.id)}
			<li>
				<a class="card list-link entry" href="/products/{product.id}">
					<div class="row spread top">
						<span class="grow name">
							<strong class="truncate">{product.name}</strong>
							<span class="muted small meta truncate">
								{product.brand ? `${product.brand} · ` : ''}{product.sourceCode ? 'Open Food Facts' : 'By hand'}
							</span>
						</span>
						<span class="energy">
							<span class="display figure">{Math.round(product.macros.energyKcal)}</span>
							<span class="muted per">kcal / 100 {product.unit}</span>
						</span>
					</div>
					<div class="row spread">
						<MacroLine macros={product.macros} energy={false} />
						<CostFigure cost={productCost(product)} />
					</div>
				</a>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.catalogue {
		justify-content: flex-start;
		white-space: normal;
		text-align: left;
	}

	.entry {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.top {
		flex-wrap: nowrap;
		align-items: flex-start;
	}

	.name {
		display: flex;
		flex-direction: column;
	}

	.meta {
		font-weight: 600;
	}

	.energy {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		line-height: 1.1;
	}

	.energy .display {
		font-size: 1.15rem;
	}

	.per {
		font-size: 0.7rem;
		font-weight: 600;
		white-space: nowrap;
	}
</style>
