use eth_tx_sim::harness::{run_trial_seeded, Protocol, TrialTrace};
use eth_tx_sim::node::NodeType;
use eth_tx_sim::stem::{config::StemConfig, digraph::build_stem_digraph};
use eth_tx_sim::topology::NetworkTopology;

const NODES: usize = 50;
const SYBILS: usize = 5;
/// Fixed so the stem coin is reproducible. Stem peers may be Sybils (a real node cannot
/// tell them apart), so this seed is one whose first stem hop lands on an honest peer.
const SEED: u64 = 0;

#[test]
fn stem_hop_precedes_fluff_which_then_covers_the_network() {
    let mut topo = NetworkTopology::new(NODES, 8);
    for i in 0..NODES {
        for step in [1, 7] {
            topo.add_symmetric_link(i, (i + step) % NODES).unwrap();
        }
    }
    for id in (NODES - SYBILS)..NODES {
        topo.nodes[id].node_type = NodeType::Sybil;
    }
    let config = StemConfig::default();
    let routes = build_stem_digraph(&topo, config.stem_degree);
    let trace: TrialTrace = run_trial_seeded(
        &topo,
        &routes,
        Protocol::DandelionStem,
        &config,
        0,
        [9; 32],
        SEED,
    );

    assert!(
        trace.stem_hop_before_fluff,
        "no stem hop before fluff: {trace:?}"
    );
    assert!(
        trace.reached * 100 > 95 * NODES,
        "coverage {}/{NODES}",
        trace.reached
    );
}
