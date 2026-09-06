<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import type { Product } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';

	let {
		products,
		value = $bindable(),
		id
	}: { products: readonly Product[]; value: string; id?: string } = $props();

	let open = $state(false);
	const selected = $derived(products.find((product) => product.id === value));

	function choose(productId: string) {
		value = productId;
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger {id}>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="outline"
				role="combobox"
				aria-expanded={open}
				class="w-full justify-between font-normal"
			>
				<span class="truncate">
					{selected ? selected.name : 'Select a product…'}
				</span>
				<ChevronsUpDownIcon class="size-4 shrink-0 opacity-50" />
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-(--bits-popover-anchor-width) p-0">
		<Command.Root>
			<Command.Input placeholder="Search products…" />
			<Command.List>
				<Command.Empty>No product found.</Command.Empty>
				<Command.Group>
					{#each products as product (product.id)}
						<Command.Item value={product.name} onSelect={() => choose(product.id)}>
							<CheckIcon
								class="size-4 {product.id === value ? 'opacity-100' : 'opacity-0'}"
							/>
							<span class="flex-1 truncate">{product.name}</span>
							<span class="text-muted-foreground text-xs">per 100 {product.unit}</span>
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
