<script lang="ts">
	import Pencil from '@lucide/svelte/icons/pencil';
	import { deleteDayPlan, runMutation } from '$lib/api';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import DeleteButton from '$lib/components/DeleteButton.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import MacroMeters from '$lib/components/MacroMeters.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { categoryName } from '$lib/domain/category';
	import { itemLabel } from '$lib/domain/day-plan';
	import { formatAmount } from '$lib/domain/format';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const plan = $derived(data.dayPlan);
</script>

<PageHeader
	title={plan.label}
	eyebrow={plan.sourcePlanId ? 'One day’s own copy' : categoryName(plan, data.categories)}
	back={plan.sourcePlanId ? '/calendar' : '/day-plans'}
	backLabel={plan.sourcePlanId ? 'Calendar' : 'Day plans'}
>
	{#snippet actions()}
		<DeleteButton
			what={plan.label}
			consequence="Days it was assigned to read as unplanned."
			onConfirm={() =>
				runMutation(() => deleteDayPlan(plan.id), {
					success: 'Day plan deleted',
					redirectTo: '/day-plans',
				})}
		/>
		<a class="button primary" href="/day-plans/{plan.id}/edit">
			<Pencil size={16} strokeWidth={2.6} aria-hidden="true" />
			Edit
		</a>
	{/snippet}
</PageHeader>

<MacroMeters actual={plan.macros} target={data.targets?.macros ?? null} caption="kcal planned">
	{#snippet aside()}
		<span class="cost"><CostFigure cost={plan.cost} /></span>
	{/snippet}
</MacroMeters>
{#if !data.targets}
	<p class="muted small"><a href="/you">Set daily targets</a> to see how this day compares.</p>
{/if}

<h2>In order</h2>
{#if plan.items.length === 0}
	<p class="empty">Nothing in this plan yet.</p>
{:else}
	<ol class="items">
		{#each plan.items as item, index (index)}
			{@const macros = item.kind === 'meal' ? item.meal.macros : item.product.macros}
			<li class:product={item.kind === 'product'}>
				<span class="display index">{index + 1}</span>
				<span class="grow body">
					{#if item.kind === 'meal'}
						<a class="truncate strong" href="/meals/{item.meal.id}">{itemLabel(item)}</a>
					{:else if item.product.name}
						<a class="truncate strong" href="/products/{item.product.id}">{itemLabel(item)}</a>
					{:else}
						<span class="truncate strong muted">{itemLabel(item)}</span>
					{/if}
					<span class="row small detail">
						{#if item.kind === 'meal'}Meal{:else}{formatAmount(item.product.amount, item.product.unit)}{/if}
						· <MacroLine macros={macros} energy={false} />
					</span>
				</span>
				<span class="kcal figure">{Math.round(macros.energyKcal)}</span>
			</li>
		{/each}
	</ol>
{/if}

<style>
	.cost {
		padding: 0.3rem 0.75rem;
		border-radius: 999px;
		background: rgb(127 127 127 / 0.22);
	}

	.items {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.items li {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 16px;
		border-radius: var(--radius);
		background: var(--surface);
	}

	.items li.product {
		background: color-mix(in srgb, var(--protein) 28%, var(--surface));
	}

	.index {
		font-size: 1.5rem;
		width: 1.3rem;
	}

	.body {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.body a {
		text-decoration: none;
	}

	.strong {
		font-weight: 700;
	}

	.detail {
		gap: 0.3rem;
		color: var(--muted);
		font-weight: 600;
	}

	.kcal {
		font-weight: 700;
		padding: 0.35rem 0.6rem;
		border-radius: 12px;
		background: var(--raised);
	}
</style>
