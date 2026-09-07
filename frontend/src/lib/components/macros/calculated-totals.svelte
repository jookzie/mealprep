<script lang="ts">
	import type { Macros } from '$lib/api';
	import MacroMeters from '$lib/components/macros/macro-meters.svelte';
	import MacrosSummary from '$lib/components/macros/macros-summary.svelte';

	/*
	 * The third zone of a composition editor: what the rows above add up to.
	 *
	 * Labelled "Calculated" and drawn as read-only text on purpose. Every recipe editor
	 * surveyed derives these from the ingredients and none lets you type them, and
	 * saying so is what stops someone hunting for the edit button that is not there.
	 */
	let {
		macros,
		target = null,
		empty = false
	}: { macros: Macros; target?: Macros | null; empty?: boolean } = $props();
</script>

<div class="bg-muted/30 space-y-4 rounded-lg border p-4">
	<div>
		<h2 class="text-sm font-medium">Calculated</h2>
		<p class="text-muted-foreground text-xs">Summed from the rows. Not editable.</p>
	</div>

	{#if empty}
		<p class="text-muted-foreground text-sm">Nothing added yet.</p>
	{:else}
		<MacrosSummary {macros} variant="pair" />
		{#if target}
			<div class="border-t pt-4">
				<MacroMeters actual={macros} {target} />
			</div>
		{/if}
	{/if}
</div>
