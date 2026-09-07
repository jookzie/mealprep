<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import type { Nutrients, Unit } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Collapsible from '$lib/components/ui/collapsible';
	import * as Table from '$lib/components/ui/table';
	import { nutrientRows } from '$lib/domain/nutrients';

	// PR-9: the four macros are shown directly, everything else sits behind this.
	// Ordered and indented the way Regulation (EU) 1169/2011 Annex XV presents a
	// nutrition declaration, since the catalog this data comes from is EU-origin.
	let { nutrients, unit }: { nutrients: Nutrients | undefined; unit: Unit } = $props();

	const rows = $derived(nutrientRows(nutrients));
	let open = $state(false);
</script>

{#if rows.length > 0}
	<Collapsible.Root bind:open>
		<Collapsible.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="ghost" size="sm" class="gap-1.5">
					<ChevronDownIcon class="size-4 transition-transform {open ? 'rotate-180' : ''}" />
					{open ? 'Hide' : 'Show'} all {rows.length} nutrients
				</Button>
			{/snippet}
		</Collapsible.Trigger>
		<Collapsible.Content class="pt-3">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Nutrient</Table.Head>
						<Table.Head class="text-right">Per 100 {unit}</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each rows as row (row.key)}
						<Table.Row>
							<Table.Cell class={row.depth === 1 ? 'text-muted-foreground pl-6' : undefined}>
								{row.label}
							</Table.Cell>
							<Table.Cell class="text-right tabular-nums">{row.value}</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Collapsible.Content>
	</Collapsible.Root>
{:else}
	<p class="text-muted-foreground text-sm">No further nutrients recorded.</p>
{/if}
