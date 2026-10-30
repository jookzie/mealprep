<script lang="ts">
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import type { Snippet } from 'svelte';

	let {
		title,
		back,
		backLabel = 'Back',
		eyebrow,
		subtitle,
		actions,
	}: {
		title: string;
		back?: string;
		/** What the back pill names: where it goes, so it reads as a place rather than a verb. */
		backLabel?: string;
		/** A small line above the title: a date, a category. */
		eyebrow?: string;
		subtitle?: string;
		actions?: Snippet;
	} = $props();
</script>

{#snippet titles()}
	<div class="titles">
		{#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
		<h1>{title}</h1>
		{#if subtitle}<p class="muted subtitle">{subtitle}</p>{/if}
	</div>
{/snippet}

<!--
	A pushed screen has its way back and its actions on one line above the title; a tab's
	top screen has no way back, so its actions sit beside the title instead.
-->
<header class="stack">
	{#if back}
		<div class="row spread">
			<a class="button back" href={back}>
				<ChevronLeft size={20} strokeWidth={2.6} aria-hidden="true" />
				<span class="truncate">{backLabel}</span>
			</a>
			{#if actions}
				<div class="row actions">{@render actions()}</div>
			{/if}
		</div>
		{@render titles()}
	{:else}
		<div class="row spread beside">
			{@render titles()}
			{#if actions}
				<div class="row actions">{@render actions()}</div>
			{/if}
		</div>
	{/if}
</header>

<style>
	header {
		gap: 0.75rem;
	}

	.back {
		padding-left: 0.5rem;
		max-width: 60%;
		gap: 0.15rem;
	}

	/* Below this the actions would squeeze the back pill, so they drop to their own line. */
	.actions {
		justify-content: flex-end;
		flex-shrink: 0;
	}

	.beside {
		align-items: flex-end;
	}

	.beside .titles {
		flex: 1 1 12rem;
		min-width: 0;
	}

	.eyebrow {
		color: var(--muted);
		font-weight: 600;
		margin-bottom: 0.15rem;
	}

	.subtitle {
		margin-top: 0.3rem;
		font-weight: 600;
	}
</style>
