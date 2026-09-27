<script lang="ts">
	import { beforeNavigate } from '$app/navigation';
	import { scanner } from '$lib/scanner.svelte';

	// Android's back button would navigate the hidden page under the camera; while a scan
	// runs it cancels the scan instead.
	beforeNavigate((navigation) => {
		if (!scanner.active) return;
		navigation.cancel();
		void scanner.cancel();
	});
</script>

{#if scanner.active}
	<div class="overlay">
		<p class="hint">Point the camera at the barcode</p>
		<div class="frame" aria-hidden="true"></div>
		<button type="button" onclick={() => scanner.cancel()}>Cancel</button>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		z-index: 40;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1.5rem;
		padding: env(safe-area-inset-top) 1rem calc(env(safe-area-inset-bottom) + 1rem);
		overflow: hidden;
	}

	.hint {
		color: #fff;
		font-weight: 600;
		text-shadow: 0 1px 4px rgb(0 0 0 / 0.8);
	}

	/* The shadow dims everything around the frame, which stays clear. */
	.frame {
		width: min(80vw, 22rem);
		aspect-ratio: 2 / 1;
		border: 3px solid #fff;
		border-radius: var(--radius);
		box-shadow: 0 0 0 100vmax rgb(0 0 0 / 0.45);
	}

	button {
		min-width: 8rem;
	}
</style>
