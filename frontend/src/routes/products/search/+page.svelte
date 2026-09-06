<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { importProduct, runMutation } from '$lib/api';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import CatalogEntryTable from '$lib/components/product/catalog-entry-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	let { data } = $props();

	// Seeded from the URL once. The load re-runs as the user types, and re-syncing
	// from it would fight the input rather than follow it.
	let query = $state(untrack(() => data.query));
	let importing = $state<string | null>(null);
	let timer: ReturnType<typeof setTimeout> | undefined;

	// Every keystroke would otherwise become a history entry and a request; replaceState
	// keeps the back button pointing at where the user came from.
	function onInput(event: Event) {
		query = (event.currentTarget as HTMLInputElement).value;
		clearTimeout(timer);
		timer = setTimeout(() => {
			const url = new URL(window.location.href);
			if (query.trim() === '') url.searchParams.delete('q');
			else url.searchParams.set('q', query.trim());
			goto(url, { replaceState: true, keepFocus: true, noScroll: true });
		}, 300);
	}

	// Deliberately does not invalidate: that would re-run the search load and spend
	// another Open Food Facts round trip on results the user is leaving behind.
	async function importEntry(code: string) {
		importing = code;
		const imported = await runMutation(
			() => importProduct({ body: { code }, throwOnError: true }),
			{ success: 'Product imported', invalidate: false }
		);
		importing = null;
		if (imported) await goto(`/products/${imported.data.product.id}`);
	}
</script>

<PageHeader
	title="Search Open Food Facts"
	description="Picking an entry snapshots its values into your products; it is never re-read afterwards."
>
	{#snippet actions()}
		<Button href="/products" variant="outline">Back to products</Button>
	{/snippet}
</PageHeader>

<Input
	value={query}
	oninput={onInput}
	placeholder="Search the catalog, e.g. rolled oats"
	class="max-w-md"
/>

{#if data.entries === null}
	<EmptyState
		icon={SearchIcon}
		title="Search the catalog"
		description="Type a product name to see matching entries from Open Food Facts."
	/>
{:else if data.entries.length === 0}
	<EmptyState title="No matches" description="Nothing in the catalog matches “{data.query}”." />
{:else}
	<CatalogEntryTable entries={data.entries} {importing} onImport={importEntry} />
	<p class="text-muted-foreground text-sm">
		Catalog data from
		<a
			href="https://world.openfoodfacts.org"
			target="_blank"
			rel="noreferrer noopener"
			class="text-foreground underline underline-offset-4">Open Food Facts</a
		>, under the Open Database License.
	</p>
{/if}
