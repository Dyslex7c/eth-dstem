use eth_tx_sim::adversary::timing::TimingEstimator;
use eth_tx_sim::node::NodeType;
use eth_tx_sim::topology::NetworkTopology;

const NODE_COUNT: usize = 5;
const SPY_A: usize = 3;
const SPY_B: usize = 4;
const LINK_LATENCY_MS: u64 = 10;

/// Graph: 0 - 1 - 2 - SpyA(3), plus 0 - SpyB(4).
/// SpyA is 3 hops from node 0; SpyB is 1 hop.
#[test]
fn node_zero_outscores_node_two_from_observed_arrivals() {
    let mut topo = NetworkTopology::new(NODE_COUNT, 4);
    for (a, b) in [(0, 1), (1, 2), (2, SPY_A), (0, SPY_B)] {
        topo.add_symmetric_link(a, b).unwrap();
    }
    topo.nodes[SPY_A].node_type = NodeType::Sybil;
    topo.nodes[SPY_B].node_type = NodeType::Sybil;

    let mut estimator = TimingEstimator::new();
    estimator.record_observation(SPY_B, 10);
    estimator.record_observation(SPY_A, 30);

    let score_node_0 = estimator.score_candidate_origin(&topo, 0, LINK_LATENCY_MS);
    let score_node_2 = estimator.score_candidate_origin(&topo, 2, LINK_LATENCY_MS);

    // From node 0: SpyB expects 1 hop (10ms, observed 10) and SpyA expects
    // 3 hops (30ms, observed 30), so every squared error is zero.
    assert_eq!(score_node_0, 0.0);

    // From node 2: SpyA is 1 hop (expects 10, observed 30) and SpyB is 3 hops
    // (expects 30, observed 10); 20^2 + 20^2 = 800.
    assert_eq!(score_node_2, 800.0);

    assert!(
        score_node_0 < score_node_2,
        "node 0 should score lower than node 2: {score_node_0} vs {score_node_2}"
    );

    // Node 0 is also the strict best candidate overall.
    let best = (0..NODE_COUNT)
        .map(|candidate| {
            (
                estimator.score_candidate_origin(&topo, candidate, LINK_LATENCY_MS),
                candidate,
            )
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    assert_eq!(best, (0.0, 0));
}
