#![doc = "Sources: user-specified expiry fixtures; dstem/src/failsafe.rs:9-20."]
use dstem::failsafe::collect_expired_records;
use dstem::tracker::{InFlightRecord, InFlightTracker};

#[test]
fn expiry_collects_only_due_records_and_drains_tracker() {
    let tracker = InFlightTracker::default();
    let a = InFlightRecord {
        tx_hash: [0xaa; 32],
        forwarded_to: [1; 64],
        deadline_ms: 1_000,
    };
    let b = InFlightRecord {
        tx_hash: [0xbb; 32],
        deadline_ms: 3_000,
        ..a
    };
    tracker.track(a);
    tracker.track(b);
    assert_eq!(collect_expired_records(&tracker, 2_000), vec![a]);
    assert_eq!(tracker.remove(&a.tx_hash), None);
    assert!(collect_expired_records(&tracker, 2_000).is_empty());
    assert_eq!(collect_expired_records(&tracker, 4_000), vec![b]);
    assert_eq!(tracker.remove(&b.tx_hash), None);
    assert!(collect_expired_records(&tracker, u64::MAX).is_empty());
}
