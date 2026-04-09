export type VectorClock = Record<string, number>;

export type ClockComparison =
  | "identical"
  | "a_descends_from_b"
  | "b_descends_from_a"
  | "concurrent";

export function compareClocks(a: VectorClock, b: VectorClock): ClockComparison {
  const allKeys = new Set([...Object.keys(a), ...Object.keys(b)]);

  let aGreater = false;
  let bGreater = false;

  for (const key of allKeys) {
    const aVal = a[key] ?? 0;
    const bVal = b[key] ?? 0;

    if (aVal > bVal) aGreater = true;
    if (bVal > aVal) bGreater = true;
  }

  if (!aGreater && !bGreater) return "identical";
  if (aGreater && !bGreater) return "a_descends_from_b";
  if (!aGreater && bGreater) return "b_descends_from_a";
  return "concurrent";
}
