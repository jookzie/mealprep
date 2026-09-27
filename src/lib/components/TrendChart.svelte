<script lang="ts">
	import { bandPath, type ChartPoint, chartDomain, linePath, valueDots } from '$lib/domain/trend-chart';
	import { dayMonthLabel, fullDateLabel } from '$lib/domain/week';
	import { type Box, nearest, scale, ticks } from '$lib/domain/weight';

	let {
		points,
		format,
		minSpan,
		label,
		valueName,
		lineName,
		bandName,
	}: {
		points: ChartPoint[];
		/** Formats a value for the readout, with its unit. */
		format: (value: number) => string;
		/** The narrowest window the y-axis shows, so a flat series does not magnify noise. */
		minSpan: number;
		/** What the chart shows, for a screen reader. */
		label: string;
		/** What a dot is: "reading", "measured". */
		valueName: string;
		/** What the line is: "7-day average", "between measurements". */
		lineName: string;
		/** What the shaded band is; no band is drawn or keyed without it. */
		bandName?: string;
	} = $props();

	const HEIGHT = 160;
	const PAD_X = 10;
	const PAD_Y = 12;
	const DOT = 2.5;

	let width = $state(320);
	let reading = $state<number | null>(null);

	const box = $derived<Box>({ width, height: HEIGHT, padX: PAD_X, padY: PAD_Y });
	const span = $derived(chartDomain(points, minSpan));
	const to = $derived(scale(points.length, span, box));
	const line = $derived(linePath(points, to));
	const band = $derived(bandName ? bandPath(points, to) : '');
	const marks = $derived(valueDots(points, to));
	const lines = $derived(ticks(span));
	const selected = $derived(reading === null ? null : (points[reading] ?? null));

	function read(event: PointerEvent) {
		const bounds = (event.currentTarget as SVGSVGElement).getBoundingClientRect();
		reading = nearest(points, event.clientX - bounds.left, to);
	}

	// A lifted finger leaves the element; only a mouse leaving means stop reading.
	function stopReading(event: PointerEvent) {
		if (event.pointerType === 'mouse') reading = null;
	}
</script>

<figure class="chart">
	<figcaption class="readout small">
		{#if selected}
			<span class="date">{fullDateLabel(selected.date)}</span>
			<span class="value">
				{selected.value === undefined ? `no ${valueName}` : format(selected.value)}
			</span>
			{#if selected.line !== undefined}
				<span class="muted">{lineName} {format(selected.line)}</span>
			{/if}
		{:else}
			<span class="muted">Tap the chart to read a day</span>
		{/if}
	</figcaption>

	<div class="plot" bind:clientWidth={width}>
		<svg
			viewBox="0 0 {width} {HEIGHT}"
			{width}
			height={HEIGHT}
			role="img"
			aria-label={label}
			onpointermove={read}
			onpointerdown={read}
			onpointerleave={stopReading}
		>
			{#if band}
				<path class="band" d={band} />
			{/if}

			{#each lines as value (value)}
				{@const y = to.y(value)}
				<line class="grid" x1={PAD_X} y1={y} x2={width - PAD_X} y2={y} />
				<text class="tick" x={PAD_X} y={y - 4}>{Number(value.toFixed(1))}</text>
			{/each}

			{#if selected && reading !== null}
				<line class="cursor" x1={to.x(reading)} y1={PAD_Y} x2={to.x(reading)} y2={HEIGHT - PAD_Y} />
			{/if}

			<path class="line" d={line} />

			{#each marks as mark (mark.date)}
				<circle class="dot" cx={mark.x} cy={mark.y} r={DOT} />
			{/each}

			{#if selected?.value !== undefined && reading !== null}
				<circle class="dot selected" cx={to.x(reading)} cy={to.y(selected.value)} r={DOT + 2} />
			{/if}
		</svg>
	</div>

	<div class="axis small muted">
		<span>{points[0] ? dayMonthLabel(points[0].date) : ''}</span>
		<span>{points.at(-1) ? dayMonthLabel(points.at(-1)?.date ?? '') : ''}</span>
	</div>
	<div class="legend small muted">
		<span class="key"><span class="swatch dot-key"></span>{valueName}</span>
		<span class="key"><span class="swatch line-key"></span>{lineName}</span>
		{#if bandName}
			<span class="key"><span class="swatch band-key"></span>{bandName}</span>
		{/if}
	</div>
</figure>

<style>
	/*
	 * The same language as the weight chart: the figure worth reading carries the ink, the
	 * raw readings are muted, and the normal range is a quiet tint behind both. No hue is
	 * spent on any of it; whether a week is good is said in words beside the chart.
	 */
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

	.band {
		fill: var(--band);
		stroke: var(--border);
		stroke-width: 1;
	}

	.grid,
	.cursor {
		stroke: var(--border);
		stroke-width: 1;
	}

	.tick {
		fill: var(--muted);
		font-size: 0.7rem;
	}

	.line {
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
		flex-wrap: wrap;
		justify-content: center;
		gap: 0 0.75rem;
		margin-top: 0.25rem;
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

	.band-key {
		width: 12px;
		height: 10px;
		background: var(--band);
		border: 1px solid var(--border);
	}
</style>
