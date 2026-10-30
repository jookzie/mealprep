<script lang="ts">
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Search from '@lucide/svelte/icons/search';
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

<PageHeader title="Open Food Facts" eyebrow="Find a product to add" back="/products" backLabel="Library" />

<form class="bar" data-sveltekit-keepfocus onsubmit={search}>
	<label class="field-pill">
		<Search size={18} strokeWidth={2.4} aria-hidden="true" />
		<input type="search" name="q" placeholder="Name or barcode" aria-label="Name or barcode" value={data.query} />
	</label>
	<button class="primary" type="submit" disabled={navigating.to !== null}>Search</button>
	<ScanButton compact />
</form>

{#if data.failure}
	<p class="notice">{data.failure}</p>
	<p class="muted small">You can still <a href="/products/new">enter the product by hand</a>.</p>
{:else if data.query !== '' && data.entries.length === 0}
	<p class="empty">
		No entries match “{data.query}”. <a href="/products/new">Enter it by hand</a> instead.
	</p>
{:else if data.query === ''}
	<p class="empty">Search by name, type a barcode, or scan one.</p>
{/if}

<ul class="list">
	{#each data.entries as entry (entry.code)}
		<li>
			<!-- Nothing is imported unseen: every entry opens for review first. -->
			<a class="card entry" href={entry.code === '' ? undefined : `/products/import/${entry.code}`}>
				{#if entry.imageUrl}
					<img src={entry.imageUrl} alt="" width="56" height="56" loading="lazy" />
				{:else}
					<span class="placeholder" aria-hidden="true"></span>
				{/if}
				<span class="grow details">
					<strong class="truncate">{entry.name || 'Unnamed entry'}</strong>
					{#if entry.brand}<span class="muted small truncate">{entry.brand}</span>{/if}
					{#if entry.missingMacros.length === 0}
						<span class="row figures">
							<MacroLine macros={entry.macros} />
						</span>
					{:else}
						<span class="missing small">Some figures missing — fill them in from the package</span>
					{/if}
				</span>
				{#if entry.code !== ''}
					<ChevronRight size={20} aria-hidden="true" />
				{/if}
			</a>
		</li>
	{/each}
</ul>

{#if data.entries.length > 0}
	<p class="small muted">Figures are per 100 g or ml, as the catalogue lists them.</p>
{/if}

<OffCredit />

<style>
	.bar {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.field-pill {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		height: 48px;
		padding: 0 0.9rem;
		border: 2px solid var(--text);
		border-radius: 999px;
	}

	.field-pill input {
		min-height: 0;
		padding: 0;
		border: none;
		background: transparent;
	}

	.bar .primary {
		min-height: 48px;
	}

	.entry {
		display: flex;
		align-items: center;
		gap: 0.85rem;
		text-decoration: none;
	}

	img,
	.placeholder {
		width: 56px;
		height: 56px;
		border-radius: 16px;
		object-fit: cover;
		background: var(--raised);
		flex: none;
	}

	.details {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.figures {
		margin-top: 0.15rem;
	}

	.missing {
		align-self: flex-start;
		font-weight: 700;
		padding: 0.15rem 0.6rem;
		border-radius: 8px;
		background: var(--warning-bg);
		color: #111111;
	}
</style>
