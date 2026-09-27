<script lang="ts">
	import '../app.css';
	import CalendarDays from '@lucide/svelte/icons/calendar-days';
	import Layers from '@lucide/svelte/icons/layers';
	import Target from '@lucide/svelte/icons/target';
	import Utensils from '@lucide/svelte/icons/utensils';
	import Wheat from '@lucide/svelte/icons/wheat';
	import { navigating, page } from '$app/state';
	import ScanOverlay from '$lib/components/ScanOverlay.svelte';
	import Toaster from '$lib/components/Toaster.svelte';

	let { children } = $props();

	/*
	 * Plan (calendar, day plans) before Library (meals, products), with targets last as
	 * configuration rather than content. On a phone the sidebar becomes a bottom bar.
	 */
	const sections = [
		{ href: '/calendar', label: 'Calendar', icon: CalendarDays },
		{ href: '/day-plans', label: 'Day plans', icon: Layers },
		{ href: '/meals', label: 'Meals', icon: Utensils },
		{ href: '/products', label: 'Products', icon: Wheat },
		{ href: '/targets', label: 'Targets', icon: Target },
	];

	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(`${href}/`);
	}
</script>

{#if navigating.to}
	<div class="loading" aria-hidden="true"></div>
{/if}

<main class="content">
	{@render children()}
</main>

<nav class="tabs" aria-label="Sections">
	{#each sections as section (section.href)}
		<a href={section.href} aria-current={isActive(section.href) ? 'page' : undefined}>
			<section.icon size={22} aria-hidden="true" />
			<span>{section.label}</span>
		</a>
	{/each}
</nav>

<Toaster />
<ScanOverlay />

<style>
	/* The camera preview shows through the page while a scan runs. */
	:global(html.scanning) .content,
	:global(html.scanning) .tabs,
	:global(html.scanning) .loading {
		visibility: hidden;
	}

	.content {
		max-width: 60rem;
		margin: 0 auto;
		padding: calc(env(safe-area-inset-top) + 1rem) 1rem
			calc(var(--nav-height) + env(safe-area-inset-bottom) + 1.5rem);
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.tabs {
		position: fixed;
		left: 0;
		right: 0;
		bottom: 0;
		height: calc(var(--nav-height) + env(safe-area-inset-bottom));
		padding-bottom: env(safe-area-inset-bottom);
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		background: var(--surface);
		border-top: 1px solid var(--border);
		z-index: 10;
	}

	.tabs a {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		font-size: 0.7rem;
		text-decoration: none;
		color: var(--muted);
	}

	.tabs a[aria-current='page'] {
		color: var(--accent);
		font-weight: 600;
	}

	.loading {
		position: fixed;
		top: 0;
		left: 0;
		height: 3px;
		width: 100%;
		z-index: 30;
		background: linear-gradient(90deg, transparent, var(--accent), transparent);
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
