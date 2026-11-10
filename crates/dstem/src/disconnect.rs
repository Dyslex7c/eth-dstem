//! Sources: [S1] Dandelion++ stem fail-safe; [S4] eth/68 capability membership.
use crate::peers::PeerRegistry;
use crate::route::StemRouteTable;
use crate::tracker::InFlightTracker;
use crate::types::{PeerId, TxHash};

/// Removes a disconnected peer and returns its stranded txs for fluffing ([S1]).
///
/// Registry, route, and flight locks are held throughout, so readers cannot
/// observe partial cleanup. Lock order is registry, routes, flights.
pub fn handle_peer_drop(
    dropped: &PeerId,
    registry: &PeerRegistry,
    route_table: &StemRouteTable,
    tracker: &InFlightTracker,
) -> Vec<TxHash> {
    let mut peers = registry.capable_peers.write();
    let mut routes = route_table.active_stem_routes.write();
    let mut flights = tracker.entries.write();
    peers.retain(|peer| peer != dropped);
    routes.retain(|peer| peer != dropped);
    let mut stranded = Vec::new();
    flights.retain(|hash, record| {
        let keep = record.forwarded_to != *dropped;
        if !keep {
            stranded.push(*hash);
        }
        keep
    });
    stranded.sort_unstable();
    stranded
}
