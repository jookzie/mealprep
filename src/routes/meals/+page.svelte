<script lang="ts">
	import CostFigure from '$lib/components/CostFigure.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { categoryName } from '$lib/domain/category';
	import { nextSort, type SortKey, sortRows } from '$lib/domain/sort';
	import { keep } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const query = keep('meals:query', '');
	const sort = keep('meals:sort', nextSort({ key: 'energyKcal', direction: 'asc' }, 'name'));

	// The meal list is a table-shaped list, so its category is one more sort.value key and one
	// more thing the filter matches, rather than a heading.
	const rows = $derived.by(() => {
		const needle = query.value.trim().toLowerCase();
		const named = data.meals.map((meal) => ({
			...meal,
			categoryName: categoryName(meal, data.categories),
		}));
		const matching = named.filter((meal) =>
			`${meal.label} ${meal.categoryName ?? ''}`.toLowerCase().includes(needle),
		);
		return sortRows(matching, sort.value);
	});

	const SORTS: { key: SortKey; label: string }[] = [
		{ key: 'name', label: 'Label' },
		{ key: 'category', label: 'Category' },
		{ key: 'energyKcal', label: 'Energy' },
		{ key: 'proteinG', label: 'Protein' },
		{ key: 'fatG', label: 'Fat' },
		{ key: 'carbohydratesG', label: 'Carbs' },
	];
</script>

<PageHeader title="Meals">
	{#snippet actions()}
		<a class="button" href="/meals/categories">Categories</a>
		<a class="button primary" href="/meals/new">New</a>
	{/snippet}
</PageHeader>

<div class="row">
	<input class="grow" type="search" placeholder="Filter by label or category" bind:value={query.value} />
	<select
		class="sort"
		aria-label="Sort by"
		value={sort.value.key}
		onchange={(event) => (sort.value = nextSort(sort.value, event.currentTarget.value as SortKey))}
	>
		{#each SORTS as option (option.key)}
			<option value={option.key}>{option.label}</option>
		{/each}
	</select>
	<button
		class="icon"
		aria-label="Reverse order"
		onclick={() => (sort.value = nextSort(sort.value, sort.value.key))}>{sort.value.direction === 'asc' ? '↑' : '↓'}</button
	>
</div>

{#if data.meals.length === 0}
	<p class="empty">No meals yet. A meal is products with serving sizes.</p>
{:else if rows.length === 0}
	<p class="empty">Nothing matches.</p>
{:else}
	<ul class="list">
		{#each rows as meal (meal.id)}
			<li>
				<a class="card list-link stack" href="/meals/{meal.id}">
					<div class="row spread">
						<strong class="grow truncate">{meal.label}</strong>
						{#if meal.categoryName}<span class="pill">{meal.categoryName}</span>{/if}
					</div>
					<div class="row spread">
						<MacroLine macros={meal.macros} />
						<CostFigure cost={meal.cost} />
					</div>
				</a>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.sort {
		width: auto;
	}
</style>
