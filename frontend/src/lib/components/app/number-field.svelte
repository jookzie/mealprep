<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { formatDecimal, parseDecimal, step } from '$lib/domain/number';

	/*
	 * Everything numeric in the app goes through here: the value stays a number or null,
	 * and the raw text is what the user sees.
	 *
	 * Deliberately type="text" with inputmode="decimal" rather than type="number". A
	 * focused number input mutates its value when the page is scrolled under the
	 * pointer, which silently corrupts a figure the user believed they had finished
	 * with. GOV.UK avoids type="number" for the same reason.
	 */
	let {
		value = $bindable(),
		label,
		id,
		suffix,
		required = false,
		placeholder,
		ariaLabel,
		ref = $bindable(null),
		onCommit
	}: {
		value: number | null;
		label?: string;
		id: string;
		suffix?: string;
		required?: boolean;
		placeholder?: string;
		/** For a field inside a repeated row, where a visible label would be noise. */
		ariaLabel?: string;
		ref?: HTMLInputElement | null;
		/** Enter in the field. The composition editors use it to commit a row. */
		onCommit?: () => void;
	} = $props();

	let text = $state(formatDecimal(value));

	function onInput(event: Event) {
		text = (event.currentTarget as HTMLInputElement).value;
		value = parseDecimal(text);
	}

	// Correcting mid-keystroke fights a user who has cleared the field to retype it, so
	// the text is only normalised once they have left it.
	function onBlur() {
		text = formatDecimal(value);
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && onCommit) {
			event.preventDefault();
			onCommit();
			return;
		}
		if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
		event.preventDefault();
		value = step(value, event.key === 'ArrowUp' ? 1 : -1, {
			shift: event.shiftKey,
			ctrl: event.ctrlKey
		});
		text = formatDecimal(value);
	}
</script>

<div class="space-y-2">
	{#if label}
		<Label for={id}>{label}</Label>
	{/if}
	<div class="relative">
		<Input
			bind:ref
			{id}
			type="text"
			inputmode="decimal"
			autocomplete="off"
			{required}
			{placeholder}
			aria-label={ariaLabel}
			value={text}
			oninput={onInput}
			onblur={onBlur}
			onkeydown={onKeydown}
			class={suffix ? 'pr-12 tabular-nums' : 'tabular-nums'}
		/>
		{#if suffix}
			<!-- The unit is inert text derived from the product, never a selector: it is
			     what makes the whole g/oz/ml class of mistake impossible (TG-4). -->
			<span
				class="text-muted-foreground pointer-events-none absolute inset-y-0 right-3 flex items-center text-sm"
			>
				{suffix}
			</span>
		{/if}
	</div>
</div>
