//! Sources: [S6] geth peer set tracks per-capability peers; [S4] `eth` cap negotiation per peer.
use crate::types::PeerId;
use parking_lot::RwLock;

/// Peers that negotiated `eth68s` ([S4]/[S6]); guard is `Send`, never poisoned.
#[derive(Debug, Default)]
pub struct PeerRegistry {
    pub(crate) capable_peers: RwLock<Vec<PeerId>>,
}
impl PeerRegistry {
    /// Inserts `peer` if absent; returns `true` when newly added.
    pub fn add_peer(&self, peer: PeerId) -> bool {
        let mut peers = self.capable_peers.write();
        if peers.contains(&peer) {
            return false;
        }
        peers.push(peer);
        true
    }
    /// Purges `peer` on disconnect; returns `true` when one was removed.
    pub fn remove_peer(&self, peer: &PeerId) -> bool {
        let mut peers = self.capable_peers.write();
        let before = peers.len();
        peers.retain(|p| p != peer);
        before != peers.len()
    }
    pub fn peer_count(&self) -> usize {
        self.capable_peers.read().len()
    }
    /// Snapshot of the stem-capable set, in insertion order.
    pub fn peers(&self) -> Vec<PeerId> {
        self.capable_peers.read().clone()
    }
}
