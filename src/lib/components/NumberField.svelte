<script lang="ts">
	import { untrack } from 'svelte';
	import { formatDecimal, parseDecimal } from '$lib/domain/number';

	let {
		label,
		value = $bindable(),
		suffix,
		required = false,
	}: {
		label: string;
		value: number | null;
		suffix?: string;
		required?: boolean;
	} = $props();

	// Seeded once: re-deriving the text from the value would eat a trailing "12," mid-typing.
	let text = $state(untrack(() => formatDecimal(value)));

	const invalid = $derived(text.trim() !== '' && value === null);
</script>

<label class="field">
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
