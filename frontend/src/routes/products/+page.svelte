<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import WheatIcon from '@lucide/svelte/icons/wheat';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import ProductTable from '$lib/components/product/product-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { formatSort, nextSort, parseSort, sortRows, type SortKey } from '$lib/domain/sort';

	let { data } = $props();

	let filter = $state('');

	// The ordering lives in the URL, like every other thing that selects what a screen
	// shows, so it survives the back button and can be shared.
	const sort = $derived(parseSort(page.url.searchParams.get('sort')));

	const shown = $derived(
		sortRows(
			data.products.filter((product) =>
				`${product.name} ${product.brand ?? ''}`
					.toLowerCase()
					.includes(filter.trim().toLowerCase())
			),
			sort
		)
	);

	function onSort(key: SortKey) {
		const url = new URL(page.url);
		url.searchParams.set('sort', formatSort(nextSort(sort, key)));
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}
</script>

<PageHeader title="Products" description="Everything meals are built from, per 100 g or 100 ml.">
	{#snippet actions()}
		<Button href="/products/search" variant="outline">
			<SearchIcon class="size-4" />
			Search Open Food Facts
		</Button>
		<Button href="/products/new">
			<PlusIcon class="size-4" />
			New product
		</Button>
	{/snippet}
</PageHeader>

{#if data.products.length === 0}
	<EmptyState
		icon={WheatIcon}
		title="No products yet"
		description="Import one from Open Food Facts, or enter its nutrients by hand."
	>
		{#snippet action()}
			<div class="flex flex-wrap justify-center gap-2">
				<Button href="/products/search" variant="outline">Search Open Food Facts</Button>
				<Button href="/products/new">Create by hand</Button>
			</div>
		{/snippet}
	</EmptyState>
{:else}
	<Input placeholder="Filter by name or brand…" bind:value={filter} class="max-w-xs" />
	{#if shown.length === 0}
		<p class="text-muted-foreground text-sm">No product matches “{filter}”.</p>
	{:else}
		<ProductTable products={shown} {sort} {onSort} />
	{/if}
{/if}
