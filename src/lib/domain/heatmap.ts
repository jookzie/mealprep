/**
 * Column shading for the entity tables.
 *
 * The scale runs between the smallest and largest value *in that column*, so the
 * shading answers "how does this row compare to the others here" rather than
 * implying an absolute verdict. Each column is tinted in its own macro's hue, so
 * intensity reads as more-of-that-macro and never as good or bad — the same rule the
 * meters follow. Every shaded cell also prints its number, so colour is never the
 * only channel carrying the value.
 */
export type Scale = (value: number) => number;

/**
 * A column where every value is equal has no spread to show, so it shades flat at
 * zero rather than painting every cell at full strength.
 */
export function columnScale(values: readonly number[]): Scale {
	if (values.length === 0) return () => 0;
	const min = Math.min(...values);
	const max = Math.max(...values);
	if (max === min) return () => 0;
	return (value) => {
		const t = (value - min) / (max - min);
		return t < 0 ? 0 : t > 1 ? 1 : t;
	};
}

/**
 * Kept well below full opacity: the cell's text has to stay legible against it in
 * both themes, and a saturated grid would shout louder than the figures it carries.
 */
export const MAX_TINT = 0.28;

export function tint(intensity: number): number {
	return intensity * MAX_TINT;
}
