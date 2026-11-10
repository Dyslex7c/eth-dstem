//! Sources: [S1] Dandelion++ picks a per-object stem peer; [S4]/[S6] cap peers.
use crate::types::PeerId;
use parking_lot::RwLock;
use rand::seq::SliceRandom;

/// Stem-phase forwarding set.
///
/// Sampling takes only the *shared* read lock for one O(1) index draw, so
/// concurrent relays never block each other and never block behind a writer
/// for longer than that draw. `update_routes` swaps the whole vector because a
/// peer that drops `eth68s` ([S4]) must stop being chosen as a stem hop.
#[derive(Debug, Default)]
pub struct StemRouteTable {
    pub(crate) active_stem_routes: RwLock<Vec<PeerId>>,
}

impl StemRouteTable {
    /// One uniformly random active stem peer, or `None` when the set is empty.
    pub fn select_forwarding_peer<R: rand::Rng>(&self, rng: &mut R) -> Option<PeerId> {
        self.active_stem_routes.read().choose(rng).copied()
    }

    /// Replaces the stem set wholesale on capability renegotiation.
    pub fn update_routes(&self, new_routes: Vec<PeerId>) {
        *self.active_stem_routes.write() = new_routes;
    }
}
