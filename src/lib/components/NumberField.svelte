<script lang="ts">
	import { untrack } from 'svelte';
	import { formatDecimal, parseDecimal } from '$lib/domain/number';

	let {
		label,
		value = $bindable(),
		suffix,
		required = false,
		tone,
	}: {
		label: string;
		value: number | null;
		suffix?: string;
		required?: boolean;
		/** A macro's own block, so the figure is entered where it is later read. */
		tone?: 'hero' | 'protein' | 'fat' | 'carbs';
	} = $props();

	// Seeded once: re-deriving the text from the value would eat a trailing "12," mid-typing.
	let text = $state(untrack(() => formatDecimal(value)));

	const invalid = $derived(text.trim() !== '' && value === null);
</script>

<label class="field {tone ?? ''}" class:block={tone !== undefined}>
	<span>{label}{suffix ? ` (${suffix})` : ''}</span>
	<input
		type="text"
		inputmode="decimal"
		autocomplete="off"
		value={text}
		{required}
		aria-invalid={invalid}
		oninput={(event) => {
			text = event.currentTarget.value;
			value = parseDecimal(text);
		}}
		onblur={() => {
			if (value !== null) text = formatDecimal(value);
		}}
	/>
</label>

<style>
	.block {
		border-radius: var(--radius);
		padding: 0.7rem 0.9rem;
		color: var(--on-macro);
		gap: 0.1rem;
	}

	.block input {
		min-height: 0;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		font-family: var(--font-display);
		font-size: 1.6rem;
		font-weight: 800;
	}

	.block input:focus {
		outline: 2px solid currentColor;
		outline-offset: 2px;
	}

	.block input[aria-invalid='true'] {
		outline: 2px solid var(--danger);
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
</style>
