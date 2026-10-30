<script lang="ts">
	import { toasts } from '$lib/toasts.svelte';
</script>

<div class="toaster" aria-live="polite">
	{#each toasts.items as toast (toast.id)}
		<button class="toast {toast.kind}" onclick={() => toasts.dismiss(toast.id)}>
			{toast.message}
		</button>
	{/each}
</div>

<style>
	/* Above the tab bar and the Ask handle, and above an open sheet too. */
	.toaster {
		position: fixed;
		left: 1rem;
		right: 1rem;
		bottom: calc(var(--chrome-bottom) + 0.5rem);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		z-index: 60;
		pointer-events: none;
	}

	.toast {
		pointer-events: auto;
		max-width: 32rem;
		width: 100%;
		justify-content: flex-start;
		text-align: left;
		white-space: normal;
		border-radius: 20px;
		padding: 0.75rem 1rem;
		background: var(--accent);
		color: var(--accent-text);
		box-shadow: 0 10px 30px rgb(0 0 0 / 0.25);
	}

	.toast.error {
		background: var(--danger);
		color: var(--bg);
	}
</style>
