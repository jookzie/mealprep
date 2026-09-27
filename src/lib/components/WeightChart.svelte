<script lang="ts">
	import type { WeightPoint } from '$lib/api';
	import { dayMonthLabel, fullDateLabel } from '$lib/domain/week';
	import {
		type Box,
		domain,
		dots,
		formatKilograms,
		nearest,
		scale,
		ticks,
		trendPath,
	} from '$lib/domain/weight';

	let { points }: { points: WeightPoint[] } = $props();

	const HEIGHT = 180;
	const PAD_X = 10;
	const PAD_Y = 14;
	/** Small, because a daily series puts a mark on nearly every pixel column. */
	const DOT = 2.5;

	let width = $state(320);
	/** The day being read off the chart, or null when nothing is being pointed at. */
	let reading = $state<number | null>(null);

	const box = $derived<Box>({ width, height: HEIGHT, padX: PAD_X, padY: PAD_Y });
	const span = $derived(domain(points));
	const to = $derived(scale(points.length, span, box));
	const path = $derived(trendPath(points, to));
	const marks = $derived(dots(points, to));
	const lines = $derived(ticks(span));
	const selected = $derived(reading === null ? null : (points[reading] ?? null));
	const latest = $derived(points.at(-1) ?? null);

	function read(event: PointerEvent) {
		const bounds = event.currentTarget as SVGSVGElement;
		const x = event.clientX - bounds.getBoundingClientRect().left;
		reading = nearest(points, x, to);
	}

	/*
	 * A touch pointer leaves the element the moment the finger lifts, so clearing on every
	 * leave would wipe the reading before it could be read. A mouse leaving means the
	 * cursor has gone, which does mean stop.
	 */
	function stopReading(event: PointerEvent) {
		if (event.pointerType === 'mouse') reading = null;
	}
</script>

<figure class="chart">
	<figcaption class="readout small">
		{#if selected}
			<span class="date">{fullDateLabel(selected.date)}</span>
			<span class="value">
				{selected.kilograms === undefined ? 'not weighed' : formatKilograms(selected.kilograms)}
			</span>
			<span class="muted">trend {formatKilograms(selected.trend)}</span>
		{:else if latest}
			<!-- The trend already stands above the chart; repeating it here would say it twice. -->
			<span class="muted">Tap the chart to read a day</span>
		{/if}
	</figcaption>

	<div class="plot" bind:clientWidth={width}>
		<svg
			viewBox="0 0 {width} {HEIGHT}"
			width={width}
			height={HEIGHT}
			role="img"
			aria-label="Weight over time: the days weighed as points, with the smoothed trend as a line. The history below lists the same figures."
			onpointermove={read}
			onpointerdown={read}
			onpointerleave={stopReading}
		>
			{#each lines as value (value)}
				{@const y = to.y(value)}
				<line class="grid" x1={PAD_X} y1={y} x2={width - PAD_X} y2={y} />
				<text class="tick" x={PAD_X} y={y - 4}>{value.toFixed(1)}</text>
			{/each}

			{#if selected && reading !== null}
				<line class="cursor" x1={to.x(reading)} y1={PAD_Y} x2={to.x(reading)} y2={HEIGHT - PAD_Y} />
			{/if}

			<path class="trend" d={path} />

			{#each marks as mark (mark.date)}
				<circle class="dot" cx={mark.x} cy={mark.y} r={DOT} />
			{/each}

			{#if selected?.kilograms !== undefined && reading !== null}
				<circle class="dot selected" cx={to.x(reading)} cy={to.y(selected.kilograms)} r={DOT + 2} />
			{/if}
		</svg>
	</div>

	<div class="axis small muted">
		<span>{points[0] ? dayMonthLabel(points[0].date) : ''}</span>
		<span class="legend">
			<span class="key"><span class="swatch dot-key"></span>weighed</span>
			<span class="key"><span class="swatch line-key"></span>trend</span>
		</span>
		<span>{latest ? dayMonthLabel(latest.date) : ''}</span>
	</div>
</figure>

<style>
	.chart {
		margin: 0;
	}

	.readout {
		display: flex;
		flex-wrap: wrap;
		gap: 0 0.5rem;
		align-items: baseline;
		min-height: 1.4rem;
	}

	.readout .value {
		font-weight: 600;
	}

	.plot {
		width: 100%;
	}

	svg {
		display: block;
		touch-action: pan-y;
	}

	/* Recessive: the grid is a reference, not a subject. */
	.grid {
		stroke: var(--border);
		stroke-width: 1;
	}

	.tick {
		fill: var(--muted);
		font-size: 0.7rem;
	}

	.cursor {
		stroke: var(--border);
		stroke-width: 1;
	}

	/*
	 * The trend is the figure worth reading, so it carries the ink; the measurements are
	 * the noise it was smoothed out of and stay muted. Neither takes a macro hue: weight
	 * is not a macro, and the accent means "interactive" everywhere else.
	 */
	.trend {
		fill: none;
		stroke: var(--text);
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.dot {
		fill: var(--muted);
	}

	.dot.selected {
		fill: var(--text);
		stroke: var(--surface);
		stroke-width: 2;
	}

	.axis {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;
	}

	.legend {
		display: flex;
		gap: 0.75rem;
	}

	.key {
		display: inline-flex;
		align-items: center;
		gap: 0.3rem;
	}

	.swatch {
		display: inline-block;
	}

	.dot-key {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--muted);
	}

	.line-key {
		width: 14px;
		height: 2px;
		background: var(--text);
	}
</style>
