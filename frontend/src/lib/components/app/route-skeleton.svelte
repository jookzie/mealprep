<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton';

	/*
	 * ssr = false and every read living in a load means the first screen of a session
	 * paints nothing at all until its data arrives. This stands in for it, in roughly
	 * the shape of what is coming, so the wait reads as loading rather than as breakage.
	 *
	 * Only the first navigation of a session needs it; after that the previous screen
	 * stays up and the loading bar carries the news.
	 */
	let { pathname }: { pathname: string } = $props();

	const shape = $derived(
		pathname.startsWith('/calendar') ? 'week' : pathname === '/' ? 'week' : 'list'
	);
</script>

<div class="space-y-6" aria-hidden="true">
	<div class="space-y-2">
		<Skeleton class="h-8 w-56" />
		<Skeleton class="h-4 w-80" />
	</div>

	{#if shape === 'week'}
		<Skeleton class="h-28 w-full rounded-lg" />
		<div class="grid gap-2 lg:grid-cols-7">
			{#each Array.from({ length: 7 }, (_, index) => index) as index (index)}
				<Skeleton class="h-40 rounded-lg" />
			{/each}
		</div>
	{:else}
		<Skeleton class="h-9 w-64" />
		<div class="space-y-2">
			{#each Array.from({ length: 6 }, (_, index) => index) as index (index)}
				<Skeleton class="h-14 w-full rounded-md" />
			{/each}
		</div>
	{/if}
</div>
