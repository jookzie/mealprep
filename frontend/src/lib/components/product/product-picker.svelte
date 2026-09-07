<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import type { Product } from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import { formatKcal } from '$lib/domain/format';
	import { byRecency } from '$lib/domain/sort';

	let {
		products,
		value = $bindable(),
		id,
		open = $bindable(false),
		placeholder = 'Select a product…'
	}: {
		products: readonly Product[];
		value: string;
		id?: string;
		open?: boolean;
		placeholder?: string;
	} = $props();

	// Most recently touched first: what you imported or edited a minute ago is what you
	// are about to use, and the alphabet has no opinion about that.
	const ordered = $derived(byRecency(products));
	const selected = $derived(products.find((product) => product.id === value));

	function choose(productId: string) {
		value = productId;
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger {id}>
		{#snippet child({ props })}
			<!-- Named explicitly: once a product is chosen the trigger's text becomes that
			     product, which describes its value rather than what the control is for. -->
			<Button
				{...props}
				variant="outline"
				role="combobox"
				aria-expanded={open}
				aria-label={placeholder}
				class="w-full justify-between font-normal"
			>
				<span class="truncate">
					{selected ? selected.name : placeholder}
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
					{#each ordered as product (product.id)}
						<!-- Brand joins the search text: several products often share a name and
						     differ only by who makes them. -->
						<Command.Item
							value="{product.name} {product.brand ?? ''}"
							onSelect={() => choose(product.id)}
						>
							<CheckIcon class="size-4 {product.id === value ? 'opacity-100' : 'opacity-0'}" />
							<span class="flex-1 truncate">
								{product.name}
								{#if product.brand}
									<span class="text-muted-foreground">· {product.brand}</span>
								{/if}
							</span>
							<span class="text-muted-foreground text-xs tabular-nums">
								{formatKcal(product.macros.energyKcal)}
							</span>
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
