<script lang="ts">
	import Minus from '@lucide/svelte/icons/minus';
	import Plus from '@lucide/svelte/icons/plus';
	import { untrack } from 'svelte';
	import { formatDecimal, parseDecimal } from '$lib/domain/number';

	let {
		label,
		value = $bindable(),
		unit,
		increment,
		tone,
	}: {
		label: string;
		value: number | null;
		unit: string;
		/** What one press of − or + moves the figure by. */
		increment: number;
		/** The block's background; the ink follows from it. */
		tone: 'hero' | 'protein' | 'fat' | 'carbs';
	} = $props();

	// Seeded once and rewritten only by the buttons: re-deriving it from the value would eat
	// a trailing "12," mid-typing.
	let text = $state(untrack(() => formatDecimal(value)));

	function nudge(direction: 1 | -1) {
		const next = Math.max(0, Math.round(((value ?? 0) + direction * increment) * 10) / 10);
		value = next;
		text = formatDecimal(next);
	}
</script>

<label class="stepper {tone}">
	<span class="label">{label} <span class="unit">{unit}</span></span>
	<span class="controls">
		<button type="button" class="icon nudge" aria-label="Lower {label}" onclick={() => nudge(-1)}>
			<Minus size={18} strokeWidth={2.8} aria-hidden="true" />
		</button>
		<span class="entry">
			<input
				type="text"
				inputmode="decimal"
				autocomplete="off"
				required
				value={text}
				aria-invalid={text.trim() !== '' && value === null}
				oninput={(event) => {
					text = event.currentTarget.value;
					value = parseDecimal(text);
				}}
				onblur={() => {
					if (value !== null) text = formatDecimal(value);
				}}
			/>
		</span>
		<button type="button" class="icon nudge" aria-label="Raise {label}" onclick={() => nudge(1)}>
			<Plus size={18} strokeWidth={2.8} aria-hidden="true" />
		</button>
	</span>
</label>

<style>
	/* One row per target: the name on the left, − figure + on the right, where the thumb is. */
	.stepper {
		border-radius: var(--radius);
		padding: 0.55rem 0.6rem 0.55rem 1rem;
		display: flex;
		align-items: center;
		gap: 0.75rem;
		color: var(--on-macro);
	}

	.hero {
		background: var(--hero);
		color: var(--hero-text);
	}

	.protein {
		background: var(--protein);
	}

	.fat {
		background: var(--fat);
	}

	.carbs {
		background: var(--carbs);
	}

	.label {
		flex: 1;
		font-weight: 700;
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.nudge {
		min-width: 36px;
		width: 36px;
		height: 36px;
		min-height: 36px;
		padding: 0;
		background: rgb(255 255 255 / 0.55);
		color: inherit;
		flex-shrink: 0;
	}

	.hero .nudge {
		background: rgb(127 127 127 / 0.25);
	}

	.entry {
		width: 5.5rem;
		display: flex;
	}

	input {
		width: 100%;
		min-width: 0;
		min-height: 0;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		text-align: center;
		font-family: var(--font-display);
		font-size: 1.6rem;
		font-weight: 800;
		letter-spacing: -0.02em;
	}

	input:focus {
		outline: 2px solid currentColor;
		outline-offset: 2px;
	}

	.unit {
		font-size: 0.8rem;
		opacity: 0.7;
	}
</style>
