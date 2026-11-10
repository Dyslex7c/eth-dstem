use crate::node::{Node, NodeId, NodeType};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkTopology {
    pub nodes: Vec<Node>,
    pub adjacency: BTreeMap<NodeId, BTreeSet<NodeId>>,
}

impl NetworkTopology {
    pub fn new(size: usize, default_max_peers: usize) -> Self {
        Self {
            nodes: (0..size)
                .map(|id| Node::new(id, NodeType::Honest, default_max_peers))
                .collect(),
            adjacency: (0..size).map(|id| (id, BTreeSet::new())).collect(),
        }
    }

    pub fn add_symmetric_link(&mut self, a: NodeId, b: NodeId) -> Result<(), &'static str> {
        if a >= self.nodes.len() || b >= self.nodes.len() {
            return Err("node id out of bounds");
        }
        if a == b {
            return Err("self-links are not allowed");
        }
        if self.nodes[a].outbound_peers.contains(&b) || self.nodes[b].outbound_peers.contains(&a) {
            return Err("link already exists");
        }
        if self.nodes[a].outbound_peers.len() >= self.nodes[a].max_peers
            || self.nodes[b].outbound_peers.len() >= self.nodes[b].max_peers
        {
            return Err("outbound peer limit reached");
        }

        self.nodes[a].outbound_peers.push(b);
        self.nodes[b].inbound_peers.push(a);
        self.nodes[b].outbound_peers.push(a);
        self.nodes[a].inbound_peers.push(b);
        self.adjacency.entry(a).or_default().insert(b);
        self.adjacency.entry(b).or_default().insert(a);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_isolated_honest_nodes() {
        let topology = NetworkTopology::new(3, 2);

        assert_eq!(topology.nodes.len(), 3);
        assert_eq!(topology.adjacency.len(), 3);
        for (id, node) in topology.nodes.iter().enumerate() {
            assert_eq!(node, &Node::new(id, NodeType::Honest, 2));
            assert!(topology.adjacency[&id].is_empty());
        }
    }

    #[test]
    fn empty_topology_rejects_links() {
        let mut topology = NetworkTopology::new(0, 2);
        let before = topology.clone();

        assert!(topology.nodes.is_empty());
        assert!(topology.adjacency.is_empty());
        assert_eq!(
            topology.add_symmetric_link(0, 1),
            Err("node id out of bounds")
        );
        assert_eq!(topology, before);
    }

    #[test]
    fn links_update_both_directions_and_allow_exact_capacity() {
        let mut topology = NetworkTopology::new(3, 2);

        assert_eq!(topology.add_symmetric_link(0, 1), Ok(()));
        assert_eq!(topology.add_symmetric_link(0, 2), Ok(()));
        assert_eq!(topology.nodes[0].outbound_peers, vec![1, 2]);
        assert_eq!(topology.nodes[0].inbound_peers, vec![1, 2]);
        assert_eq!(topology.adjacency[&0], BTreeSet::from([1, 2]));
        for id in [1, 2] {
            assert_eq!(topology.nodes[id].outbound_peers, vec![0]);
            assert_eq!(topology.nodes[id].inbound_peers, vec![0]);
            assert_eq!(topology.adjacency[&id], BTreeSet::from([0]));
        }
    }

    #[test]
    fn full_endpoint_rejects_link_without_mutation_in_either_direction() {
        let mut topology = NetworkTopology::new(3, 1);
        topology.add_symmetric_link(0, 1).unwrap();
        let before = topology.clone();

        for (a, b) in [(0, 2), (2, 0)] {
            assert_eq!(
                topology.add_symmetric_link(a, b),
                Err("outbound peer limit reached")
            );
            assert_eq!(topology, before);
        }
    }

    #[test]
    fn zero_capacity_at_either_endpoint_prevents_links() {
        for max_peers in [[0, 0], [0, 1], [1, 0]] {
            let mut topology = NetworkTopology::new(2, 1);
            for (node, limit) in topology.nodes.iter_mut().zip(max_peers) {
                node.max_peers = limit;
            }
            let before = topology.clone();

            assert_eq!(
                topology.add_symmetric_link(0, 1),
                Err("outbound peer limit reached")
            );
            assert_eq!(topology, before);
        }
    }

    #[test]
    fn duplicate_links_are_rejected_in_both_directions() {
        let mut topology = NetworkTopology::new(2, 2);
        topology.add_symmetric_link(0, 1).unwrap();
        let before = topology.clone();

        for (a, b) in [(0, 1), (1, 0)] {
            assert_eq!(
                topology.add_symmetric_link(a, b),
                Err("link already exists")
            );
            assert_eq!(topology, before);
        }
    }

    #[test]
    fn invalid_ids_and_self_links_leave_topology_unchanged() {
        let mut topology = NetworkTopology::new(2, 2);
        let before = topology.clone();

        for (a, b) in [(0, 2), (2, 0), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(
                topology.add_symmetric_link(a, b),
                Err("node id out of bounds")
            );
            assert_eq!(topology, before);
        }
        assert_eq!(
            topology.add_symmetric_link(0, 0),
            Err("self-links are not allowed")
        );
        assert_eq!(topology, before);
    }
}
