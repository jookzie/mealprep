<script lang="ts">
	import BookOpen from '@lucide/svelte/icons/book-open';
	import CalendarDays from '@lucide/svelte/icons/calendar-days';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Sun from '@lucide/svelte/icons/sun';
	import UserRound from '@lucide/svelte/icons/user-round';
	import { SECTIONS, type Section, type SectionKey, SHEET_OPEN_PX } from '$lib/domain/sections';

	let {
		current,
		hrefFor,
		onAsk,
	}: {
		current: SectionKey | undefined;
		/** Where a tab goes: where its section was left, or its top when it is already open. */
		hrefFor: (section: Section) => string;
		onAsk: () => void;
	} = $props();

	const ICONS = { today: Sun, plan: CalendarDays, library: BookOpen, you: UserRound };

	// The handle opens on a short upward drag as well as on a tap: it is the thing the user
	// pulls the sheet up by, and a tap is the single-pointer way to the same place.
	let start: number | null = null;

	function grab(event: PointerEvent) {
		start = event.clientY;
	}

	function move(event: PointerEvent) {
		if (start === null) return;
		if (start - event.clientY > SHEET_OPEN_PX) {
			start = null;
			onAsk();
		}
	}
</script>

<div class="chrome">
	<button
		type="button"
		class="ask"
		aria-label="Ask the assistant"
		onpointerdown={grab}
		onpointermove={move}
		onpointerup={() => (start = null)}
		onpointercancel={() => (start = null)}
		onclick={onAsk}
	>
		<span class="grabber" aria-hidden="true"></span>
		<span class="chip">
			<Sparkles size={16} strokeWidth={2.4} aria-hidden="true" />
			Ask
		</span>
	</button>

	<nav class="tabs" aria-label="Sections">
		{#each SECTIONS as section (section.key)}
			{@const Icon = ICONS[section.key]}
			{@const active = current === section.key}
			<a
				href={hrefFor(section)}
				class:active
				aria-current={active ? 'page' : undefined}
				aria-label={active ? undefined : section.label}
			>
				<Icon size={22} strokeWidth={2.2} aria-hidden="true" />
				{#if active}<span>{section.label}</span>{/if}
			</a>
		{/each}
	</nav>
</div>

<style>
	/* Pinned while the page underneath slides from tab to tab. */
	.chrome {
		position: fixed;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 10;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 0 20px calc(var(--nav-gap) + env(safe-area-inset-bottom));
		pointer-events: none;
		view-transition-name: chrome;
	}

	.ask {
		pointer-events: auto;
		touch-action: none;
		background: transparent;
		border: none;
		padding: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 5px;
		color: var(--text);
	}

	.grabber {
		width: 36px;
		height: 5px;
		border-radius: 3px;
		background: var(--text);
	}

	.chip {
		height: 36px;
		padding: 0 16px;
		border-radius: 18px;
		background: var(--raised);
		border: 2px solid var(--text);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 0.875rem;
		font-weight: 700;
		box-shadow: 0 6px 18px rgb(0 0 0 / 0.14);
	}

	.tabs {
		pointer-events: auto;
		width: 100%;
		max-width: 28rem;
		height: var(--nav-height);
		padding: 8px;
		border-radius: 32px;
		background: var(--accent);
		display: flex;
		align-items: center;
		justify-content: space-between;
		box-shadow: 0 10px 30px rgb(0 0 0 / 0.25);
	}

	.tabs a {
		height: 48px;
		min-width: 56px;
		border-radius: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		color: var(--accent-text);
		text-decoration: none;
		font-weight: 700;
	}

	/* The open section is the one tab that names itself, in a pill of the page colour. */
	.tabs a.active {
		background: var(--bg);
		color: var(--text);
		padding: 0 18px;
	}
</style>
