<script lang="ts">
	import TargetIcon from '@lucide/svelte/icons/target';
	import { type Macros, runMutation, setTargets } from '$lib/api';
	import PageHeader from '$lib/components/app/page-header.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';
	import TargetsForm from '$lib/components/macros/targets-form.svelte';
	import * as Card from '$lib/components/ui/card';
	import { formatKcal } from '$lib/domain/format';
	import { impliedEnergyKcal } from '$lib/domain/macros';

	let { data } = $props();

	// The API answers 404 until targets have been set for the first time; loadTargets
	// turns that into null, which is an empty form rather than an error.
	const initial = $derived<Macros>(
		data.targets?.macros ?? { energyKcal: 0, proteinG: 0, fatG: 0, carbohydratesG: 0 }
	);

	/*
	 * Four independent figures can quietly disagree: the macro grams imply an energy
	 * total of their own on the 4/9/4 factors. Saying so is the whole intervention —
	 * it is a hint, never a validation error, and nothing here is ever blocked.
	 */
	const implied = $derived(data.targets ? impliedEnergyKcal(data.targets.macros) : null);
	const drift = $derived(
		implied !== null && data.targets ? implied - data.targets.macros.energyKcal : 0
	);
	const inconsistent = $derived(Math.abs(drift) >= 50);

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
		<Card.Content class="space-y-4">
			<MacrosSummary macros={data.targets.macros} variant="grid" />
			{#if inconsistent && implied !== null}
				<p class="text-muted-foreground text-sm">
					Your protein, fat and carbs come to
					<span class="text-foreground tabular-nums">{formatKcal(implied)}</span>
					on the 4/9/4 factors, which is
					<span class="text-foreground tabular-nums">
						{formatKcal(Math.abs(drift))}
					</span>
					{drift > 0 ? 'above' : 'below'} your energy target. Both are used exactly as you set them.
				</p>
			{/if}
		</Card.Content>
	</Card.Root>
{/if}

<Card.Root>
	<Card.Header>
		<Card.Title>{data.targets ? 'Update targets' : 'Set targets'}</Card.Title>
	</Card.Header>
	<Card.Content>
		{#key data.targets?.updatedAt ?? 'unset'}
			<TargetsForm
				{initial}
				onSubmit={save}
				submitLabel={data.targets ? 'Save targets' : 'Set targets'}
			/>
		{/key}
	</Card.Content>
</Card.Root>
