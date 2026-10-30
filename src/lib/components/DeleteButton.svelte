<script lang="ts">
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Dialog from './Dialog.svelte';

	let {
		what,
		consequence = 'Anything that refers to it keeps the reference and shows it as removed.',
		onConfirm,
	}: { what: string; consequence?: string; onConfirm: () => Promise<unknown> } = $props();

	let open = $state(false);
	let busy = $state(false);

	async function confirm() {
		busy = true;
		try {
			await onConfirm();
		} finally {
			busy = false;
			open = false;
		}
	}
</script>

<button class="icon ghost danger" aria-label="Delete {what}" onclick={() => (open = true)}>
	<Trash2 size={18} strokeWidth={2.4} aria-hidden="true" />
</button>

<Dialog bind:open title="Delete {what}?">
	<p>{consequence}</p>
	<div class="grid-2">
		<button class="outline" onclick={() => (open = false)}>Keep it</button>
		<button class="danger outline" onclick={confirm} disabled={busy}>Delete</button>
	</div>
</Dialog>
