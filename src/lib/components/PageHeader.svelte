<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		back,
		subtitle,
		actions,
	}: { title: string; back?: string; subtitle?: string; actions?: Snippet } = $props();
</script>

<header class="stack">
	{#if back}
		<a class="back small muted" href={back}>← Back</a>
	{/if}
	<div class="row spread">
		<div class="titles">
			<h1>{title}</h1>
			{#if subtitle}
				<p class="muted small">{subtitle}</p>
			{/if}
		</div>
		{#if actions}
			<div class="row actions">{@render actions()}</div>
		{/if}
	</div>
</header>

<style>
	header {
		gap: 0.25rem;
	}

	.back {
		text-decoration: none;
		padding: 0.25rem 0;
	}

	/* Below this basis the title would be squeezed into a ragged column, so the
	   actions drop to their own line instead of shrinking alongside it. */
	.titles {
		flex: 1 1 14rem;
		min-width: 0;
	}

	.actions {
		flex-shrink: 0;
	}
</style>
