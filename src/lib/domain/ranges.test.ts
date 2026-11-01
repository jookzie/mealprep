import { expect, test } from "bun:test";
import { belowFloor, gramRange } from "./ranges";

test("protein scales both ends with body weight", () => {
  const range = gramRange("proteinG", 80);
  expect(range?.low).toBeCloseTo(128);
  expect(range?.high).toBeCloseTo(176);
});

test("fat and carbs have a floor only", () => {
  expect(gramRange("fatG", 80)).toEqual({ low: 40, high: undefined });
  expect(gramRange("carbohydratesG", 80)).toEqual({ low: 80, high: undefined });
});

test("energy has no range", () => {
  expect(gramRange("energyKcal", 80)).toBeNull();
});

test("only fat under its floor warns", () => {
  expect(belowFloor("fatG", 39, 80)).toBe(true);
  expect(belowFloor("fatG", 40, 80)).toBe(false);
  expect(belowFloor("carbohydratesG", 10, 80)).toBe(false);
});
