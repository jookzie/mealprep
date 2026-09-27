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
	.toaster {
		position: fixed;
		left: 0.75rem;
		right: 0.75rem;
		bottom: calc(var(--nav-height) + env(safe-area-inset-bottom) + 0.75rem);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		z-index: 20;
		pointer-events: none;
	}

	.toast {
		pointer-events: auto;
		max-width: 32rem;
		width: 100%;
		justify-content: flex-start;
		text-align: left;
		background: var(--text);
		color: var(--bg);
		border: none;
		box-shadow: 0 6px 24px rgb(0 0 0 / 0.2);
	}

	.toast.error {
		background: var(--danger);
		color: #fff;
	}
</style>
