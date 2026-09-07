<script lang="ts">
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import * as Table from '$lib/components/ui/table';
	import type { Sort, SortKey } from '$lib/domain/sort';

	// A sortable column header. aria-sort carries the state that the arrow shows, so the
	// ordering is announced rather than only drawn.
	let {
		label,
		sortKey,
		sort,
		onSort,
		align = 'left'
	}: {
		label: string;
		sortKey: SortKey;
		sort: Sort;
		onSort: (key: SortKey) => void;
		align?: 'left' | 'right';
	} = $props();

	const active = $derived(sort.key === sortKey);
</script>

<Table.Head
	class={align === 'right' ? 'text-right' : undefined}
	aria-sort={active ? (sort.direction === 'asc' ? 'ascending' : 'descending') : 'none'}
>
	<button
		type="button"
		class="text-muted-foreground hover:text-foreground focus-visible:ring-ring inline-flex items-center gap-1 rounded-sm focus-visible:ring-2 focus-visible:outline-none {align ===
		'right'
			? 'flex-row-reverse'
			: ''}"
		onclick={() => onSort(sortKey)}
	>
		{label}
		{#if !active}
			<ChevronsUpDownIcon class="size-3.5 opacity-50" />
		{:else if sort.direction === 'asc'}
			<ArrowUpIcon class="size-3.5" />
		{:else}
			<ArrowDownIcon class="size-3.5" />
		{/if}
	</button>
</Table.Head>
