<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import UtensilsIcon from '@lucide/svelte/icons/utensils';
	import EmptyState from '$lib/components/app/empty-state.svelte';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MealTable from '$lib/components/meal/meal-table.svelte';
	import { Button } from '$lib/components/ui/button';

	let { data } = $props();
</script>

<PageHeader title="Meals" description="Products with serving sizes. Macros are derived on read.">
	{#snippet actions()}
		<Button href="/meals/new">
			<PlusIcon class="size-4" />
			New meal
		</Button>
	{/snippet}
</PageHeader>

{#if data.meals.length === 0}
	<EmptyState
		icon={UtensilsIcon}
		title="No meals yet"
		description="A meal is a labelled set of products, each with a serving size."
	>
		{#snippet action()}
			<Button href="/meals/new">Create a meal</Button>
		{/snippet}
	</EmptyState>
{:else}
	<MealTable meals={data.meals} />
{/if}
