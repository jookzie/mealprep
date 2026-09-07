<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import XIcon from '@lucide/svelte/icons/x';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { duplicateKeys, newPair, type NutrientPair } from '$lib/domain/nutrient-pairs';

	// Whatever nutrients the source carried are kept (PR-6), so the vocabulary is open
	// and edited as free key/value pairs rather than a fixed set of fields.
	let { pairs = $bindable(), unit }: { pairs: NutrientPair[]; unit: string } = $props();

	const duplicates = $derived(duplicateKeys(pairs));

	function add() {
		pairs = [...pairs, newPair()];
	}

	function remove(id: string) {
		pairs = pairs.filter((pair) => pair.id !== id);
	}
</script>

<div class="space-y-3">
	<div class="flex items-center justify-between">
		<Label>Other nutrients <span class="text-muted-foreground">per 100 {unit}</span></Label>
		<Button type="button" variant="outline" size="sm" onclick={add}>
			<PlusIcon class="size-4" />
			Add nutrient
		</Button>
	</div>

	{#if pairs.length === 0}
		<p class="text-muted-foreground text-sm">
			Optional. Anything added here is shown behind the expansion on the product.
		</p>
	{:else}
		<ul class="space-y-2">
			{#each pairs as pair, index (pair.id)}
				<li class="flex items-start gap-2">
					<div class="flex-1">
						<Input
							placeholder="Nutrient, e.g. saturated fat"
							bind:value={pairs[index].key}
							aria-label="Nutrient name"
							aria-invalid={duplicates.has(pair.key.trim().toLowerCase())}
						/>
					</div>
					<div class="w-32">
						<Input
							type="text"
							inputmode="decimal"
							placeholder="0"
							bind:value={pairs[index].value}
							aria-label="Nutrient value"
							class="tabular-nums"
						/>
					</div>
					<Button
						type="button"
						variant="ghost"
						size="icon"
						onclick={() => remove(pair.id)}
						aria-label="Remove nutrient"
					>
						<XIcon class="size-4" />
					</Button>
				</li>
			{/each}
		</ul>
		{#if duplicates.size > 0}
			<p class="text-destructive text-sm">Each nutrient can only be listed once.</p>
		{/if}
	{/if}
</div>
