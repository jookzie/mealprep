<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	// A number input yields '' for an empty box and NaN for '1e', so binding a number
	// straight to one puts NaN into the draft. Everything numeric goes through here:
	// the value stays a number or null, and the raw text is what the user sees.
	let {
		value = $bindable(),
		label,
		id,
		suffix,
		min = 0,
		step = 'any',
		required = false,
		placeholder
	}: {
		value: number | null;
		label?: string;
		id: string;
		suffix?: string;
		min?: number;
		step?: string | number;
		required?: boolean;
		placeholder?: string;
	} = $props();

	let text = $state(value === null ? '' : String(value));

	function onInput(event: Event) {
		text = (event.currentTarget as HTMLInputElement).value;
		const trimmed = text.trim();
		if (trimmed === '') {
			value = null;
			return;
		}
		const parsed = Number(trimmed);
		value = Number.isFinite(parsed) ? parsed : null;
	}
</script>

<div class="space-y-2">
	{#if label}
		<Label for={id}>{label}</Label>
	{/if}
	<div class="relative">
		<Input
			{id}
			type="number"
			inputmode="decimal"
			{min}
			{step}
			{required}
			{placeholder}
			value={text}
			oninput={onInput}
			class={suffix ? 'pr-12' : undefined}
		/>
		{#if suffix}
			<span
				class="text-muted-foreground pointer-events-none absolute inset-y-0 right-3 flex items-center text-sm"
			>
				{suffix}
			</span>
		{/if}
	</div>
</div>
