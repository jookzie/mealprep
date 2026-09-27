<script lang="ts">
	import { deleteMeal, runMutation } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { categoryName } from '$lib/domain/category';
	import { formatAmount } from '$lib/domain/format';
	import { servingRows } from '$lib/domain/meal';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const meal = $derived(data.meal);
	const rows = $derived(servingRows(meal.servings, data.products));
</script>

<PageHeader title={meal.label} subtitle={categoryName(meal, data.categories)} back="/meals">
	{#snippet actions()}
		<a class="button" href="/meals/{meal.id}/edit">Edit</a>
		<DeleteButton
			what={meal.label}
			consequence="Day plans that hold it drop it from their totals."
			onConfirm={() =>
				runMutation(() => deleteMeal(meal.id), { success: 'Meal deleted', redirectTo: '/meals' })}
		/>
	{/snippet}
</PageHeader>

<section class="card stack">
	<div class="row spread">
		<h2>Calculated</h2>
		<CostFigure cost={meal.cost} label="Cost" />
	</div>
	<MacroMeters actual={meal.macros} target={null} />
</section>

<section class="card stack">
	<h2>Servings</h2>
	{#if rows.length === 0}
		<p class="muted">No products.</p>
	{:else}
		<ul class="list">
			{#each rows as row, index (`${row.productId}-${index}`)}
				<li class="stack serving">
					<div class="row spread">
						{#if row.product}
							<a class="grow truncate" href="/products/{row.productId}">{row.product.name}</a>
						{:else}
							<span class="grow muted">Product removed</span>
						{/if}
						<span class="small">{formatAmount(row.amount, row.unit)}</span>
					</div>
					<div class="row spread">
						<MacroLine macros={row.macros} />
						<CostFigure cost={row.cost} />
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.serving + .serving {
		border-top: 1px solid var(--border);
		padding-top: 0.5rem;
	}
</style>
