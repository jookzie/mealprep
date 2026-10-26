<script lang="ts">
	let { onStartOver }: { onStartOver: () => void } = $props();
</script>

<!-- Said out loud: a form that silently differs from the record it edits reads as a bug. -->
<div class="card row spread notice" role="status">
	<span class="small">Your unsaved changes are back where you left them.</span>
	<button type="button" onclick={onStartOver}>Start over</button>
</div>

<style>
	.notice {
		border-color: var(--accent);
	}
</style>
