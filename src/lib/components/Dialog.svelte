<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		title,
		children,
	}: { open: boolean; title: string; children: Snippet } = $props();

	let dialog: HTMLDialogElement | undefined = $state();

	// The native dialog is imperative, so its open state has to be pushed to the DOM.
	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) dialog.showModal();
		if (!open && dialog.open) dialog.close();
	});
</script>

<dialog bind:this={dialog} onclose={() => (open = false)}>
	{#if open}
		<div class="stack">
			<div class="row spread">
				<h2>{title}</h2>
				<button class="ghost icon" onclick={() => (open = false)} aria-label="Close">✕</button>
			</div>
			{@render children()}
		</div>
	{/if}
</dialog>
