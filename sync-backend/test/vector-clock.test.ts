import { describe, expect, test } from "bun:test";
import { compareClocks, type ClockComparison } from "../src/vector-clock.ts";

describe("compareClocks", () => {
  test("identical clocks return 'identical'", () => {
    const a = { node1: 1, node2: 2 };
    const b = { node1: 1, node2: 2 };
    expect(compareClocks(a, b)).toBe("identical" satisfies ClockComparison);
  });

  test("A descends from B (A has all of B's values and at least one higher)", () => {
    const a = { node1: 3, node2: 2 };
    const b = { node1: 1, node2: 2 };
    expect(compareClocks(a, b)).toBe(
      "a_descends_from_b" satisfies ClockComparison,
    );
  });

  test("B descends from A (B has all of A's values and at least one higher)", () => {
    const a = { node1: 1, node2: 2 };
    const b = { node1: 1, node2: 5 };
    expect(compareClocks(a, b)).toBe(
      "b_descends_from_a" satisfies ClockComparison,
    );
  });

  test("concurrent clocks (each has at least one value higher than the other)", () => {
    const a = { node1: 3, node2: 1 };
    const b = { node1: 1, node2: 3 };
    expect(compareClocks(a, b)).toBe("concurrent" satisfies ClockComparison);
  });

  test("empty clocks are identical", () => {
    expect(compareClocks({}, {})).toBe("identical" satisfies ClockComparison);
  });

  test("clock with extra key descends from the one without it", () => {
    const a = { node1: 1, node2: 1 };
    const b = { node1: 1 };
    expect(compareClocks(a, b)).toBe(
      "a_descends_from_b" satisfies ClockComparison,
    );
  });
});
