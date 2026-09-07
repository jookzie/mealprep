<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import * as Empty from '$lib/components/ui/empty';

	// The registry's Empty, given this app's shape: an icon, a title, a sentence, and the
	// action that resolves it embedded rather than left for the user to find elsewhere.
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

<Empty.Root class="border">
	<Empty.Header>
		{#if Icon}
			<Empty.Media variant="icon">
				<Icon />
			</Empty.Media>
		{/if}
		<Empty.Title>{title}</Empty.Title>
		{#if description}
			<Empty.Description>{description}</Empty.Description>
		{/if}
	</Empty.Header>
	{#if action}
		<Empty.Content>{@render action()}</Empty.Content>
	{/if}
</Empty.Root>
