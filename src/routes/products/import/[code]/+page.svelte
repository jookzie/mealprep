<script lang="ts">
	import { importProduct, runMutation } from '$lib/api';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm from '$lib/components/ProductForm.svelte';
	import { seedFromEntry } from '$lib/domain/catalogue';
	import { MACRO_LABELS } from '$lib/domain/macros';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const date = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' });
</script>

<PageHeader title="Review import" subtitle="Barcode {data.code}" back="/products" />

{#if data.entry === null}
	<p class="card error">{data.failure}</p>
	<p class="muted small">
		You can still <a href="/products/new">enter the product by hand</a> or
		<a href="/products/search">search by name</a>.
	</p>
{:else}
	{@const entry = data.entry}
	<!-- Open Food Facts is crowd-sourced, so the page says how far to trust the entry. -->
	<section class="card stack">
		<div class="row identity">
			{#if entry.imageUrl}
				<img src={entry.imageUrl} alt="" width="56" height="56" />
			{/if}
			<div class="grow">
				<strong>{entry.name || 'Unnamed entry'}</strong>
				{#if entry.brand}<span class="muted small"> · {entry.brand}</span>{/if}
			</div>
		</div>
		<ul class="facts small">
			<li>
				{entry.source === 'manufacturer'
					? 'Supplied by the manufacturer'
					: 'Typed in by Open Food Facts contributors'}{#if entry.modifiedAt}, last edited {date.format(
						new Date(entry.modifiedAt),
					)}{/if}.
			</li>
			{#if entry.missingMacros.length > 0}
				<li class="warning">
					No {entry.missingMacros.map((key) => MACRO_LABELS[key].toLowerCase()).join(', ')} figure:
					fill it in from the package.
				</li>
			{/if}
			{#each entry.warnings as warning (warning)}
				<li class="warning">Open Food Facts flags: {warning}.</li>
			{/each}
		</ul>
		<p class="muted small">Compare the figures below with the package before importing.</p>
	</section>

	{#key entry.code}
		<ProductForm
			initial={seedFromEntry(entry)}
			submitLabel="Import product"
			onSubmit={(draft) =>
				runMutation(() => importProduct(entry.code, draft), {
					success: 'Product imported',
					redirectTo: (product) => `/products/${product.id}`,
				})}
		/>
	{/key}

	<OffCredit code={entry.code} />
{/if}

<style>
	.identity {
		flex-wrap: nowrap;
	}

	img {
		width: 56px;
		height: 56px;
		border-radius: 8px;
		object-fit: cover;
		flex: none;
	}

	.facts {
		margin: 0;
		padding-left: 1.1rem;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}
</style>
