<script lang="ts">
	import ClockIcon from '@lucide/svelte/icons/clock';
	import SearchIcon from '@lucide/svelte/icons/search';
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { importProduct, runMutation } from '$lib/api';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import ErrorAlert from '$lib/components/app/error-alert.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import CatalogEntryList from '$lib/components/product/catalog-entry-list.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	let { data } = $props();

	// Seeded from the URL once. The load re-runs as the user types, and re-syncing
	// from it would fight the input rather than follow it.
	let query = $state(untrack(() => data.query));
	let importing = $state<string | null>(null);
	let timer: ReturnType<typeof setTimeout> | undefined;

	/*
	 * Debounced to 500 ms rather than fired per keystroke. This is not only a politeness:
	 * the catalog allows ten searches a minute per address, which is one every six
	 * seconds, so a live type-ahead against it would rate-limit within seconds.
	 * replaceState keeps the back button pointing at where the user came from.
	 */
	function onInput(event: Event) {
		query = (event.currentTarget as HTMLInputElement).value;
		clearTimeout(timer);
		timer = setTimeout(() => {
			const url = new URL(window.location.href);
			if (query.trim() === '') url.searchParams.delete('q');
			else url.searchParams.set('q', query.trim());
			goto(url, { replaceState: true, keepFocus: true, noScroll: true });
		}, 500);
	}

	/*
	 * The import lands on the product's own editor rather than its detail page. The
	 * catalog's names are frequently messy — "Oats Rolled Jumbo Organic 1KG" — and this
	 * is the snapshot the user lives with afterwards, since it is never re-read from the
	 * source (PR-8). Landing in the form is the moment to fix it.
	 *
	 * Deliberately does not invalidate: that would re-run the search load and spend
	 * another catalog round trip on results the user is leaving behind.
	 */
	async function importEntry(code: string) {
		importing = code;
		const imported = await runMutation(
			() => importProduct({ body: { code }, throwOnError: true }),
			{ success: 'Imported — check the name and values before you rely on them', invalidate: false }
		);
		importing = null;
		if (imported) await goto(`/products/${imported.data.product.id}/edit`);
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

{#if data.failure}
	<ErrorAlert title="The catalog could not be reached" message={data.failure} />
{:else if data.rateLimited}
	<EmptyState
		icon={ClockIcon}
		title="Open Food Facts is rate-limiting requests"
		description="It allows ten searches a minute. Wait a moment and try again."
	/>
{:else if data.entries === null}
	<EmptyState
		icon={SearchIcon}
		title="Search the catalog"
		description="Type a product name to see matching entries from Open Food Facts."
	/>
{:else if data.entries.length === 0}
	<EmptyState title="No matches" description="Nothing in the catalog matches “{data.query}”.">
		{#snippet action()}
			<Button href="/products/new">Create it by hand</Button>
		{/snippet}
	</EmptyState>
{:else}
	<CatalogEntryList entries={data.entries} {importing} onImport={importEntry} />
	<p class="text-muted-foreground text-sm">
		Catalog data from
		<a
			href="https://world.openfoodfacts.org"
			target="_blank"
			rel="noreferrer noopener"
			class="text-foreground underline underline-offset-4">Open Food Facts</a
		>, under the Open Database License. Product images are theirs and are linked, not stored.
	</p>
{/if}
