//! Sources: [S1] Dandelion++ stem fail-safe; [S4] eth/68 pooled-tx follow-up.
use dstem::tracker::{InFlightRecord, InFlightTracker};
use dstem::types::{PeerId, TxHash};

#[test]
fn track_then_remove_returns_original_record_once() {
    let tracker = InFlightTracker::default();
    let tx: TxHash = [0xaa; 32];
    let peer: PeerId = [1; 64];
    let record = InFlightRecord {
        tx_hash: tx,
        forwarded_to: peer,
        deadline_ms: 5_000,
    };
    tracker.track(record);
    assert_eq!(tracker.remove(&tx), Some(record));
    assert_eq!(tracker.remove(&tx), None, "already resolved");
}
