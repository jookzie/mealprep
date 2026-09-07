<script lang="ts">
	import PackageIcon from '@lucide/svelte/icons/package';

	/*
	 * The catalog's own photograph, linked rather than stored: the data is ODbL but the
	 * images are CC-BY-SA, and copying one is what pulls a share-alike obligation into
	 * the app.
	 *
	 * The catalog itself says an image URL "may not always be present", and a present
	 * one can still fail — the host is a third party the app does not control. Both
	 * cases land on the same glyph, so a result row never shows a broken image.
	 */
	let { src, alt = '' }: { src?: string; alt?: string } = $props();

	let failed = $state(false);
</script>

{#if src && !failed}
	<img
		{src}
		{alt}
		loading="lazy"
		class="size-10 rounded object-contain"
		onerror={() => (failed = true)}
	/>
{:else}
	<div class="bg-muted flex size-10 items-center justify-center rounded">
		<PackageIcon class="text-muted-foreground size-4" />
	</div>
{/if}
