<script lang="ts" generics="T extends { id: string; label: string; categoryId?: string; macros: Macros }">
	import type { Category, Macros } from '$lib/api';
	import { groupByCategory, nonEmptyGroups } from '$lib/domain/category';
	import MacroLine from './MacroLine.svelte';

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

	// Categories are headings inside the one searchable list, never a first choice.
	const groups = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const matching = items.filter((item) => item.label.toLowerCase().includes(needle));
		return nonEmptyGroups(groupByCategory(matching, categories));
	});
</script>

<div class="stack">
	<input type="search" {placeholder} bind:value={query} />
	{#if groups.length === 0}
		<p class="empty">Nothing matches.</p>
	{:else}
		<div class="picker stack">
			{#each groups as group (group.key)}
				<section class="stack">
					{#if groups.length > 1}
						<h3 class="muted small">{group.label}</h3>
					{/if}
					<ul class="list">
						{#each group.items as item (item.id)}
							<li>
								<button
									type="button"
									class="pick"
									class:selected={item.id === selectedId}
									onclick={() => onPick(item)}
								>
									<strong>{item.label}</strong>
									<MacroLine macros={item.macros} />
								</button>
							</li>
						{/each}
					</ul>
				</section>
			{/each}
		</div>
	{/if}
</div>

<style>
	.picker {
		max-height: 55dvh;
		overflow-y: auto;
	}

	.pick {
		width: 100%;
		flex-direction: column;
		align-items: flex-start;
		text-align: left;
		background: var(--surface);
	}

	.selected {
		border-color: var(--accent);
		box-shadow: inset 0 0 0 1px var(--accent);
	}
</style>
