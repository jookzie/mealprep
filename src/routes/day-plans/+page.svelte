<script lang="ts">
	import Plus from '@lucide/svelte/icons/plus';
	import CostFigure from '$lib/components/CostFigure.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SectionSwitch from '$lib/components/SectionSwitch.svelte';
	import { groupByCategory } from '$lib/domain/category';
	import { itemLabel } from '$lib/domain/day-plan';
	import { formatSigned } from '$lib/domain/format';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const groups = $derived(groupByCategory(data.dayPlans, data.categories));
</script>

<PageHeader title="Plan">
	{#snippet actions()}
		<a class="button primary" href="/day-plans/new">
			<Plus size={18} strokeWidth={2.6} aria-hidden="true" />
			New day plan
		</a>
	{/snippet}
</PageHeader>

<SectionSwitch
	label="Plan"
	current="/day-plans"
	links={[
		{ href: '/calendar', label: 'Calendar' },
		{ href: '/day-plans', label: `Day plans · ${data.dayPlans.length}` },
	]}
/>

{#if data.dayPlans.length === 0}
	<p class="empty">No day plans yet. A day plan is the meals and products of one day, in order.</p>
{:else}
	{#each groups as group (group.key)}
		<section class="stack group">
			{#if groups.length > 1}
				<h2 class="heading">{group.label}</h2>
			{/if}
			{#if group.items.length === 0}
				<p class="muted small">Nothing filed here yet.</p>
			{/if}
			<ul class="list">
				{#each group.items as plan (plan.id)}
					<li>
						<a class="card list-link entry" href="/day-plans/{plan.id}">
							<div class="row spread top">
								<strong class="grow truncate">{plan.label}</strong>
								<span class="display kcal figure">{Math.round(plan.macros.energyKcal)}<small> kcal</small></span>
							</div>
							{#if plan.items.length > 0}
								<span class="muted small truncate">{plan.items.map(itemLabel).join(' · ')}</span>
							{/if}
							<div class="row spread">
								<MacroLine macros={plan.macros} energy={false} />
								<span class="row meta">
									{#if data.targets}
										<span class="small strong delta"
											>{formatSigned(plan.macros.energyKcal - data.targets.macros.energyKcal, 'kcal')}</span
										>
									{/if}
									<CostFigure cost={plan.cost} />
								</span>
							</div>
						</a>
					</li>
				{/each}
			</ul>
		</section>
	{/each}
	<a class="button outline manage" href="/day-plans/categories">Edit categories</a>
{/if}

<style>
	.group {
		gap: 0.5rem;
	}

	.heading {
		font-size: 1.1rem;
	}

	.entry {
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
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

	.delta {
		color: var(--muted);
	}

	.manage {
		align-self: flex-start;
	}
</style>
