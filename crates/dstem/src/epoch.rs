//! Sources: [S1] Dandelion++ per-epoch stem-set reshuffle (degree-bounded line graph).
use crate::peers::PeerRegistry;
use crate::route::StemRouteTable;
use rand::Rng;

/// Rebuilds the stem forwarding set for a new epoch ([S1]).
///
/// Draws up to `degree` distinct peers by partial Fisher-Yates: each pick is
/// uniform over the peers not yet chosen, and the shuffle runs on a private
/// snapshot, so `route_table` readers only ever see a complete old or complete
/// new set. Selection depends solely on `rng`, so a seeded RNG replays exactly.
pub fn reshuffle_stem_routes<R: Rng>(
    rng: &mut R,
    registry: &PeerRegistry,
    route_table: &StemRouteTable,
    degree: usize,
) {
    let mut peers = registry.peers();
    let k = degree.min(peers.len());
    for i in 0..k {
        let j = i + rng.gen_range(0..peers.len() - i);
        peers.swap(i, j);
    }
    peers.truncate(k);
    route_table.update_routes(peers);
}
