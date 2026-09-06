<script lang="ts">
	import type { Component, Snippet } from 'svelte';

	let {
		icon: Icon,
		title,
		description,
		action
	}: {
		icon?: Component<{ class?: string }>;
		title: string;
		description?: string;
		action?: Snippet;
	} = $props();
</script>

<div
	class="border-muted-foreground/25 flex flex-col items-center justify-center rounded-lg border border-dashed px-6 py-14 text-center"
>
	{#if Icon}
		<Icon class="text-muted-foreground mb-3 size-8" />
	{/if}
	<p class="font-medium">{title}</p>
	{#if description}
		<p class="text-muted-foreground mt-1 max-w-sm text-sm">{description}</p>
	{/if}
	{#if action}
		<div class="mt-5">{@render action()}</div>
	{/if}
</div>
