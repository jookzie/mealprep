<script lang="ts">
	import Plus from '@lucide/svelte/icons/plus';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import LibraryTools from '$lib/components/LibraryTools.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SectionSwitch from '$lib/components/SectionSwitch.svelte';
	import { categoryName, REMOVED_KEY, UNCATEGORISED_KEY } from '$lib/domain/category';
	import { nextSort, type SortKey, sortRows } from '$lib/domain/sort';
	import { keep } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const query = keep('meals:query', '');
	const sort = keep('meals:sort', nextSort({ key: 'energyKcal', direction: 'asc' }, 'name'));
	const category = keep<string>('meals:category', 'all');

	const known = $derived(new Set(data.categories.map((entry) => entry.id)));

	function inCategory(categoryId: string | undefined): boolean {
		if (category.value === 'all') return true;
		if (category.value === UNCATEGORISED_KEY) return categoryId === undefined;
		if (category.value === REMOVED_KEY) return categoryId !== undefined && !known.has(categoryId);
		return categoryId === category.value;
	}

	// The category is a chip to narrow by and one more sort key, rather than a heading.
	const rows = $derived.by(() => {
		const needle = query.value.trim().toLowerCase();
		const named = data.meals.map((meal) => ({
			...meal,
			categoryName: categoryName(meal, data.categories),
		}));
		const matching = named.filter(
			(meal) =>
				inCategory(meal.categoryId) &&
				`${meal.label} ${meal.categoryName ?? ''}`.toLowerCase().includes(needle),
		);
		return sortRows(matching, sort.value);
	});

	const SORTS: { key: SortKey; label: string }[] = [
		{ key: 'name', label: 'Name' },
		{ key: 'category', label: 'Category' },
		{ key: 'energyKcal', label: 'Energy' },
		{ key: 'proteinG', label: 'Protein' },
		{ key: 'fatG', label: 'Fat' },
		{ key: 'carbohydratesG', label: 'Carbs' },
	];
</script>

<PageHeader title="Library">
	{#snippet actions()}
		<a class="button primary" href="/meals/new">
			<Plus size={18} strokeWidth={2.6} aria-hidden="true" />
			New meal
		</a>
	{/snippet}
</PageHeader>

<SectionSwitch
	label="Library"
	current="/meals"
	links={[
		{ href: '/meals', label: `Meals · ${data.meals.length}` },
		{ href: '/products', label: 'Products' },
	]}
/>

<LibraryTools placeholder="Search meals" sorts={SORTS} bind:query={query.value} bind:sort={sort.value} />

{#if data.categories.length > 0}
	<div class="chips" role="group" aria-label="Category">
		<button class="chip" aria-pressed={category.value === 'all'} onclick={() => (category.value = 'all')}>All</button>
		{#each data.categories as entry (entry.id)}
			<button class="chip" aria-pressed={category.value === entry.id} onclick={() => (category.value = entry.id)}
				>{entry.name}</button
			>
		{/each}
		<button
			class="chip"
			aria-pressed={category.value === UNCATEGORISED_KEY}
			onclick={() => (category.value = UNCATEGORISED_KEY)}>None</button
		>
		<a class="chip button manage" href="/meals/categories">Edit categories</a>
	</div>
{/if}

{#if data.meals.length === 0}
	<p class="empty">No meals yet. A meal is products with serving sizes.</p>
{:else if rows.length === 0}
	<p class="empty">Nothing matches.</p>
{:else}
	<ul class="list">
		{#each rows as meal (meal.id)}
			<li>
				<a class="card list-link entry" href="/meals/{meal.id}">
					<div class="row spread top">
						<strong class="grow truncate">{meal.label}</strong>
						<span class="display kcal figure">{Math.round(meal.macros.energyKcal)}<small> kcal</small></span>
					</div>
					<div class="row spread">
						<MacroLine macros={meal.macros} energy={false} />
						<span class="row meta">
							{#if meal.categoryName}<span class="muted small strong">{meal.categoryName}</span>{/if}
							<CostFigure cost={meal.cost} />
						</span>
					</div>
				</a>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.entry {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.top {
		flex-wrap: nowrap;
		align-items: baseline;
	}

	.kcal {
		font-size: 1.15rem;
		white-space: nowrap;
	}

	.kcal small {
		font-family: var(--font-body);
		font-size: 0.75rem;
		font-weight: 600;
	}

	.meta {
		gap: 0.5rem;
	}

	.strong {
		font-weight: 700;
	}

	.manage {
		background: transparent;
		border: 2px dashed var(--border);
	}
</style>
