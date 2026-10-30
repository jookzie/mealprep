<script lang="ts">
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import { untrack } from 'svelte';
	import { pushState } from '$app/navigation';
	import {
		exportDatabase,
		importDatabase,
		resetDemoData,
		runMutation,
		setTargets,
	} from '$lib/api';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import TargetStepper from '$lib/components/TargetStepper.svelte';
	import { formatKcal } from '$lib/domain/format';
	import { impliedEnergyKcal } from '$lib/domain/macros';
	import { domain, formatKilograms, formatRate, scale, trendPath, visible } from '$lib/domain/weight';
	import { forgetOnSave, keepDraft } from '$lib/memory.svelte';
	import { toasts } from '$lib/toasts.svelte';
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
	let confirmingImport = $state(false);

	// Four independent numbers can quietly disagree; saying so informs and blocks nothing.
	const implied = $derived.by(() => {
		const { proteinG, fatG, carbohydratesG } = form;
		if (proteinG === null || fatG === null || carbohydratesG === null) return null;
		return impliedEnergyKcal({ energyKcal: 0, proteinG, fatG, carbohydratesG });
	});
	const complete = $derived(
		form.energyKcal !== null && form.proteinG !== null && form.fatG !== null && form.carbohydratesG !== null,
	);
	const changed = $derived(
		!data.targets ||
			form.energyKcal !== data.targets.macros.energyKcal ||
			form.proteinG !== data.targets.macros.proteinG ||
			form.fatG !== data.targets.macros.fatG ||
			form.carbohydratesG !== data.targets.macros.carbohydratesG,
	);

	// The last three months of the trend, drawn small: the shape, not the figures.
	const SPARK = { width: 120, height: 48, padX: 3, padY: 6 };
	const spark = $derived.by(() => {
		const points = visible(data.weight.points, 90);
		if (points.length < 2) return null;
		return trendPath(points, scale(points.length, domain(points), SPARK));
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const { energyKcal, proteinG, fatG, carbohydratesG } = form;
		if (energyKcal === null || proteinG === null || fatG === null || carbohydratesG === null) {
			return;
		}
		busy = true;
		await forgetOnSave(
			draft,
			runMutation(() => setTargets({ energyKcal, proteinG, fatG, carbohydratesG }), {
				success: 'Targets saved',
			}),
		);
		busy = false;
	}

	async function exportAll() {
		busy = true;
		const saved = await runMutation(() => exportDatabase(), { invalidate: false });
		busy = false;
		if (saved) toasts.push('success', 'Exported');
	}

	/*
	 * Like the reset below, a successful import closes the app, so only a failure comes
	 * back here — and a failure has replaced nothing.
	 */
	async function importAll() {
		confirmingImport = false;
		busy = true;
		await runMutation(() => importDatabase(), { invalidate: false });
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

<PageHeader title="You" />

<!-- Weight is what the targets exist to move, so it heads the screen they are set on. -->
<a class="weight" href="/weight">
	<span class="grow">
		<span class="small strong">Weight trend</span>
		<span class="display trend figure">
			{data.weight.trend === undefined ? 'No weigh-ins' : formatKilograms(data.weight.trend)}
		</span>
		{#if data.weight.ratePerWeek !== undefined}
			<span class="rate">{formatRate(data.weight.ratePerWeek)}</span>
		{/if}
	</span>
	{#if spark}
		<svg viewBox="0 0 {SPARK.width} {SPARK.height}" width={SPARK.width} height={SPARK.height} aria-hidden="true">
			<path d={spark} />
		</svg>
	{/if}
	<ChevronRight size={20} aria-hidden="true" />
</a>

<form class="stack" onsubmit={submit}>
	<div class="row spread">
		<h2>Daily targets</h2>
		{#if implied !== null}
			<span class="muted small strong">Macros add up to {formatKcal(implied)}</span>
		{/if}
	</div>
	<div class="targets">
		<TargetStepper label="Energy" unit="kcal" increment={50} tone="hero" bind:value={form.energyKcal} />
		<TargetStepper label="Protein" unit="g" increment={5} tone="protein" bind:value={form.proteinG} />
		<TargetStepper label="Fat" unit="g" increment={5} tone="fat" bind:value={form.fatG} />
		<TargetStepper label="Carbs" unit="g" increment={5} tone="carbs" bind:value={form.carbohydratesG} />
	</div>
	{#if changed}
		<button class="confirm wide" type="submit" disabled={busy || !complete}>Save targets</button>
	{/if}
</form>

<ul class="settings">
	<li>
		<a href="/meals/categories">
			<span class="grow strong">Meal categories</span>
			<ChevronRight size={18} aria-hidden="true" />
		</a>
	</li>
	<li>
		<a href="/day-plans/categories">
			<span class="grow strong">Day plan categories</span>
			<ChevronRight size={18} aria-hidden="true" />
		</a>
	</li>
	<li>
		<button type="button" onclick={() => pushState('', { ask: true })}>
			<span class="grow strong">Ask</span>
			<span class="muted small">
				{data.assistantKey.present
					? `Key ••••${data.assistantKey.hint ?? ''}`
					: 'Not set up'}
			</span>
			<ChevronRight size={18} aria-hidden="true" />
		</button>
	</li>
</ul>

<!--
	The database is a private file on the phone, so this is the only way it leaves or
	arrives: one file, saved or picked through the system's file picker.
-->
<section class="card stack" aria-label="Your data">
	<div>
		<h2>Your data</h2>
		<p class="small muted">
			Export saves a copy of everything wherever you choose. Import replaces everything with a
			copy you pick; what is here now is kept as a backup first, and the app closes — reopen it
			afterwards.
		</p>
	</div>
	<div class="grid-2">
		{#if confirmingImport}
			<button type="button" class="outline" onclick={() => (confirmingImport = false)}>Cancel</button>
			<button type="button" class="danger outline" disabled={busy} onclick={importAll}>
				Replace everything
			</button>
		{:else}
			<button type="button" class="outline" disabled={busy} onclick={() => (confirmingImport = true)}>
				Import
			</button>
			<button type="button" class="primary" disabled={busy} onclick={exportAll}>Export</button>
		{/if}
	</div>
</section>

<!--
	Acceptance builds only: `data.acceptance` is false whenever the reset command is not
	registered, which is every production build. It sits here because this is the
	settings-shaped screen, and behind a confirmation because it discards everything.
-->
{#if data.acceptance}
	<section class="card stack acceptance">
		<div>
			<h2>Acceptance build</h2>
			<p class="small">
				This install is for trying changes, and its data is separate from the real app's.
				Resetting replaces everything with the demo dataset and closes the app; reopen it
				afterwards.
			</p>
		</div>
		<div class="grid-2">
			{#if confirmingReset}
				<button type="button" class="outline" onclick={() => (confirmingReset = false)}>Cancel</button>
				<button type="button" class="danger outline" disabled={busy} onclick={reset}>
					Discard and reset
				</button>
			{:else}
				<button type="button" class="outline span" disabled={busy} onclick={() => (confirmingReset = true)}>
					Reset to demo data
				</button>
			{/if}
		</div>
	</section>
{/if}

<style>
	.strong {
		font-weight: 700;
	}

	.weight {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 1rem 1.1rem;
		border: 2px solid var(--text);
		border-radius: var(--radius-lg);
		text-decoration: none;
	}

	.weight .grow {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
	}

	.trend {
		font-size: 2rem;
		line-height: 1.1;
	}

	.rate {
		margin-top: 0.3rem;
		font-size: 0.8rem;
		font-weight: 700;
		padding: 0.2rem 0.6rem;
		border-radius: 10px;
		background: var(--carbs);
		color: var(--on-macro);
	}

	svg path {
		fill: none;
		stroke: var(--text);
		stroke-width: 3;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.wide {
		width: 100%;
		min-height: 52px;
	}

	.targets {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.span {
		grid-column: 1 / -1;
	}

	.settings {
		list-style: none;
		margin: 0;
		padding: 0;
		background: var(--surface);
		border-radius: var(--radius);
		overflow: hidden;
	}

	.settings li + li {
		border-top: 1.5px solid var(--border);
	}

	.settings a,
	.settings button {
		width: 100%;
		min-height: 56px;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0 1rem;
		border-radius: 0;
		background: transparent;
		text-decoration: none;
		font-weight: 500;
		text-align: left;
	}

	.acceptance {
		background: var(--warning-bg);
		color: #111111;
	}

	.acceptance .outline {
		border-color: #111111;
		color: #111111;
	}
</style>
