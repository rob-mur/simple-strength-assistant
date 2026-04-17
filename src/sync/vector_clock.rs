use std::collections::HashMap;

/// A vector clock maps device IDs to their sequence numbers.
/// Used to determine causal ordering between synced states.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct VectorClock(pub HashMap<String, u64>);

/// Relationship between two vector clocks.
#[derive(Debug, Clone, PartialEq)]
pub enum ClockRelation {
    /// Self is strictly newer (descends from other)
    AheadOf,
    /// Other is strictly newer (other descends from self)
    BehindOf,
    /// Neither descends from the other — true concurrency/conflict
    Concurrent,
    /// Clocks are identical
    Equal,
}

impl VectorClock {
    /// Create an empty vector clock.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Returns `true` if the clock has no entries (never been incremented).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return the sequence number for a device, defaulting to 0.
    pub fn get(&self, device_id: &str) -> u64 {
        *self.0.get(device_id).unwrap_or(&0)
    }

    /// Increment this device's counter and return the new value.
    pub fn increment(&mut self, device_id: &str) -> u64 {
        let entry = self.0.entry(device_id.to_string()).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Merge another clock into self, taking the max for each device.
    pub fn merge(&mut self, other: &VectorClock) {
        for (device, &seq) in &other.0 {
            let entry = self.0.entry(device.clone()).or_insert(0);
            if seq > *entry {
                *entry = seq;
            }
        }
    }

    /// Compare this clock against `other` and return their relationship.
    ///
    /// Rules:
    /// - If self ≥ other on every device and > on at least one → `AheadOf`
    /// - If other ≥ self on every device and > on at least one → `BehindOf`
    /// - If neither dominates the other → `Concurrent`
    /// - If both are equal on every device → `Equal`
    pub fn compare(&self, other: &VectorClock) -> ClockRelation {
        // Collect all device IDs from both clocks
        let all_devices: std::collections::HashSet<&String> =
            self.0.keys().chain(other.0.keys()).collect();

        let mut self_ahead = false;
        let mut other_ahead = false;

        for device in all_devices {
            let s = self.get(device);
            let o = other.get(device);
            if s > o {
                self_ahead = true;
            } else if o > s {
                other_ahead = true;
            }
        }

        match (self_ahead, other_ahead) {
            (false, false) => ClockRelation::Equal,
            (true, false) => ClockRelation::AheadOf,
            (false, true) => ClockRelation::BehindOf,
            (true, true) => ClockRelation::Concurrent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(entries: &[(&str, u64)]) -> VectorClock {
        let mut c = VectorClock::new();
        for (device, seq) in entries {
            c.0.insert(device.to_string(), *seq);
        }
        c
    }

    // --- compare ---

    #[test]
    fn test_equal_clocks() {
        let a = clock(&[("d1", 3), ("d2", 5)]);
        let b = clock(&[("d1", 3), ("d2", 5)]);
        assert_eq!(a.compare(&b), ClockRelation::Equal);
    }

    #[test]
    fn test_both_empty_clocks_are_equal() {
        let a = VectorClock::new();
        let b = VectorClock::new();
        assert_eq!(a.compare(&b), ClockRelation::Equal);
    }

    #[test]
    fn test_self_ahead_of_other() {
        let a = clock(&[("d1", 5)]);
        let b = clock(&[("d1", 3)]);
        assert_eq!(a.compare(&b), ClockRelation::AheadOf);
    }

    #[test]
    fn test_self_behind_other() {
        let a = clock(&[("d1", 2)]);
        let b = clock(&[("d1", 7)]);
        assert_eq!(a.compare(&b), ClockRelation::BehindOf);
    }

    #[test]
    fn test_concurrent_clocks() {
        let a = clock(&[("d1", 5), ("d2", 1)]);
        let b = clock(&[("d1", 3), ("d2", 4)]);
        assert_eq!(a.compare(&b), ClockRelation::Concurrent);
    }

    #[test]
    fn test_self_empty_other_nonempty_is_behind() {
        let a = VectorClock::new();
        let b = clock(&[("d1", 2)]);
        assert_eq!(a.compare(&b), ClockRelation::BehindOf);
    }

    #[test]
    fn test_self_nonempty_other_empty_is_ahead() {
        let a = clock(&[("d1", 1)]);
        let b = VectorClock::new();
        assert_eq!(a.compare(&b), ClockRelation::AheadOf);
    }

    #[test]
    fn test_clocks_share_no_device_ids() {
        let a = clock(&[("d1", 3)]);
        let b = clock(&[("d2", 4)]);
        // a has 0 for d2, b has 0 for d1 → concurrent
        assert_eq!(a.compare(&b), ClockRelation::Concurrent);
    }

    #[test]
    fn test_clocks_share_some_device_ids_ahead() {
        // a has more on d1, equal on d2 → AheadOf
        let a = clock(&[("d1", 5), ("d2", 3)]);
        let b = clock(&[("d1", 3), ("d2", 3)]);
        assert_eq!(a.compare(&b), ClockRelation::AheadOf);
    }

    // --- increment ---

    #[test]
    fn test_increment_new_device() {
        let mut c = VectorClock::new();
        let seq = c.increment("dev-a");
        assert_eq!(seq, 1);
        assert_eq!(c.get("dev-a"), 1);
    }

    #[test]
    fn test_increment_existing_device() {
        let mut c = clock(&[("dev-a", 4)]);
        let seq = c.increment("dev-a");
        assert_eq!(seq, 5);
        assert_eq!(c.get("dev-a"), 5);
    }

    // --- merge ---

    #[test]
    fn test_merge_takes_max() {
        let mut a = clock(&[("d1", 5), ("d2", 2)]);
        let b = clock(&[("d1", 3), ("d2", 7)]);
        a.merge(&b);
        assert_eq!(a.get("d1"), 5);
        assert_eq!(a.get("d2"), 7);
    }

    #[test]
    fn test_is_empty_on_new_clock() {
        let c = VectorClock::new();
        assert!(c.is_empty());
    }

    #[test]
    fn test_is_empty_false_after_increment() {
        let mut c = VectorClock::new();
        c.increment("dev-a");
        assert!(!c.is_empty());
    }

    #[test]
    fn test_merge_adds_new_devices() {
        let mut a = clock(&[("d1", 3)]);
        let b = clock(&[("d2", 4)]);
        a.merge(&b);
        assert_eq!(a.get("d1"), 3);
        assert_eq!(a.get("d2"), 4);
    }
}
