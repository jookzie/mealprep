<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar-days';
	import LayersIcon from '@lucide/svelte/icons/layers';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import TargetIcon from '@lucide/svelte/icons/target';
	import UtensilsIcon from '@lucide/svelte/icons/utensils';
	import WheatIcon from '@lucide/svelte/icons/wheat';
	import { goto } from '$app/navigation';
	import * as Command from '$lib/components/ui/command';

	/*
	 * ⌘K for navigation, and ⌘1–⌘4 for the four sections. The cheapest large ergonomics
	 * win available for a desktop tool one person uses every day; Command was already
	 * installed and used inside three popovers before this existed.
	 *
	 * It deliberately lists screens and actions only. Jumping to a named product or meal
	 * would mean loading every entity into the shell on every page, which is a read the
	 * routes own, not the chrome.
	 */
	let open = $state(false);

	const sections = [
		{ href: '/calendar', label: 'Calendar', icon: CalendarIcon, shortcut: '1' },
		{ href: '/day-plans', label: 'Day plans', icon: LayersIcon, shortcut: '2' },
		{ href: '/meals', label: 'Meals', icon: UtensilsIcon, shortcut: '3' },
		{ href: '/products', label: 'Products', icon: WheatIcon, shortcut: '4' }
	];

	const actions = [
		{ href: '/day-plans/new', label: 'New day plan', icon: PlusIcon },
		{ href: '/meals/new', label: 'New meal', icon: PlusIcon },
		{ href: '/products/new', label: 'New product', icon: PlusIcon },
		{ href: '/products/search', label: 'Search Open Food Facts', icon: SearchIcon },
		{ href: '/targets', label: 'Daily targets', icon: TargetIcon }
	];

	function onKeydown(event: KeyboardEvent) {
		if (!(event.metaKey || event.ctrlKey)) return;

		if (event.key === 'k') {
			event.preventDefault();
			open = !open;
			return;
		}

		const section = sections.find((candidate) => candidate.shortcut === event.key);
		if (section) {
			event.preventDefault();
			open = false;
			goto(section.href);
		}
	}

	function run(href: string) {
		open = false;
		goto(href);
	}
</script>

<svelte:window onkeydown={onKeydown} />

<Command.Dialog bind:open>
	<Command.Input placeholder="Go to a screen, or start something new…" />
	<Command.List>
		<Command.Empty>Nothing matches.</Command.Empty>
		<Command.Group heading="Go to">
			{#each sections as section (section.href)}
				<Command.Item value={section.label} onSelect={() => run(section.href)}>
					<section.icon class="size-4" />
					<span>{section.label}</span>
					<Command.Shortcut>⌘{section.shortcut}</Command.Shortcut>
				</Command.Item>
			{/each}
		</Command.Group>
		<Command.Separator />
		<Command.Group heading="Create">
			{#each actions as action (action.href)}
				<Command.Item value={action.label} onSelect={() => run(action.href)}>
					<action.icon class="size-4" />
					<span>{action.label}</span>
				</Command.Item>
			{/each}
		</Command.Group>
	</Command.List>
</Command.Dialog>
