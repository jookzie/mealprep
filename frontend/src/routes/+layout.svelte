<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { navigating, page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import AppSidebar from '$lib/components/app/app-sidebar.svelte';
	import CommandPalette from '$lib/components/app/command-palette.svelte';
	import LoadingBar from '$lib/components/app/loading-bar.svelte';
	import RouteSkeleton from '$lib/components/app/route-skeleton.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Toaster } from '$lib/components/ui/sonner';

	let { children } = $props();

	/*
	 * The week grid needs seven readable columns and a form does not, so the container
	 * width follows the screen rather than one figure pinned globally.
	 */
	const wide = $derived(page.url.pathname === '/' || page.url.pathname.startsWith('/calendar'));

	// A navigation with nothing behind it is the first screen of the session, which
	// under ssr = false paints nothing until its load resolves.
	const firstLoad = $derived(navigating.to !== null && navigating.from === null);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />
<Toaster position="bottom-right" />
<CommandPalette />
<LoadingBar active={navigating.to !== null && navigating.from !== null} />

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
			<Sidebar.Trigger class="-ml-1" />
			<span class="text-muted-foreground ml-auto hidden text-xs sm:block">
				Press <kbd class="bg-muted rounded px-1.5 py-0.5 font-medium">⌘K</kbd> to go anywhere
			</span>
		</header>
		<main
			class="mx-auto w-full flex-1 space-y-6 p-4 sm:p-6 {wide ? 'max-w-[100rem]' : 'max-w-5xl'}"
		>
			{#if firstLoad}
				<RouteSkeleton pathname={navigating.to?.url.pathname ?? '/'} />
			{:else}
				{@render children()}
			{/if}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>
