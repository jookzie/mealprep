<script lang="ts" generics="T extends { id: string; label: string; categoryId?: string; macros: Macros }">
	import Check from '@lucide/svelte/icons/check';
	import type { Category, Macros } from '$lib/api';
	import { groupByCategory, nonEmptyGroups } from '$lib/domain/category';

	let {
		items,
		categories,
		selectedId,
		placeholder,
		onPick,
	}: {
		items: readonly T[];
		categories: readonly Category[];
		selectedId?: string;
		placeholder: string;
		onPick: (item: T) => void;
	} = $props();

	let query = $state('');

	const whole = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

	// Categories are headings inside the one searchable list, never a first choice.
	const groups = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const matching = items.filter((item) => item.label.toLowerCase().includes(needle));
		return nonEmptyGroups(groupByCategory(matching, categories));
	});
</script>

<div class="stack picker">
	{#if items.length > 6}
		<input type="search" {placeholder} bind:value={query} />
	{/if}
	{#if groups.length === 0}
		<p class="empty">Nothing matches.</p>
	{:else}
		{#each groups as group (group.key)}
			<section class="group">
				{#if groups.length > 1}
					<h3 class="heading">{group.label}</h3>
				{/if}
				<ul class="list">
					{#each group.items as item (item.id)}
						{@const chosen = item.id === selectedId}
						<li>
							<button type="button" class="pick" class:chosen aria-pressed={chosen} onclick={() => onPick(item)}>
								<span class="grow truncate name">{item.label}</span>
								<span class="kcal">{whole.format(item.macros.energyKcal)} kcal</span>
								<span class="tick" aria-hidden="true">
									{#if chosen}<Check size={13} strokeWidth={4} />{/if}
								</span>
							</button>
						</li>
					{/each}
				</ul>
			</section>
		{/each}
	{/if}
</div>

<style>
	.picker {
		gap: 0.9rem;
	}

	.group {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.heading {
		font-size: 0.75rem;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--muted);
	}

	.list {
		gap: 0.4rem;
	}

	.pick {
		width: 100%;
		min-height: 56px;
		border-radius: 20px;
		padding: 0 0.9rem 0 1rem;
		justify-content: flex-start;
		text-align: left;
		gap: 0.75rem;
		background: var(--surface);
	}

	.pick.chosen {
		background: var(--fat);
		color: var(--on-macro);
		border-color: var(--on-macro);
	}

	.name {
		font-weight: 700;
	}

	.kcal {
		font-size: 0.85rem;
		font-weight: 600;
		opacity: 0.8;
	}

	.tick {
		width: 24px;
		height: 24px;
		border-radius: 12px;
		border: 2px solid currentColor;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.chosen .tick {
		background: #111111;
		color: #ffffff;
		border-color: #111111;
	}
</style>
