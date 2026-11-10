use crate::node::NodeId;
use crate::stem::digraph::StemTopology;
use crate::stem::routing::RoutingDecision;

pub fn handle_stem_forward_failure(
    tx_hash: [u8; 32],
    current_node: NodeId,
    failed_peer: NodeId,
    stem_topo: &mut StemTopology,
) -> RoutingDecision {
    let _ = tx_hash;
    let Some(peers) = stem_topo.anonymity_routes.get_mut(&current_node) else {
        return RoutingDecision::BroadcastFluff;
    };
    peers.retain(|peer| *peer != failed_peer);
    match peers.first() {
        Some(next_hop) => RoutingDecision::ForwardStem {
            next_hop: *next_hop,
        },
        None => RoutingDecision::BroadcastFluff,
    }
}
