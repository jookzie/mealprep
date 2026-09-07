<script lang="ts">
	import type { Macros } from '$lib/api';
	import { formatGrams, formatKcal } from '$lib/domain/format';
	import { MACRO_LABELS } from '$lib/domain/macros';
	import { MAX_FRACTION, pointAt, polygon, radarPoints, ringRadius } from '$lib/domain/radar';
	import { MACRO_HUE } from './hues';

	/*
	 * The four macros as one shape, so two plans can be told apart at a glance by
	 * outline rather than by reading eight numbers.
	 *
	 * Each axis is a fraction of its own target, because kcal and grams cannot share a
	 * linear scale. The emphasised ring is the target; the box holds 1.5x it, so going
	 * over is visible rather than clipped at the edge.
	 *
	 * Deliberately decorative for assistive technology: every value it draws is printed
	 * as text beside it, so announcing the polygon would only repeat the figures.
	 */
	let {
		macros,
		denominator,
		size = 120,
		showLabels = true,
		relativeTo = 'target'
	}: {
		macros: Macros;
		denominator: Macros;
		size?: number;
		showLabels?: boolean;
		/** What the emphasised ring means, which changes whether it is a target at all. */
		relativeTo?: 'target' | 'largest';
	} = $props();

	const radius = $derived(size / 2 - (showLabels ? 16 : 3));
	const points = $derived(radarPoints(macros, denominator, radius, MACRO_LABELS));
	const shape = $derived(polygon(points));
	const targetRing = $derived(ringRadius(radius, 1));
	const halfRing = $derived(ringRadius(radius, 0.5));

	const summary = $derived(
		points
			.map((p) => {
				const value = p.key === 'energyKcal' ? formatKcal(p.value) : formatGrams(p.value);
				return `${p.label} ${value}`;
			})
			.join(', ')
	);
</script>

<figure class="m-0 flex flex-col items-center gap-1">
	<svg
		viewBox="{-size / 2} {-size / 2} {size} {size}"
		width={size}
		height={size}
		role="img"
		aria-label={summary}
	>
		<!-- Rings: the outer edge is 1.5x, the emphasised one is the target itself. -->
		<circle r={radius} fill="none" class="stroke-border" stroke-width="1" />
		<circle r={halfRing} fill="none" class="stroke-border" stroke-width="0.5" />
		<circle
			r={targetRing}
			fill="none"
			class={relativeTo === 'target' ? 'stroke-foreground/45' : 'stroke-border'}
			stroke-width={relativeTo === 'target' ? 1.25 : 0.5}
			stroke-dasharray={relativeTo === 'target' ? '3 2' : undefined}
		/>

		{#each points as point (point.key)}
			{@const outer = pointAt(points.indexOf(point), points.length, radius, MAX_FRACTION)}
			<line x1="0" y1="0" x2={outer.x} y2={outer.y} class="stroke-border" stroke-width="0.5" />
		{/each}

		<polygon
			points={shape}
			class="fill-primary/20 stroke-primary"
			stroke-width="1.5"
			stroke-linejoin="round"
		/>

		{#each points as point (point.key)}
			<circle cx={point.x} cy={point.y} r="2.5" fill={MACRO_HUE[point.key]} />
		{/each}

		{#if showLabels}
			{#each points as point (point.key)}
				{@const label = pointAt(points.indexOf(point), points.length, radius + 11, MAX_FRACTION)}
				<text
					x={label.x}
					y={label.y}
					text-anchor="middle"
					dominant-baseline="middle"
					class="text-[9px] font-medium"
					fill={MACRO_HUE[point.key]}
				>
					{point.label.charAt(0)}
				</text>
			{/each}
		{/if}
	</svg>
</figure>
