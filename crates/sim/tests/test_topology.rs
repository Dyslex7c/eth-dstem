use eth_tx_sim::topology::NetworkTopology;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

const NODE_COUNT: usize = 50;
const DEFAULT_MAX_PEERS: usize = 10;
const RANDOM_LINK_ATTEMPTS: usize = 4000;

#[test]
fn random_50_node_topology_respects_capacity_and_symmetry() {
    let mut topology = NetworkTopology::new(NODE_COUNT, DEFAULT_MAX_PEERS);
    let mut rng = StdRng::seed_from_u64(eth68_seed());

    let mut added = 0usize;
    for _ in 0..RANDOM_LINK_ATTEMPTS {
        let a = rng.gen_range(0..NODE_COUNT);
        let b = rng.gen_range(0..NODE_COUNT);
        if topology.add_symmetric_link(a, b).is_ok() {
            added += 1;
        }
    }

    assert_eq!(topology.nodes.len(), NODE_COUNT);
    assert_eq!(topology.adjacency.len(), NODE_COUNT);
    assert!(added > 0, "random link generation produced no links");

    for node in &topology.nodes {
        assert_eq!(node.max_peers, DEFAULT_MAX_PEERS);
        assert!(
            node.outbound_peers.len() <= node.max_peers,
            "node {} exceeds outbound peer limit: {} > {}",
            node.id,
            node.outbound_peers.len(),
            node.max_peers
        );
        assert!(
            node.inbound_peers.len() <= node.max_peers,
            "node {} exceeds inbound peer limit: {} > {}",
            node.id,
            node.inbound_peers.len(),
            node.max_peers
        );

        let outbound_unique: BTreeSet<_> = node.outbound_peers.iter().copied().collect();
        assert_eq!(
            outbound_unique.len(),
            node.outbound_peers.len(),
            "node {} has duplicate outbound peers",
            node.id
        );
        assert!(
            !outbound_unique.contains(&node.id),
            "self link on node {}",
            node.id
        );
    }

    for node in &topology.nodes {
        let id = node.id;
        let outbound: BTreeSet<_> = node.outbound_peers.iter().copied().collect();
        let inbound: BTreeSet<_> = node.inbound_peers.iter().copied().collect();
        let adjacency = &topology.adjacency[&id];

        assert_eq!(
            adjacency, &outbound,
            "adjacency of node {id} differs from outbound peers"
        );
        assert_eq!(
            adjacency, &inbound,
            "adjacency of node {id} differs from inbound peers"
        );

        for &peer in &outbound {
            let reverse = &topology.nodes[peer];
            let reverse_outbound: BTreeSet<_> = reverse.outbound_peers.iter().copied().collect();
            let reverse_inbound: BTreeSet<_> = reverse.inbound_peers.iter().copied().collect();

            assert!(
                reverse_outbound.contains(&id),
                "link {id}->{peer} is not registered in {peer}'s outbound peers"
            );
            assert!(
                reverse_inbound.contains(&id),
                "link {id}->{peer} is not registered in {peer}'s inbound peers"
            );
            assert!(
                topology.adjacency[&peer].contains(&id),
                "link {id}->{peer} is not registered in {peer}'s adjacency"
            );
            assert!(
                !topology.adjacency[&peer].contains(&peer),
                "self link in adjacency of node {peer}"
            );
        }
    }
}

// Stable seed so the random graph is reproducible across runs.
const fn eth68_seed() -> u64 {
    0x00657468_00363800
}
