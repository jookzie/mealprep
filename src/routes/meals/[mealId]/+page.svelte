<script lang="ts">
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Pencil from '@lucide/svelte/icons/pencil';
	import { deleteMeal, runMutation } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
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

<PageHeader title={meal.label} eyebrow={categoryName(meal, data.categories)} back="/meals" backLabel="Library">
	{#snippet actions()}
		<DeleteButton
			what={meal.label}
			consequence="Day plans that hold it drop it from their totals."
			onConfirm={() =>
				runMutation(() => deleteMeal(meal.id), { success: 'Meal deleted', redirectTo: '/meals' })}
		/>
		<a class="button primary" href="/meals/{meal.id}/edit">
			<Pencil size={16} strokeWidth={2.6} aria-hidden="true" />
			Edit
		</a>
	{/snippet}
</PageHeader>

<MacroMeters actual={meal.macros} target={null} caption="kcal · {rows.length} {rows.length === 1 ? 'product' : 'products'}">
	{#snippet aside()}
		<span class="cost"><CostFigure cost={meal.cost} /></span>
	{/snippet}
</MacroMeters>

<h2>In this meal</h2>
{#if rows.length === 0}
	<p class="empty">No products.</p>
{:else}
	<ul class="servings">
		{#each rows as row, index (`${row.productId}-${index}`)}
			<li>
				{#if row.product}
					<a href="/products/{row.productId}">
						<span class="grow">
							<strong class="truncate">{row.product.name}</strong>
							<span class="muted small">
								{formatAmount(row.amount, row.unit)} · <CostFigure cost={row.cost} />
							</span>
						</span>
						<span class="figure strong">{Math.round(row.macros.energyKcal)}</span>
						<ChevronRight size={18} aria-hidden="true" />
					</a>
				{:else}
					<span class="removed">
						<span class="grow">
							<strong class="muted">Product removed</strong>
							<span class="muted small">{formatAmount(row.amount, row.unit)}</span>
						</span>
					</span>
				{/if}
			</li>
		{/each}
	</ul>
{/if}

<style>
	.cost {
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: rgb(127 127 127 / 0.22);
	}

	.servings {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.servings a,
	.removed {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0.7rem 0;
		border-bottom: 1.5px solid var(--border);
		text-decoration: none;
	}

	.servings .grow {
		display: flex;
		flex-direction: column;
	}

	.strong {
		font-weight: 700;
	}
</style>
