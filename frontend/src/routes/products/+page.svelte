<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import WheatIcon from '@lucide/svelte/icons/wheat';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import ProductTable from '$lib/components/product/product-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	let { data } = $props();

	let filter = $state('');
	const shown = $derived(
		data.products.filter((product) =>
			product.name.toLowerCase().includes(filter.trim().toLowerCase())
		)
	);
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
	<Input placeholder="Filter products…" bind:value={filter} class="max-w-xs" />
	{#if shown.length === 0}
		<p class="text-muted-foreground text-sm">No product matches “{filter}”.</p>
	{:else}
		<ProductTable products={shown} />
	{/if}
{/if}
