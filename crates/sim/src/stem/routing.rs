use crate::node::NodeId;
use crate::stem::config::StemConfig;
use crate::stem::digraph::StemTopology;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutingPhase {
    Stem,
    Fluff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutingDecision {
    ForwardStem { next_hop: NodeId },
    BroadcastFluff,
}

pub fn evaluate_routing_step<R: Rng>(
    rng: &mut R,
    current_node: NodeId,
    stem_topo: &StemTopology,
    config: &StemConfig,
) -> RoutingDecision {
    let stem_peers = stem_topo.get_stem_peers(current_node);
    if rng.gen_range(0.0..1.0) < config.stem_probability && !stem_peers.is_empty() {
        let next_hop = stem_peers[rng.gen_range(0..stem_peers.len())];
        RoutingDecision::ForwardStem { next_hop }
    } else {
        RoutingDecision::BroadcastFluff
    }
}
