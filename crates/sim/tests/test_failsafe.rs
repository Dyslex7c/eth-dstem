use eth_tx_sim::stem::failsafe::FailSafeTracker;

const ORIGIN: usize = 7;
const TX_1: [u8; 32] = [1; 32];
const TX_2: [u8; 32] = [2; 32];

#[test]
fn check_expired_drains_only_due_deadlines() {
    let mut tracker = FailSafeTracker::new();
    tracker.register_stem_tx(TX_1, ORIGIN, 5_000);
    tracker.register_stem_tx(TX_2, ORIGIN, 10_000);

    let expired = tracker.check_expired(6_000);
    assert_eq!(expired, vec![(TX_1, ORIGIN)]);
    assert!(!tracker.pending_deadlines.contains_key(&TX_1));
    assert_eq!(tracker.pending_deadlines[&TX_2], (ORIGIN, 10_000));

    let expired = tracker.check_expired(11_000);
    assert_eq!(expired, vec![(TX_2, ORIGIN)]);
    assert!(tracker.pending_deadlines.is_empty());
    assert!(tracker.check_expired(11_000).is_empty());
}
