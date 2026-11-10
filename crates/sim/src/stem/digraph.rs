use crate::node::{NodeId, NodeType};
use crate::topology::NetworkTopology;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StemTopology {
    pub anonymity_routes: HashMap<NodeId, Vec<NodeId>>,
}

pub fn build_stem_digraph(topo: &NetworkTopology, degree: usize) -> StemTopology {
    let mut anonymity_routes = HashMap::with_capacity(topo.nodes.len());
    let honest = NodeType::Honest;
    for node in topo.nodes.iter().filter(|n| n.node_type == honest) {
        let mut peers = node.outbound_peers.clone();
        peers.sort_by_key(|peer| (salt_rank(node.id, *peer), *peer));
        peers.truncate(degree.min(peers.len()));
        anonymity_routes.insert(node.id, peers);
    }
    StemTopology { anonymity_routes }
}

impl StemTopology {
    pub fn get_stem_peers(&self, node: NodeId) -> &[NodeId] {
        self.anonymity_routes.get(&node).map_or(&[], Vec::as_slice)
    }
}

fn salt_rank(node: NodeId, peer: NodeId) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for byte in node.to_le_bytes().iter().chain(peer.to_le_bytes().iter()) {
        h = (h ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
