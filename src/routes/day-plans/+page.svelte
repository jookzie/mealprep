<script lang="ts">
	import CostFigure from '$lib/components/CostFigure.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { groupByCategory } from '$lib/domain/category';
	import { itemLabel } from '$lib/domain/day-plan';
	import { formatSigned } from '$lib/domain/format';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const groups = $derived(groupByCategory(data.dayPlans, data.categories));
</script>

<PageHeader title="Day plans">
	{#snippet actions()}
		<a class="button" href="/day-plans/categories">Categories</a>
		<a class="button primary" href="/day-plans/new">New</a>
	{/snippet}
</PageHeader>

{#if data.dayPlans.length === 0}
	<p class="empty">No day plans yet. A day plan is the meals and products of one day, in order.</p>
{:else}
	{#each groups as group (group.key)}
		<section class="stack">
			{#if groups.length > 1}
				<h2 class="muted">{group.label}</h2>
			{/if}
			{#if group.items.length === 0}
				<p class="muted small">Nothing filed here yet.</p>
			{/if}
			<ul class="list">
				{#each group.items as plan (plan.id)}
					<li>
						<a class="card list-link stack" href="/day-plans/{plan.id}">
							<div class="row spread">
								<strong class="grow truncate">{plan.label}</strong>
								<CostFigure cost={plan.cost} />
							</div>
							<MacroLine macros={plan.macros} />
							{#if data.targets}
								<span class="muted small"
									>{formatSigned(plan.macros.energyKcal - data.targets.macros.energyKcal, 'kcal')} against
									the energy target</span
								>
							{/if}
							{#if plan.items.length > 0}
								<span class="muted small truncate">{plan.items.map(itemLabel).join(' · ')}</span>
							{/if}
						</a>
					</li>
				{/each}
			</ul>
		</section>
	{/each}
{/if}
