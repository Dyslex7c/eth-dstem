use crate::node::NodeId;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FailSafeTracker {
    pub pending_deadlines: HashMap<[u8; 32], (NodeId, u64)>,
}

impl FailSafeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_stem_tx(&mut self, tx_hash: [u8; 32], origin: NodeId, deadline_ms: u64) {
        self.pending_deadlines
            .insert(tx_hash, (origin, deadline_ms));
    }

    pub fn check_expired(&mut self, current_time_ms: u64) -> Vec<([u8; 32], NodeId)> {
        let mut expired = Vec::new();
        self.pending_deadlines.retain(|hash, (origin, deadline)| {
            let is_expired = *deadline <= current_time_ms;
            if is_expired {
                expired.push((*hash, *origin));
            }
            !is_expired
        });
        expired.sort_unstable();
        expired
    }
}
