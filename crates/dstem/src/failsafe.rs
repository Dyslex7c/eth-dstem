//! Sources: [S1] Dandelion++ stem fail-safe (deadline fluff); [S4] eth/68 fallback.
use crate::tracker::{InFlightRecord, InFlightTracker};

/// Evicts every stem hold whose deadline has passed, under one write lock.
///
/// Scan and removal share that lock, so a concurrent `track` or `remove` cannot
/// see a half-evicted set or broadcast the same tx twice ([S1]). Records come
/// back sorted by hash so a seeded run replays the same fluff order.
pub fn collect_expired_records(tracker: &InFlightTracker, now_ms: u64) -> Vec<InFlightRecord> {
    let mut entries = tracker.entries.write();
    let mut expired = Vec::new();
    entries.retain(|_, record| {
        let due = record.deadline_ms <= now_ms;
        if due {
            expired.push(*record);
        }
        !due
    });
    expired.sort_unstable_by_key(|record| record.tx_hash);
    expired
}
