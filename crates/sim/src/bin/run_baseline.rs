#![doc = "Sources: [S14] local model: crates/sim/src/gossip.rs:18-60, crates/sim/src/engine.rs:15-24, crates/sim/src/adversary/first_spy.rs:14-27; [S15] user-specified experiment: 100 trials, 200 nodes, 10% Sybils.
[S14]/[S15] are local source IDs. Peer cap, latency, and seed are experiment inputs, not client defaults.
The model uses constant hop latency, immediate hash fetches, and passive colluding Sybils following normal relay behavior. It omits client fetch timers, pool admission, and queue limits; correspondence to real Ethereum origin leakage is unknown."]

use eth_tx_sim::adversary::first_spy::FirstSpyEstimator;
use eth_tx_sim::engine::SimulationEngine;
use eth_tx_sim::event::{Event, EventType};
use eth_tx_sim::gossip::propagate_eth68;
use eth_tx_sim::node::{NodeId, NodeType};
use eth_tx_sim::topology::NetworkTopology;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};

const TRIALS: usize = 100;
const NETWORK_SIZE: usize = 200;
const SYBIL_PCT: usize = 10;
const SYBIL_COUNT: usize = NETWORK_SIZE * SYBIL_PCT / 100;
const MODEL_MAX_PEERS: usize = 10;
const MODEL_LINK_LATENCY_MS: u64 = 10;
const MODEL_SEED: u64 = 0x0065_7468_0036_3800;

fn random_topology(rng: &mut StdRng) -> NetworkTopology {
    let mut topo = NetworkTopology::new(NETWORK_SIZE, MODEL_MAX_PEERS);
    let mut ids: Vec<NodeId> = (0..NETWORK_SIZE).collect();
    ids.shuffle(rng);
    for index in 0..NETWORK_SIZE {
        topo.add_symmetric_link(ids[index], ids[(index + 1) % NETWORK_SIZE])
            .expect("ring links fit within the peer cap");
    }
    for _ in 0..NETWORK_SIZE * MODEL_MAX_PEERS * 2 {
        let a = rng.gen_range(0..NETWORK_SIZE);
        let b = rng.gen_range(0..NETWORK_SIZE);
        let _ = topo.add_symmetric_link(a, b);
    }
    ids.shuffle(rng);
    for &id in &ids[..SYBIL_COUNT] {
        topo.nodes[id].node_type = NodeType::Sybil;
    }
    topo
}

fn simulate_trial(topo: &NetworkTopology, origin: NodeId, tx_hash: [u8; 32]) -> Option<NodeId> {
    let mut engine = SimulationEngine::new();
    let mut estimator = FirstSpyEstimator::new();
    let mut has_tx = vec![false; topo.nodes.len()];
    let mut fetching = vec![false; topo.nodes.len()];
    let mut first_observation_time = None;
    has_tx[origin] = true;
    propagate_eth68(&mut engine, topo, origin, tx_hash, MODEL_LINK_LATENCY_MS);

    while let Some(event) = engine.step() {
        if first_observation_time.is_some_and(|time| event.scheduled_time_ms > time) {
            break;
        }
        if topo.nodes[event.to].node_type == NodeType::Sybil
            && topo.nodes[event.from].node_type != NodeType::Sybil
            && matches!(
                event.payload,
                EventType::AnnounceHash { .. } | EventType::DeliverTx { .. }
            )
        {
            estimator.record_observation(event.to, event.from, event.scheduled_time_ms);
            first_observation_time = Some(event.scheduled_time_ms);
        }
        match event.payload {
            EventType::AnnounceHash { tx_hash } if !has_tx[event.to] && !fetching[event.to] => {
                fetching[event.to] = true;
                engine.schedule(Event {
                    scheduled_time_ms: engine.current_time_ms + MODEL_LINK_LATENCY_MS,
                    from: event.to,
                    to: event.from,
                    payload: EventType::FetchRequest { tx_hash },
                });
            }
            EventType::FetchRequest { tx_hash } if has_tx[event.to] => {
                engine.schedule(Event {
                    scheduled_time_ms: engine.current_time_ms + MODEL_LINK_LATENCY_MS,
                    from: event.to,
                    to: event.from,
                    payload: EventType::DeliverTx { tx_hash },
                });
            }
            EventType::DeliverTx { tx_hash } if !has_tx[event.to] => {
                has_tx[event.to] = true;
                propagate_eth68(&mut engine, topo, event.to, tx_hash, MODEL_LINK_LATENCY_MS);
            }
            _ => {}
        }
    }
    estimator.estimate_origin()
}

