use crate::node::{Node, NodeId, NodeType};
use crate::topology::NetworkTopology;

pub fn inject_sybils(topo: &mut NetworkTopology, target: NodeId, count: usize) -> Vec<NodeId> {
    if target >= topo.nodes.len() {
        return Vec::new();
    }

    let sybil_max_peers = topo.nodes[target].max_peers;
    let mut created = Vec::with_capacity(count);

    for _ in 0..count {
        let id = topo.nodes.len();
        topo.nodes
            .push(Node::new(id, NodeType::Sybil, sybil_max_peers));
        topo.adjacency.insert(id, Default::default());

        match topo.add_symmetric_link(target, id) {
            Ok(()) => created.push(id),
            Err(_) => {
                topo.nodes.pop();
                topo.adjacency.remove(&id);
                break;
            }
        }
    }

    created
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn injects_requested_sybils_and_links_both_directions() {
        let mut topology = NetworkTopology::new(5, 10);
        let created = inject_sybils(&mut topology, 0, 3);

        assert_eq!(created, vec![5, 6, 7]);
        assert_eq!(topology.nodes.len(), 8);
        assert_eq!(topology.adjacency.len(), 8);

        for id in &created {
            let sybil = &topology.nodes[*id];
            assert_eq!(sybil.id, *id);
            assert_eq!(sybil.node_type, NodeType::Sybil);
            assert_eq!(sybil.outbound_peers, vec![0]);
            assert_eq!(sybil.inbound_peers, vec![0]);
            assert_eq!(topology.adjacency[id], BTreeSet::from([0]));
            assert!(topology.nodes[0].outbound_peers.contains(id));
            assert!(topology.nodes[0].inbound_peers.contains(id));
            assert!(topology.adjacency[&0].contains(id));
        }
    }

    #[test]
    fn respects_target_outbound_capacity_without_overflow() {
        let mut topology = NetworkTopology::new(3, 2);
        topology.add_symmetric_link(0, 1).unwrap();

        let created = inject_sybils(&mut topology, 0, 3);

        assert_eq!(created.len(), 1);
        assert!(topology.nodes[0].outbound_peers.len() <= topology.nodes[0].max_peers);
        assert_eq!(topology.nodes[0].outbound_peers.len(), 2);
        assert_eq!(topology.nodes.len(), 4);
        assert_eq!(topology.adjacency.len(), 4);
    }

    #[test]
    fn returns_empty_when_target_capacity_is_already_full() {
        let mut topology = NetworkTopology::new(2, 1);
        topology.add_symmetric_link(0, 1).unwrap();
        let before = topology.clone();

        let created = inject_sybils(&mut topology, 0, 4);

        assert!(created.is_empty());
        assert_eq!(topology, before);
    }

    #[test]
    fn rejects_out_of_bounds_target_and_zero_count() {
        let mut topology = NetworkTopology::new(3, 10);
        let before = topology.clone();

        assert!(inject_sybils(&mut topology, 3, 2).is_empty());
        assert!(inject_sybils(&mut topology, usize::MAX, 2).is_empty());
        assert_eq!(topology, before);

        assert!(inject_sybils(&mut topology, 0, 0).is_empty());
        assert_eq!(topology, before);
    }

    #[test]
    fn sybils_never_exceed_their_own_peer_limit() {
        let mut topology = NetworkTopology::new(4, 3);
        let created = inject_sybils(&mut topology, 2, 3);

        assert_eq!(created.len(), 3);
        for node in &topology.nodes {
            assert!(node.outbound_peers.len() <= node.max_peers);
            assert!(node.inbound_peers.len() <= node.max_peers);
        }
    }
}
