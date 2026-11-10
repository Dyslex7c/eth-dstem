//! Sources: [S1] Dandelion++ stem fail-safe; [S4] eth/68 pooled-tx follow-up.
use crate::types::{PeerId, TxHash};
use parking_lot::RwLock;
use std::collections::HashMap;

/// A tx sitting on a stem hop: who holds it, and when the hold ends. Once
/// `deadline_ms` passes the holder falls back to `eth/68` diffusion ([S1]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InFlightRecord {
    pub tx_hash: TxHash,
    pub forwarded_to: PeerId,
    pub deadline_ms: u64,
}

/// Stem in-flight set keyed by tx hash. `RwLock` (not `Mutex`) so concurrent
/// relays probing different hashes never serialise on the gossip hot path.
#[derive(Debug, Default)]
pub struct InFlightTracker {
    pub(crate) entries: RwLock<HashMap<TxHash, InFlightRecord>>,
}

impl InFlightTracker {
    /// Records `record` under its own `tx_hash`, superseding any earlier hop.
    pub fn track(&self, record: InFlightRecord) {
        self.entries.write().insert(record.tx_hash, record);
    }

    /// Clears `hash`. `None` means already resolved, so an expiry racing a
    /// fluff must not be treated as real work by the caller.
    pub fn remove(&self, hash: &TxHash) -> Option<InFlightRecord> {
        self.entries.write().remove(hash)
    }
}
