<script lang="ts">
	let {
		label,
		links,
		current,
	}: {
		label: string;
		links: readonly { href: string; label: string }[];
		/** The href of the link that is open. */
		current: string;
	} = $props();
</script>

<!-- A tab holds two lists (calendar and day plans, meals and products); this flips between them. -->
<nav class="segmented" aria-label={label}>
	{#each links as link (link.href)}
		<a href={link.href} aria-current={link.href === current ? 'page' : undefined}>{link.label}</a>
	{/each}
</nav>
