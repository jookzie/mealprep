<script lang="ts">
	import { Meter } from 'bits-ui';
	import { formatGrams, formatKcal, formatSigned } from '$lib/domain/format';
	import { ratio } from '$lib/domain/macros';

	/*
	 * Planned against target, for one macro.
	 *
	 * A meter, not a progress bar: MDN defines a progressbar as task completion, always
	 * readonly and monotonic, while a meter is "a gauge indicating the scalar or
	 * fractional amount within a known range" — which is exactly what a nutrient
	 * against a target is. bits-ui ships Meter; shadcn-svelte's Progress does not wrap it.
	 *
	 * Going over target is drawn, never flagged (TG-3): the overflow segment keeps the
	 * macro's own hue and is differentiated by a hatch instead of a warning colour, so
	 * red in this app always means "something is broken", never "you ate too much".
	 */
	let {
		label,
		hue,
		actual,
		target,
		unit,
		mode = 'planned'
	}: {
		label: string;
		hue: string;
		actual: number;
		target: number;
		unit: 'kcal' | 'g';
		mode?: 'planned' | 'remaining';
	} = $props();

	// The target sits at a fixed mark on every row, so rows and days are comparable by
	// eye. The headroom past it draws up to a quarter over target on the same linear
	// scale; beyond that the bar clamps and the figures carry the rest.
	const TARGET_MARK = 80;
	const MAX_RATIO = 1.25;
	// Landing this close counts as hitting the target, which brightens the target line.
	const TOLERANCE = 0.02;

	const fraction = $derived(ratio(actual, target));
	const filled = $derived(Math.min(fraction, 1) * TARGET_MARK);
	const overflow = $derived(
		fraction > 1 ? (Math.min(fraction, MAX_RATIO) - 1) * TARGET_MARK : 0
	);
	const met = $derived(target > 0 && Math.abs(fraction - 1) <= TOLERANCE);

	const format = $derived(unit === 'kcal' ? formatKcal : formatGrams);
	const delta = $derived(actual - target);
	const primary = $derived(
		mode === 'remaining'
			? delta > 0
				? `${format(delta)} over`
				: `${format(-delta)} left`
			: `${format(actual)} of ${format(target)}`
	);

	/*
	 * Assistive technology often announces aria-valuenow as a percentage of the range,
	 * which would read 231 against a max of 200 as nonsense. The text is spelled out.
	 */
	const valueText = $derived(
		`${format(actual)} of ${format(target)} ${label.toLowerCase()}` +
			(met
				? ', target met'
				: delta === 0
					? ''
					: `, ${format(Math.abs(delta))} ${delta > 0 ? 'over' : 'under'} target`)
	);
</script>

<div class="space-y-1.5">
	<div class="flex flex-wrap items-baseline justify-between gap-x-3 text-sm">
		<span class="flex items-center gap-2">
			<span class="size-2 rounded-full" style:background-color={hue} aria-hidden="true"></span>
			<span class="text-muted-foreground">{label}</span>
		</span>
		<span class="flex items-baseline gap-3">
			<span class="whitespace-nowrap tabular-nums">{primary}</span>
			{#if mode === 'planned'}
				<span class="text-muted-foreground w-20 text-right text-xs tabular-nums">
					{formatSigned(delta, unit)}
				</span>
			{/if}
		</span>
	</div>

	<Meter.Root
		value={Math.min(actual, target * MAX_RATIO)}
		max={target > 0 ? target : 1}
		aria-label={label}
		aria-valuetext={valueText}
		class="bg-muted relative h-2.5 w-full overflow-hidden rounded-full"
		style="--macro-hue: {hue}"
	>
		<div class="absolute inset-y-0 left-0" style:width="{filled}%" style:background-color={hue}>
		</div>
		{#if overflow > 0}
			<div
				class="macro-overflow absolute inset-y-0"
				style:left="{TARGET_MARK}%"
				style:width="{overflow}%"
			></div>
		{/if}
		<!-- The target line never moves, so a row can be read against the one above it. -->
		<div
			class="absolute inset-y-0 -translate-x-1/2 transition-all"
			style:left="{TARGET_MARK}%"
			style:width={met ? '3px' : '1.5px'}
			style:background-color={met ? hue : 'var(--color-border)'}
			style:opacity={met ? '1' : '0.9'}
		></div>
	</Meter.Root>
</div>
