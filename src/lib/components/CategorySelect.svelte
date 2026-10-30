<script lang="ts">
	import type { Category } from '$lib/api';
	import { REMOVED_LABEL } from '$lib/domain/category';

	let {
		categories,
		value = $bindable(),
	}: { categories: readonly Category[]; value: string | undefined } = $props();

	const removed = $derived(
		value !== undefined && !categories.some((category) => category.id === value),
	);
</script>

<!-- A handful of categories at most, so they are all on show as chips rather than in a menu. -->
<fieldset class="field">
	<legend>Category</legend>
	<div class="chips">
		<button type="button" class="chip" aria-pressed={value === undefined} onclick={() => (value = undefined)}>
			None
		</button>
		{#if removed}
			<button type="button" class="chip" aria-pressed="true">{REMOVED_LABEL}</button>
		{/if}
		{#each categories as category (category.id)}
			<button
				type="button"
				class="chip"
				aria-pressed={value === category.id}
				onclick={() => (value = category.id)}>{category.name}</button
			>
		{/each}
	</div>
</fieldset>

<style>
	legend {
		font-weight: 700;
		font-size: 0.85rem;
		margin-bottom: 0.3rem;
	}

	.chip[aria-pressed='true'] {
		background: var(--fat);
		color: var(--on-macro);
		border-color: var(--on-macro);
	}
</style>
