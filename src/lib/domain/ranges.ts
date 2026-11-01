import type { MacroKey } from "./macros";

/**
 * Reference ranges in grams per kilogram of body weight, drawn as brackets on the meters.
 * Protein is the range usually cited for hypertrophy; fat and carbs have a floor only — the
 * fat floor is what hormonal function is commonly said to need, the carb floor a common
 * lower bound for training. Informational, like every other figure here (`TG-3`).
 */
export const RANGES_G_PER_KG: Partial<
  Record<MacroKey, { low: number; high?: number }>
> = {
  proteinG: { low: 1.6, high: 2.2 },
  fatG: { low: 0.5 },
  carbohydratesG: { low: 1 },
};

export type GramRange = { low: number; high?: number };

/** The range in grams for this body weight, or null when the macro has none. */
export function gramRange(key: MacroKey, kilograms: number): GramRange | null {
  const range = RANGES_G_PER_KG[key];
  if (range === undefined) return null;
  return {
    low: range.low * kilograms,
    high: range.high === undefined ? undefined : range.high * kilograms,
  };
}

/** Only fat warns: under its floor is the one case the range is about health, not progress. */
export function belowFloor(
  key: MacroKey,
  grams: number,
  kilograms: number,
): boolean {
  const range = gramRange(key, kilograms);
  return key === "fatG" && range !== null && grams < range.low;
}
