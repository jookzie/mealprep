<script lang="ts">
	import type { Category } from '$lib/api';
	import DeleteButton from './DeleteButton.svelte';
	import Dialog from './Dialog.svelte';

	let {
		categories,
		counts,
		noun,
		onCreate,
		onRename,
		onDelete,
	}: {
		categories: readonly Category[];
		counts: ReadonlyMap<string, number>;
		noun: { one: string; many: string };
		onCreate: (name: string) => Promise<unknown>;
		onRename: (id: string, name: string) => Promise<unknown>;
		onDelete: (id: string) => Promise<unknown>;
	} = $props();

	let name = $state('');
	let renaming = $state<Category | null>(null);
	let renameText = $state('');
	let renameOpen = $state(false);

	async function create(event: SubmitEvent) {
		event.preventDefault();
		const created = await onCreate(name);
		if (created !== undefined) name = '';
	}

	function startRename(category: Category) {
		renaming = category;
		renameText = category.name;
		renameOpen = true;
	}

	async function rename(event: SubmitEvent) {
		event.preventDefault();
		if (!renaming) return;
		const renamed = await onRename(renaming.id, renameText);
		if (renamed !== undefined) renameOpen = false;
	}
</script>

<form class="card row" onsubmit={create}>
	<input class="grow" placeholder="New category name" aria-label="New category name" bind:value={name} />
	<button class="primary" type="submit" disabled={name.trim() === ''}>Add</button>
</form>

{#if categories.length === 0}
	<p class="empty">No categories yet.</p>
{:else}
	<ul class="list">
		{#each categories as category (category.id)}
			{@const count = counts.get(category.id) ?? 0}
			<li class="card row spread">
				<div class="grow">
					<strong>{category.name}</strong>
					<p class="muted small">{count} {count === 1 ? noun.one : noun.many}</p>
				</div>
				<button onclick={() => startRename(category)}>Rename</button>
				<DeleteButton
					what={category.name}
					consequence="What it groups stays, and reads as being in a removed category until you file it elsewhere."
					onConfirm={() => onDelete(category.id)}
				/>
			</li>
		{/each}
	</ul>
{/if}

<Dialog bind:open={renameOpen} title="Rename category">
	<form class="stack" onsubmit={rename}>
		<input aria-label="Category name" bind:value={renameText} />
		<div class="row end">
			<button type="button" onclick={() => (renameOpen = false)}>Cancel</button>
			<button class="primary" type="submit" disabled={renameText.trim() === ''}>Save</button>
		</div>
	</form>
</Dialog>
