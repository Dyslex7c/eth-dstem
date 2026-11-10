use eth_tx_sim::stem::digraph::build_stem_digraph;
use eth_tx_sim::topology::NetworkTopology;

/// Circulant graph: node `i` links `(i + d) % 30` for `d` in 1..=4, so every node has
/// exactly 8 symmetric outbound peers and the `d = 1` ring keeps the graph connected.
#[test]
fn stem_digraph_has_degree_two_over_registered_peers() {
    let mut topo = NetworkTopology::new(30, 8);
    for i in 0..30 {
        for d in 1..=4 {
            topo.add_symmetric_link(i, (i + d) % 30).unwrap();
        }
    }
    assert!(topo.nodes.iter().all(|n| n.outbound_peers.len() == 8));

    let stem = build_stem_digraph(&topo, 2);

    assert_eq!(stem.anonymity_routes.len(), 30);
    for node in &topo.nodes {
        let known = &topo.adjacency[&node.id];
        let peers = stem.get_stem_peers(node.id);
        assert_eq!(peers.len(), 2, "node {}", node.id);
        assert_ne!(peers[0], peers[1]);
        for peer in peers {
            assert!(known.contains(peer), "unregistered peer {peer}");
            assert!(node.outbound_peers.contains(peer), "{} -> {peer}", node.id);
        }
    }
}
