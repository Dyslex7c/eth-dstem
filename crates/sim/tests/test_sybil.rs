use eth_tx_sim::node::NodeType;
use eth_tx_sim::sybil::inject_sybils;
use eth_tx_sim::topology::NetworkTopology;
use std::collections::BTreeSet;

const NODE_COUNT: usize = 20;
const DEFAULT_MAX_PEERS: usize = 10;
const SYBIL_COUNT: usize = 5;
const TARGET: usize = 0;

#[test]
fn injects_five_sybils_around_target() {
    let mut topo = NetworkTopology::new(NODE_COUNT, DEFAULT_MAX_PEERS);
    topo.nodes[TARGET].node_type = NodeType::Target;
    assert_eq!(topo.nodes.len(), NODE_COUNT);

    let created = inject_sybils(&mut topo, TARGET, SYBIL_COUNT);

    // 5 new nodes exist, appended after the original 20.
    assert_eq!(created.len(), SYBIL_COUNT);
    assert_eq!(
        created,
        (NODE_COUNT..NODE_COUNT + SYBIL_COUNT).collect::<Vec<_>>()
    );
    assert_eq!(topo.nodes.len(), NODE_COUNT + SYBIL_COUNT);
    assert_eq!(topo.adjacency.len(), NODE_COUNT + SYBIL_COUNT);

    // Each new node is a Sybil linked back to the target.
    for &id in &created {
        let sybil = &topo.nodes[id];
        assert_eq!(sybil.id, id);
        assert_eq!(sybil.node_type, NodeType::Sybil, "node {id} is not Sybil");
        assert_eq!(sybil.outbound_peers, vec![TARGET]);
        assert_eq!(sybil.inbound_peers, vec![TARGET]);
        assert_eq!(topo.adjacency[&id], BTreeSet::from([TARGET]));
        assert!(topo.nodes[id].outbound_peers.len() <= topo.nodes[id].max_peers);
    }

    // Target's peer table contains all 5 Sybils in both directions.
    let target = &topo.nodes[TARGET];
    assert_eq!(target.node_type, NodeType::Target);
    let expected: BTreeSet<_> = created.iter().copied().collect();
    let outbound: BTreeSet<_> = target.outbound_peers.iter().copied().collect();
    let inbound: BTreeSet<_> = target.inbound_peers.iter().copied().collect();

    assert_eq!(outbound, expected, "target outbound peers missing Sybils");
    assert_eq!(inbound, expected, "target inbound peers missing Sybils");
    assert_eq!(
        topo.adjacency[&TARGET], expected,
        "target adjacency missing Sybils"
    );
    assert!(target.outbound_peers.len() <= target.max_peers);
    assert!(target.inbound_peers.len() <= target.max_peers);

    // Original honest nodes are untouched.
    for node in &topo.nodes[1..NODE_COUNT] {
        assert_eq!(node.node_type, NodeType::Honest);
        assert!(node.outbound_peers.is_empty());
        assert!(node.inbound_peers.is_empty());
    }
}
