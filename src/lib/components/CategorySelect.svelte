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

<label class="field">
	<span>Category</span>
	<select
		value={value ?? ''}
		onchange={(event) => {
			const selected = event.currentTarget.value;
			value = selected === '' ? undefined : selected;
		}}
	>
		<option value="">Uncategorised</option>
		{#if removed}
			<option value={value}>{REMOVED_LABEL}</option>
		{/if}
		{#each categories as category (category.id)}
			<option value={category.id}>{category.name}</option>
		{/each}
	</select>
</label>
