<script lang="ts">
	import DownloadIcon from '@lucide/svelte/icons/download';
	import type { CatalogEntry } from '$lib/api';
	import MacroStrip from '$lib/components/macros/macro-strip.svelte';
	import NutrientPanel from '$lib/components/macros/nutrient-panel.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Item from '$lib/components/ui/item';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import CatalogThumbnail from './catalog-thumbnail.svelte';

	/*
	 * Search results are result rows, not a table — a table with a collapsible nested
	 * inside one of its cells is neither.
	 *
	 * The thumbnail's licence and fallback story live in catalog-thumbnail.
	 */
	let {
		entries,
		importing,
		onImport
	}: {
		entries: CatalogEntry[];
		importing: string | null;
		onImport: (code: string) => void;
	} = $props();
</script>

<Item.Group class="gap-2">
	{#each entries as entry (entry.code)}
		<Item.Root variant="outline" class="items-start">
			<Item.Media variant="image">
				<CatalogThumbnail src={entry.imageUrl} />
			</Item.Media>

			<Item.Content>
				<Item.Title>{entry.name || 'Unnamed entry'}</Item.Title>
				<Item.Description>
					{#if entry.brand}<span class="text-foreground">{entry.brand}</span> · {/if}per 100
					{entry.unit} · {entry.code}
				</Item.Description>
				<div class="mt-1">
					<NutrientPanel nutrients={entry.nutrients} unit={entry.unit} />
				</div>
			</Item.Content>

			<MacroStrip macros={entry.macros} />

			<Item.Actions>
				{#if entry.complete}
					<Button size="sm" disabled={importing !== null} onclick={() => onImport(entry.code)}>
						<DownloadIcon class="size-4" />
						{importing === entry.code ? 'Importing…' : 'Import'}
					</Button>
				{:else}
					<!-- The API rejects these with a 422; saying so up front beats letting the
					     user find out by clicking. -->
					<Tooltip.Provider>
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<span {...props} class="inline-block">
										<Button size="sm" disabled>Import</Button>
									</span>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>
								This entry is missing one of the four macros, so it cannot be imported.
							</Tooltip.Content>
						</Tooltip.Root>
					</Tooltip.Provider>
				{/if}
			</Item.Actions>
		</Item.Root>
	{/each}
</Item.Group>
