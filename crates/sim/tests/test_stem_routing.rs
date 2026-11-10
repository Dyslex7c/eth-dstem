use eth_tx_sim::stem::config::StemConfig;
use eth_tx_sim::stem::digraph::StemTopology;
use eth_tx_sim::stem::routing::{evaluate_routing_step, RoutingDecision};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

#[test]
fn forwards_about_eighty_percent_split_evenly_across_stem_peers() {
    let stem = StemTopology {
        anonymity_routes: HashMap::from([(0, vec![1, 2])]),
    };
    let config = StemConfig::new(0.80, 8_000, 2).unwrap();
    let mut rng = StdRng::seed_from_u64(0x5354_454d);
    let mut hits = [0usize; 3];
    for _ in 0..1_000 {
        match evaluate_routing_step(&mut rng, 0, &stem, &config) {
            RoutingDecision::ForwardStem { next_hop } => hits[next_hop] += 1,
            RoutingDecision::BroadcastFluff => {}
        }
    }

    assert_eq!(hits[0], 0, "a node must never stem-forward to itself");
    let forward_ratio = (hits[1] + hits[2]) as f64 / 1_000.0;
    assert!((0.76..=0.84).contains(&forward_ratio), "{forward_ratio}");
    for peer in [1, 2] {
        let share = hits[peer] as f64 / 1_000.0;
        assert!((0.35..=0.45).contains(&share), "peer {peer}: {share}");
    }
}
