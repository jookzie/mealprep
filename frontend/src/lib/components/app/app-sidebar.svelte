<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar-days';
	import LayersIcon from '@lucide/svelte/icons/layers';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import TargetIcon from '@lucide/svelte/icons/target';
	import UtensilsIcon from '@lucide/svelte/icons/utensils';
	import WheatIcon from '@lucide/svelte/icons/wheat';
	import { toggleMode } from 'mode-watcher';
	import { page } from '$app/state';
	import * as Sidebar from '$lib/components/ui/sidebar';

	// Ordered by what builds on what: products come first, meals build on them,
	// day plans on meals, and the calendar on day plans.
	const sections = [
		{ href: '/calendar', label: 'Calendar', icon: CalendarIcon },
		{ href: '/day-plans', label: 'Day plans', icon: LayersIcon },
		{ href: '/meals', label: 'Meals', icon: UtensilsIcon },
		{ href: '/products', label: 'Products', icon: WheatIcon },
		{ href: '/targets', label: 'Targets', icon: TargetIcon }
	];

	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(`${href}/`);
	}
</script>

<Sidebar.Root collapsible="icon">
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg">
					{#snippet child({ props })}
						<a href="/calendar" {...props}>
							<div
								class="bg-primary text-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
							>
								<UtensilsIcon class="size-4" />
							</div>
							<div class="grid flex-1 text-left leading-tight">
								<span class="truncate font-medium">Mealprep</span>
								<span class="text-muted-foreground truncate text-xs">Plan against targets</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>

	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each sections as section (section.href)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton isActive={isActive(section.href)} tooltipContent={section.label}>
								{#snippet child({ props })}
									<a href={section.href} {...props}>
										<section.icon />
										<span>{section.label}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>

	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={toggleMode} tooltipContent="Toggle theme">
					<SunIcon class="dark:hidden" />
					<MoonIcon class="hidden dark:block" />
					<span>Toggle theme</span>
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
	<Sidebar.Rail />
</Sidebar.Root>
