<script lang="ts">
	import { importProduct, runMutation } from '$lib/api';
	import OffCredit from '$lib/components/OffCredit.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProductForm, { type ProductFormState } from '$lib/components/ProductForm.svelte';
	import { seedFromEntry } from '$lib/domain/catalogue';
	import { MACRO_LABELS } from '$lib/domain/macros';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const date = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' });

	const draft = $derived(keepDraft<ProductFormState>(`import:${data.code}`));
</script>

<PageHeader title="Review import" eyebrow="Barcode {data.code}" back="/products" backLabel="Library" />

{#if data.entry === null}
	<p class="notice">{data.failure}</p>
	<p class="muted small">
		You can still <a href="/products/new">enter the product by hand</a> or
		<a href="/products/search">search by name</a>.
	</p>
{:else}
	{@const entry = data.entry}
	<!-- Open Food Facts is crowd-sourced, so the page says how far to trust the entry. -->
	<section class="card stack source">
		<div class="row identity">
			{#if entry.imageUrl}
				<img src={entry.imageUrl} alt="" width="64" height="64" />
			{/if}
			<div class="grow">
				<strong class="display name">{entry.name || 'Unnamed entry'}</strong>
				{#if entry.brand}<p class="muted small strong">{entry.brand}</p>{/if}
			</div>
		</div>
		<p class="small">
			<span class="pill" class:trusted={entry.source === 'manufacturer'}>
				{entry.source === 'manufacturer' ? 'From the manufacturer' : 'Typed in by contributors'}
			</span>
			{#if entry.modifiedAt}
				<span class="muted"> · last edited {date.format(new Date(entry.modifiedAt))}</span>
			{/if}
		</p>
		<p class="muted small">Compare the figures below with the package before importing.</p>
	</section>

	{#if entry.missingMacros.length > 0 || entry.warnings.length > 0}
		<ul class="notice facts small" aria-label="Before you import">
			{#if entry.missingMacros.length > 0}
				<li>
					No {entry.missingMacros.map((key) => MACRO_LABELS[key].toLowerCase()).join(', ')} figure:
					fill it in from the package.
				</li>
			{/if}
			{#each entry.warnings as warning (warning)}
				<li>Open Food Facts flags: {warning}.</li>
			{/each}
		</ul>
	{/if}

	{#key entry.code}
		<ProductForm
			initial={seedFromEntry(entry)}
			{draft}
			submitLabel="Import"
			onSubmit={(submitted) =>
				forgetOnSave(
					draft,
					runMutation(() => importProduct(entry.code, submitted), {
						success: 'Product imported',
						redirectTo: (product) => `/products/${product.id}`,
					}),
				)}
		/>
	{/key}

	<OffCredit code={entry.code} />
{/if}

<style>
	.identity {
		flex-wrap: nowrap;
	}

	img {
		width: 64px;
		height: 64px;
		border-radius: 18px;
		object-fit: cover;
		flex: none;
	}

	.name {
		font-size: 1.35rem;
		line-height: 1.1;
	}

	.strong {
		font-weight: 700;
	}

	.trusted {
		background: var(--carbs);
		color: var(--on-macro);
	}

	.facts {
		margin: 0;
		padding-left: 2rem;
		list-style: disc;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-weight: 600;
	}
</style>
