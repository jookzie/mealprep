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

	/*
	 * Plan is scheduling — what happens when. Library is the building blocks — what
	 * exists. The split mirrors a distinction already in the domain rather than an
	 * invented one, and it keeps the dependency ordering inside each group.
	 *
	 * Targets sits in the footer instead: it is one row of configuration, edited rarely,
	 * and no group label honestly covers both Products and Targets.
	 */
	const groups = [
		{
			label: 'Plan',
			items: [
				{ href: '/calendar', label: 'Calendar', icon: CalendarIcon, shortcut: '⌘1' },
				{ href: '/day-plans', label: 'Day plans', icon: LayersIcon, shortcut: '⌘2' }
			]
		},
		{
			label: 'Library',
			items: [
				{ href: '/meals', label: 'Meals', icon: UtensilsIcon, shortcut: '⌘3' },
				{ href: '/products', label: 'Products', icon: WheatIcon, shortcut: '⌘4' }
			]
		}
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
		{#each groups as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.href)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton isActive={isActive(item.href)} tooltipContent={item.label}>
									{#snippet child({ props })}
										<a href={item.href} {...props}>
											<item.icon />
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
								<Sidebar.MenuBadge class="text-muted-foreground text-xs">
									{item.shortcut}
								</Sidebar.MenuBadge>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>

	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton isActive={isActive('/targets')} tooltipContent="Targets">
					{#snippet child({ props })}
						<a href="/targets" {...props}>
							<TargetIcon />
							<span>Targets</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
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
