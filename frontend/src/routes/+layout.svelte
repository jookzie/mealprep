<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import favicon from '$lib/assets/favicon.svg';
	import AppSidebar from '$lib/components/app/app-sidebar.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Toaster } from '$lib/components/ui/sonner';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />
<Toaster position="bottom-right" />

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<header class="flex h-14 shrink-0 items-center gap-2 border-b px-4">
			<Sidebar.Trigger class="-ml-1" />
		</header>
		<main class="mx-auto w-full max-w-5xl flex-1 space-y-6 p-4 sm:p-6">
			{@render children()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>
