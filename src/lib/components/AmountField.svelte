<script lang="ts">
	import { untrack } from 'svelte';
	import { formatDecimal, parseDecimal } from '$lib/domain/number';

	let {
		label,
		value = $bindable(),
		unit,
	}: {
		/** Read by screen readers only: the row the field sits in already names it visibly. */
		label: string;
		value: number | null;
		unit: string;
	} = $props();

	// Seeded once: re-deriving the text from the value would eat a trailing "12," mid-typing.
	let text = $state(untrack(() => formatDecimal(value)));

	const invalid = $derived(text.trim() === '' || value === null || value <= 0);
</script>

<!-- The serving size, inline in its row: the one figure of a composition the user types. -->
<label class="amount" class:invalid>
	<input
		type="text"
		inputmode="decimal"
		autocomplete="off"
		aria-label={label}
		aria-invalid={invalid}
		value={text}
		oninput={(event) => {
			text = event.currentTarget.value;
			value = parseDecimal(text);
		}}
		onblur={() => {
			if (value !== null) text = formatDecimal(value);
		}}
	/>
	<span>{unit}</span>
</label>

<style>
	.amount {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		height: 44px;
		padding: 0 0.6rem;
		border-radius: 14px;
		background: var(--raised);
		border: 2px solid transparent;
		font-weight: 700;
		flex-shrink: 0;
	}

	.amount:focus-within {
		border-color: var(--text);
	}

	.amount.invalid {
		border-color: var(--danger);
	}

	input {
		width: 3.2rem;
		min-height: 0;
		padding: 0;
		border: none;
		background: transparent;
		text-align: right;
		font-weight: 700;
	}

	input:focus {
		outline: none;
	}

	span {
		color: var(--muted);
		font-size: 0.8rem;
	}
</style>
