<script lang="ts">
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

<button class="danger" onclick={() => (open = true)}>Delete</button>

<Dialog bind:open title="Delete {what}?">
	<p class="muted">{consequence}</p>
	<div class="row end">
		<button onclick={() => (open = false)}>Cancel</button>
		<button class="primary" onclick={confirm} disabled={busy}>Delete</button>
	</div>
</Dialog>
