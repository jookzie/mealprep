/**
 * Number entry is the app's most repeated interaction, so parsing is one function
 * rather than a `Number()` at every call site.
 *
 * A European keyboard writes `12,5`, and `Number('12,5')` is NaN — which the old
 * guard turned into a silently empty field. Both separators are accepted and
 * normalised here instead.
 */
export function parseDecimal(text: string): number | null {
	const trimmed = text.trim().replace(',', '.');
	if (trimmed === '') return null;
	// Number('') is 0 and Number('1e') is NaN; neither may reach a draft.
	const parsed = Number(trimmed);
	return Number.isFinite(parsed) ? parsed : null;
}

/** What the field shows once the user leaves it: their value, without their comma. */
export function formatDecimal(value: number | null): string {
	return value === null ? '' : String(value);
}

/**
 * Arrow-key stepping, as every spin button does it: plain is ±1, Shift is ±10 and
 * Ctrl is ±0.1. The result is rounded to kill the float dust ±0.1 accumulates.
 */
export function step(
	value: number | null,
	direction: 1 | -1,
	modifiers: { shift?: boolean; ctrl?: boolean } = {},
): number {
	const size = modifiers.shift ? 10 : modifiers.ctrl ? 0.1 : 1;
	return Math.round(((value ?? 0) + direction * size) * 1000) / 1000;
}
