<script lang="ts">
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

<PageHeader title={plan.label} subtitle={categoryName(plan, data.categories)} back="/day-plans">
	{#snippet actions()}
		<a class="button" href="/day-plans/{plan.id}/edit">Edit</a>
		<DeleteButton
			what={plan.label}
			consequence="Days it was assigned to read as unplanned."
			onConfirm={() =>
				runMutation(() => deleteDayPlan(plan.id), {
					success: 'Day plan deleted',
					redirectTo: '/day-plans',
				})}
		/>
	{/snippet}
</PageHeader>

<section class="card stack">
	<div class="row spread">
		<h2>Against targets</h2>
		<CostFigure cost={plan.cost} label="Cost" />
	</div>
	<MacroMeters actual={plan.macros} target={data.targets?.macros ?? null} />
	{#if !data.targets}
		<p class="muted small"><a href="/targets">Set daily targets</a> to see how this day compares.</p>
	{/if}
</section>

<section class="card stack">
	<h2>In order</h2>
	{#if plan.items.length === 0}
		<p class="muted">Nothing in this plan yet.</p>
	{:else}
		<ol class="list">
			{#each plan.items as item, index (index)}
				<li class="stack item">
					<div class="row spread">
						{#if item.kind === 'meal'}
							<a class="grow truncate" href="/meals/{item.meal.id}">{itemLabel(item)}</a>
							<span class="pill">Meal</span>
						{:else}
							<span class="grow truncate" class:muted={!item.product.name}>{itemLabel(item)}</span>
							<span class="small">{formatAmount(item.product.amount, item.product.unit)}</span>
						{/if}
					</div>
					<div class="row spread">
						<MacroLine macros={item.kind === 'meal' ? item.meal.macros : item.product.macros} />
						<CostFigure cost={item.kind === 'meal' ? item.meal.cost : item.product.cost} />
					</div>
				</li>
			{/each}
		</ol>
	{/if}
</section>

<style>
	.item + .item {
		border-top: 1px solid var(--border);
		padding-top: 0.5rem;
	}
</style>
