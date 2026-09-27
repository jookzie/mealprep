<script lang="ts">
	import {
		formatClock,
		formatDuration,
		hourTicks,
		type SleepDay,
		sleepDomain,
	} from '$lib/domain/health';
	import { dayMonthLabel, fullDateLabel } from '$lib/domain/week';

	let { days }: { days: SleepDay[] } = $props();

	const HEIGHT = 180;
	/** Room on the left for the clock labels. */
	const AXIS = 38;
	const PAD_Y = 8;

	let width = $state(320);
	let reading = $state<number | null>(null);

	const span = $derived(sleepDomain(days));
	const hours = $derived(hourTicks(span));
	const column = $derived((width - AXIS) / Math.max(days.length, 1));
	/** Bars thin to a line on a long range, and never fatter than a finger needs. */
	const bar = $derived(Math.min(Math.max(column - 2, 1), 14));
	const selected = $derived(reading === null ? null : (days[reading] ?? null));

	/** Bedtime at the top and waking at the bottom, the way the night is read. */
	function y(minute: number): number {
		const spread = span.max - span.min || 1;
		return PAD_Y + ((minute - span.min) / spread) * (HEIGHT - PAD_Y * 2);
	}

	function x(index: number): number {
		return AXIS + column * index + column / 2;
	}

	function read(event: PointerEvent) {
		const bounds = (event.currentTarget as SVGSVGElement).getBoundingClientRect();
		const index = Math.floor((event.clientX - bounds.left - AXIS) / column);
		reading = Math.min(Math.max(index, 0), days.length - 1);
	}

	function stopReading(event: PointerEvent) {
		if (event.pointerType === 'mouse') reading = null;
	}
</script>

<figure class="chart">
	<figcaption class="readout small">
		{#if selected?.night}
			{@const night = selected.night}
			<span class="date">{fullDateLabel(selected.date)}</span>
			<span class="value">{formatDuration(night.asleepMinutes)}</span>
			<span class="muted">{formatClock(night.bedMinute)}–{formatClock(night.wakeMinute)}</span>
			{#if night.deepMinutes !== undefined && night.remMinutes !== undefined}
				<span class="muted">
					deep {formatDuration(night.deepMinutes)} · REM {formatDuration(night.remMinutes)}
				</span>
			{/if}
		{:else if selected}
			<span class="date">{fullDateLabel(selected.date)}</span>
			<span class="muted">no night recorded</span>
		{:else}
			<span class="muted">Tap a night to read it</span>
		{/if}
	</figcaption>

	<div class="plot" bind:clientWidth={width}>
		<svg
			viewBox="0 0 {width} {HEIGHT}"
			{width}
			height={HEIGHT}
			role="img"
			aria-label="Each night as a bar from bedtime at the top to waking at the bottom. Level bar ends mean regular sleep times."
			onpointermove={read}
			onpointerdown={read}
			onpointerleave={stopReading}
		>
			{#each hours as minute (minute)}
				<line class="grid" x1={AXIS} y1={y(minute)} x2={width} y2={y(minute)} />
				<text class="tick" x={0} y={y(minute) + 4}>{formatClock(minute)}</text>
			{/each}

			{#each days as day, index (day.date)}
				{#if day.night}
					<rect
						class="night"
						class:selected={reading === index}
						x={x(index) - bar / 2}
						y={y(day.night.bedMinute)}
						width={bar}
						height={Math.max(y(day.night.wakeMinute) - y(day.night.bedMinute), 1)}
						rx={Math.min(bar / 2, 4)}
					/>
				{/if}
			{/each}

			{#if selected && reading !== null && !selected.night}
				<line class="cursor" x1={x(reading)} y1={PAD_Y} x2={x(reading)} y2={HEIGHT - PAD_Y} />
			{/if}
		</svg>
	</div>

	<div class="axis small muted">
		<span>{days[0] ? dayMonthLabel(days[0].date) : ''}</span>
		<span>{days.at(-1) ? dayMonthLabel(days.at(-1)?.date ?? '') : ''}</span>
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

	.grid,
	.cursor {
		stroke: var(--border);
		stroke-width: 1;
	}

	.tick {
		fill: var(--muted);
		font-size: 0.7rem;
	}

	/* Muted like the raw readings elsewhere; the selected night takes the ink. */
	.night {
		fill: var(--muted);
	}

	.night.selected {
		fill: var(--text);
	}

	.axis {
		display: flex;
		justify-content: space-between;
		padding-left: 38px;
	}
</style>
