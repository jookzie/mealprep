<script lang="ts">
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { deleteMeal } from '$lib/api';
	import DeleteAction from '$lib/components/app/delete-action.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import ServingTable from '$lib/components/meal/serving-table.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { servingRows } from '$lib/domain/meal';

	let { data } = $props();

	const rows = $derived(servingRows(data.meal.servings, data.products));
	const missing = $derived(rows.filter((row) => row.product === undefined).length);
</script>

<PageHeader title={data.meal.label}>
	{#snippet actions()}
		<Button href="/meals/{data.meal.id}/edit" variant="outline">
			<PencilIcon class="size-4" />
			Edit
		</Button>
		<DeleteAction
			name={data.meal.label}
			variant="outline"
			size="default"
			description="The meal is removed. Day plans that use it will show it as removed."
			action={() => deleteMeal({ path: { mealId: data.meal.id }, throwOnError: true })}
			result={{ redirectTo: '/meals' }}
		/>
	{/snippet}
</PageHeader>

<Card.Root>
	<Card.Header>
		<Card.Title>Macros</Card.Title>
		<Card.Description>Derived from the servings below.</Card.Description>
	</Card.Header>
	<Card.Content>
		<MacrosSummary macros={data.meal.macros} variant="grid" />
	</Card.Content>
</Card.Root>

<Card.Root>
	<Card.Header>
		<Card.Title>Servings</Card.Title>
		{#if missing > 0}
			<Card.Description>
				{missing}
				{missing === 1 ? 'product has' : 'products have'} been deleted. Edit the meal to replace
				{missing === 1 ? 'it' : 'them'}.
			</Card.Description>
		{/if}
	</Card.Header>
	<Card.Content>
		{#if rows.length === 0}
			<p class="text-muted-foreground text-sm">This meal has no servings.</p>
		{:else}
			<ServingTable {rows} />
		{/if}
	</Card.Content>
</Card.Root>
