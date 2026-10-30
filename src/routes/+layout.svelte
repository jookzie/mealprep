<script lang="ts">
	import '@fontsource-variable/bricolage-grotesque';
	import '@fontsource-variable/dm-sans';
	import '../app.css';
	import { afterNavigate, goto, onNavigate, pushState } from '$app/navigation';
	import { navigating, page } from '$app/state';
	import { runMutation, setAssistantKey } from '$lib/api';
	import AskSheet from '$lib/components/AskSheet.svelte';
	import ScanOverlay from '$lib/components/ScanOverlay.svelte';
	import TabBar from '$lib/components/TabBar.svelte';
	import Toaster from '$lib/components/Toaster.svelte';
	import { classifySwipe, direction, neighbour, type Section, sectionOf } from '$lib/domain/sections';
	import { scanner } from '$lib/scanner.svelte';

	let { data, children } = $props();

	const current = $derived(sectionOf(page.url.pathname));

	/*
	 * An editor takes the whole screen: its save bar sits where the tab bar would, and a
	 * stray swipe should not carry the user off a half-typed form. The draft would survive
	 * it, but the user would have to find their way back.
	 */
	const editing = $derived(
		/\/(new|edit)$/.test(page.url.pathname) || page.url.pathname.startsWith('/products/import/'),
	);
	$effect(() => {
		document.documentElement.toggleAttribute('data-editing', editing);
	});

	/*
	 * Where each section was left, so a tab — or a swipe — returns to the form or detail the
	 * user was on rather than to the top of the section. Tapping the open tab goes to its top.
	 */
	let left = $state<Partial<Record<Section['key'], string>>>({});

	afterNavigate(({ to }) => {
		if (!to) return;
		const section = sectionOf(to.url.pathname);
		if (section) left[section.key] = `${to.url.pathname}${to.url.search}`;
	});

	function hrefFor(section: Section): string {
		return current?.key === section.key ? section.href : (left[section.key] ?? section.href);
	}

	/*
	 * Sideways swipes walk the sections and wrap around. A swipe that starts on something
	 * that scrolls or edits sideways belongs to that thing, as does one from the screen's
	 * edge, which Android keeps for its back gesture.
	 */
	const EDGE_PX = 24;
	let touch: { x: number; y: number; at: number } | null = null;

	function claimsSideways(target: EventTarget | null): boolean {
		for (let node = target as HTMLElement | null; node && node !== document.body; node = node.parentElement) {
			if (node.matches('input, textarea, select, [contenteditable], [data-swipe-ignore]')) return true;
			const overflow = getComputedStyle(node).overflowX;
			if ((overflow === 'auto' || overflow === 'scroll') && node.scrollWidth > node.clientWidth) {
				return true;
			}
		}
		return false;
	}

	function touchstart(event: TouchEvent) {
		touch = null;
		if (event.touches.length !== 1) return;
		const { clientX, clientY } = event.touches[0];
		if (clientX < EDGE_PX || clientX > window.innerWidth - EDGE_PX) return;
		if (claimsSideways(event.target)) return;
		touch = { x: clientX, y: clientY, at: performance.now() };
	}

	function touchend(event: TouchEvent) {
		const started = touch;
		touch = null;
		if (!started || !current || editing || askOpen || scanner.active) return;
		if (document.querySelector('dialog[open]')) return;
		const { clientX, clientY } = event.changedTouches[0];
		const step = classifySwipe({
			dx: clientX - started.x,
			dy: clientY - started.y,
			ms: performance.now() - started.at,
		});
		if (step === null) return;
		const target = neighbour(current.key, step);
		void goto(left[target.key] ?? target.href);
	}

	/*
	 * Changing section slides the page the way the tabs run; everything else just swaps.
	 * The tab bar has a view-transition name of its own, so it stays put while the page moves.
	 */
	onNavigate((navigation) => {
		if (!document.startViewTransition) return;
		if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		const from = navigation.from && sectionOf(navigation.from.url.pathname);
		const to = navigation.to && sectionOf(navigation.to.url.pathname);
		if (!from || !to || from.key === to.key) return;
		const root = document.documentElement;
		root.dataset.slide = direction(from.key, to.key) ?? '';
		return new Promise<void>((resolve) => {
			const transition = document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
			transition.finished.finally(() => delete root.dataset.slide);
		});
	});

	/*
	 * The Ask sheet is a shallow history entry, so Android's back gesture closes it instead of
	 * leaving the screen it was opened over. The chat itself lives in `$lib/assistant` and
	 * outlives the sheet.
	 */
	const askOpen = $derived(page.state.ask === true);

	function openAsk() {
		if (!askOpen) pushState('', { ask: true });
	}

	function closeAsk() {
		if (askOpen) history.back();
	}

	function saveKey(key: string) {
		return runMutation(() => setAssistantKey(key), { success: 'Ask is on' });
	}
</script>

<svelte:window ontouchstart={touchstart} ontouchend={touchend} ontouchcancel={() => (touch = null)} />

{#if navigating.to}
	<div class="loading" aria-hidden="true"></div>
{/if}

<main class="content">
	{@render children()}
</main>

{#if !editing}
	<TabBar current={current?.key} {hrefFor} onAsk={openAsk} />
{/if}

<AskSheet open={askOpen} keyStatus={data.assistantKey} onclose={closeAsk} onSaveKey={saveKey} />

<Toaster />
<ScanOverlay />

<style>
	/* The camera preview shows through the page while a scan runs. */
	:global(html.scanning) .content,
	:global(html.scanning) :global(.chrome),
	:global(html.scanning) .loading {
		visibility: hidden;
	}

	.content {
		max-width: 40rem;
		margin: 0 auto;
		padding: calc(env(safe-area-inset-top) + 1.5rem) 1.125rem calc(var(--chrome-bottom) + 1.5rem);
		display: flex;
		flex-direction: column;
		gap: 1rem;
		min-height: 100dvh;
	}

	.loading {
		position: fixed;
		top: 0;
		left: 0;
		height: 3px;
		width: 100%;
		z-index: 30;
		background: linear-gradient(90deg, transparent, var(--text), transparent);
		background-size: 50% 100%;
		background-repeat: no-repeat;
		animation: sweep 1s linear infinite;
	}

	@keyframes sweep {
		from {
			background-position: -50% 0;
		}
		to {
			background-position: 150% 0;
		}
	}
</style>
