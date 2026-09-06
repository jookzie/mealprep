<script lang="ts">
	import TargetIcon from '@lucide/svelte/icons/target';
	import { type Macros, runMutation, setTargets } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import TargetsForm from '$lib/components/macros/targets-form.svelte';
	import * as Card from '$lib/components/ui/card';

	let { data } = $props();

	// The API answers 404 until targets have been set for the first time; loadTargets
	// turns that into null, which is an empty form rather than an error.
	const initial = $derived<Macros>(
		data.targets?.macros ?? { energyKcal: 0, proteinG: 0, fatG: 0, carbohydratesG: 0 }
	);

	async function save(macros: Macros) {
		await runMutation(() => setTargets({ body: { macros }, throwOnError: true }), {
			success: 'Targets saved'
		});
	}
</script>

<PageHeader
	title="Daily targets"
	description="The figures every plan is measured against. Nothing is ever blocked for missing them."
/>

{#if !data.targets}
	<Card.Root>
		<Card.Header>
			<Card.Title class="flex items-center gap-2">
				<TargetIcon class="size-4" />
				No targets set yet
			</Card.Title>
			<Card.Description>
				Set your daily energy, protein, fat and carbohydrates to see plans measured against them.
			</Card.Description>
		</Card.Header>
	</Card.Root>
{:else}
	<Card.Root>
		<Card.Header>
			<Card.Title>Current targets</Card.Title>
		</Card.Header>
		<Card.Content>
			<MacrosSummary macros={data.targets.macros} variant="grid" />
		</Card.Content>
	</Card.Root>
{/if}

<Card.Root>
	<Card.Header>
		<Card.Title>{data.targets ? 'Update targets' : 'Set targets'}</Card.Title>
	</Card.Header>
	<Card.Content>
		{#key data.targets?.updatedAt ?? 'unset'}
			<TargetsForm {initial} onSubmit={save} submitLabel={data.targets ? 'Save targets' : 'Set targets'} />
		{/key}
	</Card.Content>
</Card.Root>
