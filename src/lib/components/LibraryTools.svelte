<script lang="ts">
	import ArrowDownWideNarrow from '@lucide/svelte/icons/arrow-down-wide-narrow';
	import ArrowUpNarrowWide from '@lucide/svelte/icons/arrow-up-narrow-wide';
	import Search from '@lucide/svelte/icons/search';
	import type { Snippet } from 'svelte';
	import { nextSort, type Sort, type SortKey } from '$lib/domain/sort';

	let {
		placeholder,
		sorts,
		query = $bindable(),
		sort = $bindable(),
		extra,
	}: {
		placeholder: string;
		sorts: readonly { key: SortKey; label: string }[];
		query: string;
		sort: Sort;
		/** A button that belongs beside the search, such as the barcode scanner. */
		extra?: Snippet;
	} = $props();
</script>

<!-- Search, sort and its direction on one line: the list's whole toolbar. -->
<div class="tools">
	<label class="search">
		<Search size={18} strokeWidth={2.4} aria-hidden="true" />
		<input type="search" {placeholder} aria-label={placeholder} bind:value={query} />
	</label>
	<select
		class="sort"
		aria-label="Sort by"
		value={sort.key}
		onchange={(event) => (sort = nextSort(sort, event.currentTarget.value as SortKey))}
	>
		{#each sorts as option (option.key)}
			<option value={option.key}>{option.label}</option>
		{/each}
	</select>
	<button
		type="button"
		class="icon direction"
		aria-label={sort.direction === 'asc' ? 'Ascending; reverse' : 'Descending; reverse'}
		onclick={() => (sort = nextSort(sort, sort.key))}
	>
		{#if sort.direction === 'asc'}
			<ArrowUpNarrowWide size={20} strokeWidth={2.4} aria-hidden="true" />
		{:else}
			<ArrowDownWideNarrow size={20} strokeWidth={2.4} aria-hidden="true" />
		{/if}
	</button>
	{@render extra?.()}
</div>

<style>
	.tools {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.search {
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

	.search input {
		min-height: 0;
		padding: 0;
		border: none;
		background: transparent;
	}

	.sort {
		width: auto;
		max-width: 7.5rem;
		border-radius: 999px;
		border-color: var(--text);
		font-weight: 700;
	}

	.direction {
		width: 48px;
		height: 48px;
		border: 2px solid var(--text);
		background: transparent;
	}
</style>