fn main() {
    let mut rng = StdRng::seed_from_u64(MODEL_SEED);
    let mut correct = 0;
    for _ in 0..TRIALS {
        let mut topo = random_topology(&mut rng);
        let honest: Vec<NodeId> = topo
            .nodes
            .iter()
            .filter(|node| node.node_type == NodeType::Honest)
            .map(|node| node.id)
            .collect();
        let origin = honest[rng.gen_range(0..honest.len())];
        topo.nodes[origin].node_type = NodeType::Target;
        let mut tx_hash = [0; 32];
        rng.fill_bytes(&mut tx_hash);
        if simulate_trial(&topo, origin, tx_hash) == Some(origin) {
            correct += 1;
        }
    }
    let top1_precision = correct as f64 / TRIALS as f64;
    println!("trials,sybil_pct,top1_precision");
    println!("{TRIALS},{SYBIL_PCT},{top1_precision:.6}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_graph_has_exact_sybil_fraction_and_no_peer_overflow() {
        let mut rng = StdRng::seed_from_u64(MODEL_SEED);
        let topo = random_topology(&mut rng);
        assert_eq!(topo.nodes.len(), NETWORK_SIZE);
        assert_eq!(
            topo.nodes
                .iter()
                .filter(|node| node.node_type == NodeType::Sybil)
                .count(),
            SYBIL_COUNT
        );
        for node in &topo.nodes {
            assert!(node.outbound_peers.len() <= MODEL_MAX_PEERS);
            assert!(node.inbound_peers.len() <= MODEL_MAX_PEERS);
        }
        let mut visited = vec![false; NETWORK_SIZE];
        let mut pending = vec![0];
        visited[0] = true;
        while let Some(node) = pending.pop() {
            for &peer in &topo.adjacency[&node] {
                if !visited[peer] {
                    visited[peer] = true;
                    pending.push(peer);
                }
            }
        }
        assert!(visited.into_iter().all(|seen| seen));
        let mut repeated = StdRng::seed_from_u64(MODEL_SEED);
        assert_eq!(topo, random_topology(&mut repeated));
    }

    #[test]
    fn directly_connected_spy_identifies_target() {
        let mut topo = NetworkTopology::new(2, 1);
        topo.add_symmetric_link(0, 1).unwrap();
        topo.nodes[0].node_type = NodeType::Target;
        topo.nodes[1].node_type = NodeType::Sybil;
        assert_eq!(simulate_trial(&topo, 0, [1; 32]), Some(0));
    }

    #[test]
    fn distant_spy_identifies_relay_instead_of_true_origin() {
        let mut topo = NetworkTopology::new(3, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        topo.add_symmetric_link(1, 2).unwrap();
        topo.nodes[0].node_type = NodeType::Target;
        topo.nodes[2].node_type = NodeType::Sybil;
        assert_eq!(simulate_trial(&topo, 0, [2; 32]), Some(1));
    }

    #[test]
    fn hash_only_recipient_fetches_before_relaying_to_spy() {
        let mut topo = NetworkTopology::new(6, 4);
        for peer in 1..=4 {
            topo.add_symmetric_link(0, peer).unwrap();
        }
        let tx_hash = [4; 32];
        let mut engine = SimulationEngine::new();
        propagate_eth68(&mut engine, &topo, 0, tx_hash, MODEL_LINK_LATENCY_MS);
        let relay = engine
            .queue
            .iter()
            .find(|event| matches!(event.payload, EventType::AnnounceHash { .. }))
            .unwrap()
            .to;
        topo.add_symmetric_link(relay, 5).unwrap();
        topo.nodes[5].node_type = NodeType::Sybil;
        assert_eq!(simulate_trial(&topo, 0, tx_hash), Some(relay));
    }

    #[test]
    fn simultaneous_observations_are_all_recorded_before_stopping() {
        let mut topo = NetworkTopology::new(5, 2);
        for (a, b) in [(0, 1), (0, 2), (1, 3), (2, 4)] {
            topo.add_symmetric_link(a, b).unwrap();
        }
        topo.nodes[3].node_type = NodeType::Sybil;
        topo.nodes[4].node_type = NodeType::Sybil;
        assert_eq!(simulate_trial(&topo, 0, [5; 32]), Some(1));
    }

    #[test]
    fn absence_of_spy_observations_is_not_counted_as_success() {
        let mut topo = NetworkTopology::new(3, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        topo.nodes[2].node_type = NodeType::Sybil;
        assert_eq!(simulate_trial(&topo, 0, [3; 32]), None);
    }
}
