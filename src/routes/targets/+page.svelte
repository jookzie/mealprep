<script lang="ts">
	import { untrack } from 'svelte';
	import { resetDemoData, runMutation, setTargets } from '$lib/api';
	import NumberField from '$lib/components/NumberField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { formatKcal } from '$lib/domain/format';
	import { impliedEnergyKcal } from '$lib/domain/macros';
	import { formatKilograms, formatRate } from '$lib/domain/weight';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	type Figures = {
		energyKcal: number | null;
		proteinG: number | null;
		fatG: number | null;
		carbohydratesG: number | null;
	};

	// Kept against the targets it was typed over, so a save starts the next visit clean.
	const draft = untrack(() => keepDraft<Figures>(`targets:${data.targets?.updatedAt ?? 'none'}`));
	const seed = untrack(() => data.targets?.macros);
	draft.value ??= {
		energyKcal: seed?.energyKcal ?? null,
		proteinG: seed?.proteinG ?? null,
		fatG: seed?.fatG ?? null,
		carbohydratesG: seed?.carbohydratesG ?? null,
	};
	const form = draft.value;
	let busy = $state(false);
	let confirmingReset = $state(false);

	// Four independent numbers can quietly disagree; saying so informs and blocks nothing.
	const implied = $derived.by(() => {
		const { proteinG, fatG, carbohydratesG } = form;
		if (proteinG === null || fatG === null || carbohydratesG === null) return null;
		return impliedEnergyKcal({ energyKcal: 0, proteinG, fatG, carbohydratesG });
	});
	const disagrees = $derived(
		implied !== null &&
			form.energyKcal !== null &&
			Math.abs(implied - form.energyKcal) > form.energyKcal * 0.05,
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const { energyKcal, proteinG, fatG, carbohydratesG } = form;
		if (energyKcal === null || proteinG === null || fatG === null || carbohydratesG === null) {
			return;
		}
		const macros = { energyKcal, proteinG, fatG, carbohydratesG };
		busy = true;
		await forgetOnSave(draft, runMutation(() => setTargets(macros), { success: 'Targets saved' }));
		busy = false;
	}

	/*
	 * Acceptance only. The app closes as part of the reset, so there is nothing to
	 * invalidate and nothing to toast afterwards: runMutation is here for the failure
	 * path, which is the only path that comes back.
	 */
	async function reset() {
		confirmingReset = false;
		busy = true;
		await runMutation(() => resetDemoData(), { invalidate: false });
		busy = false;
	}
</script>

<PageHeader title="Daily targets" subtitle="What plans and the calendar are measured against" />

<form class="card stack" onsubmit={submit}>
	<div class="grid-2">
		<NumberField label="Energy" suffix="kcal" bind:value={form.energyKcal} required />
		<NumberField label="Protein" suffix="g" bind:value={form.proteinG} required />
		<NumberField label="Fat" suffix="g" bind:value={form.fatG} required />
		<NumberField label="Carbs" suffix="g" bind:value={form.carbohydratesG} required />
	</div>
	{#if disagrees && implied !== null && form.energyKcal !== null}
		<p class="small muted">
			Protein, fat and carbs add up to {formatKcal(implied)}, while the energy target is
			{formatKcal(form.energyKcal)}. Nothing is blocked by this.
		</p>
	{/if}
	<div class="row end">
		<button
			class="primary"
			type="submit"
			disabled={busy ||
				form.energyKcal === null ||
				form.proteinG === null ||
				form.fatG === null ||
				form.carbohydratesG === null}>Save</button
		>
	</div>
</form>

<!--
	Weight is the other thing plans are measured against, so it sits with the targets rather
	than taking a sixth tab: both platforms' guidelines cap a bottom bar at five.
-->
<a class="card list-link row spread" href="/weight">
	<div>
		<strong>Weight</strong>
		<span class="muted small">
			{#if data.weight.trend === undefined}
				· nothing logged yet
			{:else}
				· trend {formatKilograms(data.weight.trend)}
			{/if}
		</span>
	</div>
	<span class="small muted">
		{data.weight.ratePerWeek === undefined ? '' : formatRate(data.weight.ratePerWeek)} →
	</span>
</a>

<!--
	Acceptance builds only: `data.acceptance` is false whenever the reset command is not
	registered, which is every production build. It sits with the targets because this is
	the settings-shaped screen, and behind a confirmation because it discards everything.
-->
{#if data.acceptance}
	<section class="card stack">
		<div>
			<strong>Acceptance build</strong>
			<p class="small muted">
				This install is for trying changes, and its data is separate from the real app's.
				Resetting replaces everything with the demo dataset and closes the app; reopen it
				afterwards.
			</p>
		</div>
		<div class="row end">
			{#if confirmingReset}
				<button type="button" class="ghost" onclick={() => (confirmingReset = false)}>
					Cancel
				</button>
				<button type="button" class="danger" disabled={busy} onclick={reset}>
					Discard and reset
				</button>
			{:else}
				<button type="button" disabled={busy} onclick={() => (confirmingReset = true)}>
					Reset to demo data
				</button>
			{/if}
		</div>
	</section>
{/if}
