<script lang="ts">
	import { untrack } from 'svelte';
	import type { Macros } from '$lib/api';
	import NumberField from '$lib/components/app/number-field.svelte';
	import { Button } from '$lib/components/ui/button';

	let {
		initial,
		submitLabel = 'Save targets',
		onSubmit
	}: {
		initial: Macros;
		submitLabel?: string;
		onSubmit: (macros: Macros) => Promise<void>;
	} = $props();

	// Seeded once, deliberately: re-syncing from the prop would wipe what the user is
	// typing every time a mutation invalidates the loaded data. The page re-seeds by
	// keying this component on the entity instead.
	const seed = untrack(() => initial);
	let energyKcal = $state<number | null>(seed.energyKcal);
	let proteinG = $state<number | null>(seed.proteinG);
	let fatG = $state<number | null>(seed.fatG);
	let carbohydratesG = $state<number | null>(seed.carbohydratesG);
	let pending = $state(false);

	const complete = $derived(
		energyKcal !== null && proteinG !== null && fatG !== null && carbohydratesG !== null
	);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!complete) return;
		pending = true;
		await onSubmit({
			energyKcal: energyKcal as number,
			proteinG: proteinG as number,
			fatG: fatG as number,
			carbohydratesG: carbohydratesG as number
		});
		pending = false;
	}
</script>

<form class="space-y-6" onsubmit={submit}>
	<div class="grid gap-4 sm:grid-cols-2">
		<NumberField id="energyKcal" label="Energy" suffix="kcal" bind:value={energyKcal} required />
		<NumberField id="proteinG" label="Protein" suffix="g" bind:value={proteinG} required />
		<NumberField id="fatG" label="Fat" suffix="g" bind:value={fatG} required />
		<NumberField id="carbohydratesG" label="Carbs" suffix="g" bind:value={carbohydratesG} required />
	</div>
	<Button type="submit" disabled={pending || !complete}>
		{pending ? 'Saving…' : submitLabel}
	</Button>
</form>
