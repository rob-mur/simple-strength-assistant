use std::collections::HashMap;

/// A vector clock maps device IDs to their sequence numbers.
///
/// Currently unused — will be consumed by the sync client in #91.
#[allow(dead_code)]
pub(crate) type VectorClock = HashMap<String, u64>;

/// The relationship between two vector clocks A and B.
///
/// Currently unused — will be consumed by the sync client in #91.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClockRelationship {
    /// Both clocks are identical (same entries, same sequence numbers).
    Identical,
    /// Clock A is strictly newer — it descends from B.
    ADescendsFromB,
    /// Clock B is strictly newer — it descends from A.
    BDescendsFromA,
    /// Neither clock subsumes the other; they have diverged.
    Concurrent,
}

/// Compares two vector clocks and returns their causal relationship.
///
/// This is a pure function with no side effects. Calling it multiple times
/// with the same inputs always returns the same result.
///
/// # Rules
/// - For each device seen in either clock, treat an absent entry as sequence 0.
/// - A descends from B iff every device has `a[d] >= b[d]` AND at least one
///   device has `a[d] > b[d]`.
/// - B descends from A iff every device has `b[d] >= a[d]` AND at least one
///   device has `b[d] > a[d]`.
/// - If neither strictly dominates the other and they differ, they are
///   `Concurrent`.
/// - If all entries are equal (including both empty), they are `Identical`.
///
/// Currently unused — will be consumed by the sync client in #91.
#[allow(dead_code)]
pub(crate) fn compare_vector_clocks(a: &VectorClock, b: &VectorClock) -> ClockRelationship {
    let mut a_greater = false;
    let mut b_greater = false;

    // First pass: iterate over all entries in `a`, looking up each in `b`.
    for (device, &seq_a) in a {
        let seq_b = b.get(device).copied().unwrap_or(0);
        if seq_a > seq_b {
            a_greater = true;
        } else if seq_b > seq_a {
            b_greater = true;
        }
    }

    // Second pass: check entries in `b` that are absent from `a`.
    for (device, &seq_b) in b {
        if !a.contains_key(device) && seq_b > 0 {
            b_greater = true;
        }
    }

    match (a_greater, b_greater) {
        (false, false) => ClockRelationship::Identical,
        (true, false) => ClockRelationship::ADescendsFromB,
        (false, true) => ClockRelationship::BDescendsFromA,
        (true, true) => ClockRelationship::Concurrent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(entries: &[(&str, u64)]) -> VectorClock {
        entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    // QA item: Given two identical clocks, the function reports them as identical.
    #[test]
    fn identical_clocks_are_reported_as_identical() {
        let a = clock(&[("device-1", 3), ("device-2", 7)]);
        let b = clock(&[("device-1", 3), ("device-2", 7)]);
        assert_eq!(compare_vector_clocks(&a, &b), ClockRelationship::Identical);
    }

    // QA item: Both empty clocks are identical.
    #[test]
    fn two_empty_clocks_are_identical() {
        let a = clock(&[]);
        let b = clock(&[]);
        assert_eq!(compare_vector_clocks(&a, &b), ClockRelationship::Identical);
    }

    // QA item: Given clock A with higher sequence on at least one device and no
    // lower numbers, the function reports A descends from B.
    #[test]
    fn a_with_higher_sequence_descends_from_b() {
        let a = clock(&[("device-1", 5), ("device-2", 3)]);
        let b = clock(&[("device-1", 3), ("device-2", 3)]);
        assert_eq!(
            compare_vector_clocks(&a, &b),
            ClockRelationship::ADescendsFromB
        );
    }

    // QA item: Given clock B with higher sequence on at least one device and no
    // lower numbers, the function reports B descends from A.
    #[test]
    fn b_with_higher_sequence_descends_from_a() {
        let a = clock(&[("device-1", 2)]);
        let b = clock(&[("device-1", 5)]);
        assert_eq!(
            compare_vector_clocks(&a, &b),
            ClockRelationship::BDescendsFromA
        );
    }

    // QA item: Given clocks where each has at least one device with a higher
    // sequence number, the function reports them as concurrent/diverged.
    #[test]
    fn clocks_where_each_leads_on_different_devices_are_concurrent() {
        let a = clock(&[("device-1", 5), ("device-2", 1)]);
        let b = clock(&[("device-1", 3), ("device-2", 4)]);
        assert_eq!(compare_vector_clocks(&a, &b), ClockRelationship::Concurrent);
    }

    // QA item: Given one empty clock and one non-empty clock, the non-empty
    // clock descends from the empty one (not concurrent).
    #[test]
    fn non_empty_clock_descends_from_empty_clock() {
        let a = clock(&[("device-1", 1)]);
        let b = clock(&[]);
        assert_eq!(
            compare_vector_clocks(&a, &b),
            ClockRelationship::ADescendsFromB
        );
    }

    #[test]
    fn empty_clock_is_descended_from_by_non_empty_clock() {
        let a = clock(&[]);
        let b = clock(&[("device-1", 2)]);
        assert_eq!(
            compare_vector_clocks(&a, &b),
            ClockRelationship::BDescendsFromA
        );
    }

    // QA item: Given two clocks with no shared device IDs and both non-empty,
    // the function reports them as concurrent/diverged.
    #[test]
    fn clocks_with_no_shared_devices_are_concurrent() {
        let a = clock(&[("device-1", 3)]);
        let b = clock(&[("device-2", 5)]);
        assert_eq!(compare_vector_clocks(&a, &b), ClockRelationship::Concurrent);
    }

    // QA item: Given two clocks sharing some but not all device IDs, the function
    // correctly classifies the relationship using all present device entries.
    #[test]
    fn partial_overlap_a_dominates_when_absent_entries_are_treated_as_zero() {
        // a has device-1 and device-2; b only has device-1 (device-2 absent = 0)
        // a[device-1]=3 >= b[device-1]=3, a[device-2]=2 > b[device-2]=0 → A descends
        let a = clock(&[("device-1", 3), ("device-2", 2)]);
        let b = clock(&[("device-1", 3)]);
        assert_eq!(
            compare_vector_clocks(&a, &b),
            ClockRelationship::ADescendsFromB
        );
    }

    #[test]
    fn partial_overlap_concurrent_when_each_leads_on_their_own_device() {
        // a[device-1]=5 > b[device-1]=0, b[device-2]=4 > a[device-2]=0 → concurrent
        let a = clock(&[("device-1", 5)]);
        let b = clock(&[("device-2", 4)]);
        assert_eq!(compare_vector_clocks(&a, &b), ClockRelationship::Concurrent);
    }

    // QA item: The function is pure — same inputs always produce same outputs.
    #[test]
    fn function_is_pure_same_inputs_produce_same_outputs() {
        let a = clock(&[("device-x", 10), ("device-y", 2)]);
        let b = clock(&[("device-x", 8), ("device-y", 5)]);

        let result1 = compare_vector_clocks(&a, &b);
        let result2 = compare_vector_clocks(&a, &b);
        let result3 = compare_vector_clocks(&a, &b);

        assert_eq!(result1, ClockRelationship::Concurrent);
        assert_eq!(result2, ClockRelationship::Concurrent);
        assert_eq!(result3, ClockRelationship::Concurrent);

        // The input clocks must be unchanged.
        assert_eq!(a.get("device-x"), Some(&10));
        assert_eq!(b.get("device-y"), Some(&5));
    }
}
