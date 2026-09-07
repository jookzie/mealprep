<script lang="ts">
	// A navigation between screens keeps the previous one on screen while its load runs,
	// so what is missing there is not a placeholder but an indication that anything is
	// happening at all.
	let { active }: { active: boolean } = $props();
</script>

{#if active}
	<div class="pointer-events-none fixed inset-x-0 top-0 z-50 h-0.5" role="status" aria-live="polite">
		<div class="bg-primary h-full w-1/3 animate-[loading-sweep_1.1s_ease-in-out_infinite]"></div>
		<span class="sr-only">Loading</span>
	</div>
{/if}

<style>
	@keyframes loading-sweep {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(400%);
		}
	}
</style>
