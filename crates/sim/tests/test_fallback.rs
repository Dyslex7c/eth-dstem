use eth_tx_sim::stem::digraph::StemTopology;
use eth_tx_sim::stem::fallback::handle_stem_forward_failure;
use eth_tx_sim::stem::routing::RoutingDecision;
use std::collections::HashMap;

#[test]
fn last_stem_peer_failing_purges_it_and_fluffs() {
    let mut stem = StemTopology {
        anonymity_routes: HashMap::from([(10, vec![99])]),
    };
    let decision = handle_stem_forward_failure([4; 32], 10, 99, &mut stem);

    assert_eq!(decision, RoutingDecision::BroadcastFluff);
    assert!(stem.get_stem_peers(10).is_empty());
}

#[test]
fn surviving_alternative_is_picked_when_one_of_two_fails() {
    let mut stem = StemTopology {
        anonymity_routes: HashMap::from([(10, vec![99, 42])]),
    };
    let decision = handle_stem_forward_failure([4; 32], 10, 99, &mut stem);

    assert_eq!(decision, RoutingDecision::ForwardStem { next_hop: 42 });
    assert_eq!(stem.get_stem_peers(10), &[42]);
}
