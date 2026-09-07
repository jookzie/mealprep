<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import UtensilsIcon from '@lucide/svelte/icons/utensils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MealTable from '$lib/components/meal/meal-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { formatSort, nextSort, parseSort, sortRows, type SortKey } from '$lib/domain/sort';

	let { data } = $props();

	let filter = $state('');

	// The ordering lives in the URL, as on every other list.
	const sort = $derived(parseSort(page.url.searchParams.get('sort')));

	const shown = $derived(
		sortRows(
			data.meals.filter((meal) => meal.label.toLowerCase().includes(filter.trim().toLowerCase())),
			sort
		)
	);

	function onSort(key: SortKey) {
		const url = new URL(page.url);
		url.searchParams.set('sort', formatSort(nextSort(sort, key)));
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}
</script>

<PageHeader title="Meals" description="Products with serving sizes. Macros are derived on read.">
	{#snippet actions()}
		<Button href="/meals/new">
			<PlusIcon class="size-4" />
			New meal
		</Button>
	{/snippet}
</PageHeader>

{#if data.meals.length === 0}
	<EmptyState
		icon={UtensilsIcon}
		title="No meals yet"
		description="A meal is a labelled set of products, each with a serving size."
	>
		{#snippet action()}
			<Button href="/meals/new">Create a meal</Button>
		{/snippet}
	</EmptyState>
{:else}
	<Input placeholder="Filter meals…" bind:value={filter} class="max-w-xs" />
	{#if shown.length === 0}
		<p class="text-muted-foreground text-sm">No meal matches “{filter}”.</p>
	{:else}
		<MealTable meals={shown} {sort} {onSort} />
	{/if}
{/if}
