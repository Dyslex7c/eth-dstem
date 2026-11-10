use eth_tx_sim::adversary::first_spy::FirstSpyEstimator;

const SPY_1: usize = 101;
const SPY_2: usize = 102;
const NODE_A: usize = 1;
const NODE_B: usize = 2;

#[test]
fn first_spy_identifies_earliest_delivering_peer_across_spies() {
    let mut estimator = FirstSpyEstimator::new();

    estimator.record_observation(SPY_1, NODE_A, 15);
    estimator.record_observation(SPY_2, NODE_B, 8);

    assert_eq!(estimator.estimate_origin(), Some(NODE_B));
    assert_ne!(estimator.estimate_origin(), Some(NODE_A));
}
