<script lang="ts">
	import DownloadIcon from '@lucide/svelte/icons/download';
	import type { CatalogEntry } from '$lib/api';
	import NutrientPanel from '$lib/components/macros/nutrient-panel.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { formatGrams, formatKcal } from '$lib/domain/format';

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

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head>Catalog entry</Table.Head>
			<Table.Head class="text-right">Energy</Table.Head>
			<Table.Head class="text-right">Protein</Table.Head>
			<Table.Head class="text-right">Fat</Table.Head>
			<Table.Head class="text-right">Carbs</Table.Head>
			<Table.Head class="w-1"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each entries as entry (entry.code)}
			<Table.Row>
				<Table.Cell class="max-w-xs">
					<p class="font-medium">{entry.name}</p>
					<p class="text-muted-foreground text-xs">per 100 {entry.unit} · {entry.code}</p>
					<div class="mt-1">
						<NutrientPanel nutrients={entry.nutrients} unit={entry.unit} />
					</div>
				</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatKcal(entry.macros.energyKcal)}</Table.Cell
				>
				<Table.Cell class="text-right tabular-nums">{formatGrams(entry.macros.proteinG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">{formatGrams(entry.macros.fatG)}</Table.Cell>
				<Table.Cell class="text-right tabular-nums">
					{formatGrams(entry.macros.carbohydratesG)}
				</Table.Cell>
				<Table.Cell>
					{#if entry.complete}
						<Button
							size="sm"
							disabled={importing !== null}
							onclick={() => onImport(entry.code)}
						>
							<DownloadIcon class="size-4" />
							{importing === entry.code ? 'Importing…' : 'Import'}
						</Button>
					{:else}
						<!-- The API rejects these with a 422; saying so up front beats
						     letting the user find out by clicking. -->
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
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
