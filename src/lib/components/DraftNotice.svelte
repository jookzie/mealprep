<script lang="ts">
	let { onStartOver }: { onStartOver: () => void } = $props();
</script>

<!-- Said out loud: a form that silently differs from the record it edits reads as a bug. -->
<div class="row spread draft" role="status">
	<span class="small">Your unsaved changes are back where you left them.</span>
	<button type="button" class="compact" onclick={onStartOver}>Start over</button>
</div>

<style>
	.draft {
		flex-wrap: nowrap;
		padding: 0.6rem 0.6rem 0.6rem 1rem;
		border-radius: var(--radius);
		border: 2px solid var(--text);
	}
</style>
