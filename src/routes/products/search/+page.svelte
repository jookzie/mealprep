<script lang="ts">
	import { goto } from '$app/navigation';
	import { navigating } from '$app/state';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ScanButton from '$lib/components/ScanButton.svelte';
	import { barcodeIn } from '$lib/domain/catalogue';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// A barcode names one exact entry, so it skips the fuzzy name search.
	function search(event: SubmitEvent & { currentTarget: HTMLFormElement }) {
		const query = new FormData(event.currentTarget).get('q');
		const code = typeof query === 'string' ? barcodeIn(query) : null;
		if (code === null) return;
		event.preventDefault();
		void goto(`/products/import/${code}`);
	}
</script>

<PageHeader title="Search catalogue" back="/products">
	{#snippet actions()}
		<ScanButton />
	{/snippet}
</PageHeader>

<form class="row" data-sveltekit-keepfocus onsubmit={search}>
	<input
		class="grow"
		type="search"
		name="q"
		placeholder="Name or barcode"
		value={data.query}
	/>
	<button class="primary" type="submit" disabled={navigating.to !== null}>Search</button>
</form>

{#if data.failure}
	<p class="card error">{data.failure}</p>
	<p class="muted small">You can still <a href="/products/new">enter the product by hand</a>.</p>
{:else if data.query !== '' && data.entries.length === 0}
	<p class="empty">No entries match “{data.query}”.</p>
{/if}

<ul class="list">
	{#each data.entries as entry (entry.code)}
		<li class="card row entry">
			{#if entry.imageUrl}
				<img src={entry.imageUrl} alt="" width="48" height="48" loading="lazy" />
			{:else}
				<div class="placeholder" aria-hidden="true"></div>
			{/if}
			<div class="grow stack details">
				<div>
					<strong>{entry.name || 'Unnamed entry'}</strong>
					{#if entry.brand}<span class="muted small"> · {entry.brand}</span>{/if}
				</div>
				{#if entry.missingMacros.length === 0}
					<span class="small muted">per 100 {entry.unit}: <MacroLine macros={entry.macros} /></span>
				{:else}
					<span class="pill">Missing figures to fill in from the package</span>
				{/if}
			</div>
			<!-- Nothing is imported unseen: every entry opens for review first. -->
			{#if entry.code !== ''}
				<a class="button" href="/products/import/{entry.code}" aria-label="Review {entry.name}"
					>Review</a
				>
			{/if}
		</li>
	{/each}
</ul>

<OffCredit />

<style>
	.entry {
		flex-wrap: nowrap;
		align-items: flex-start;
	}

	img,
	.placeholder {
		width: 48px;
		height: 48px;
		border-radius: 8px;
		object-fit: cover;
		background: var(--surface-2);
		flex: none;
	}

	.details {
		gap: 0.25rem;
	}
</style>
