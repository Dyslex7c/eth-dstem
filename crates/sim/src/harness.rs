//! Sources: [S14] local model — graph construction, hop latency, and the two-hop hash
//! fetch are identical to `crates/sim/src/bin/run_baseline.rs:23-92`; [S15] experiment inputs:
//! 100 trials, 200 nodes, 10% Sybils. These are model inputs, not client defaults, and
//! correspondence to real Ethereum origin leakage is `unknown`.

use crate::adversary::first_spy::FirstSpyEstimator;
use crate::adversary::timing::TimingEstimator;
use crate::engine::SimulationEngine;
use crate::event::{Event, EventType};
use crate::gossip::propagate_eth68;
use crate::metrics::SimulationMetrics;
use crate::node::{NodeId, NodeType};
use crate::stem::config::StemConfig;
use crate::stem::digraph::{build_stem_digraph, StemTopology};
use crate::stem::failsafe::FailSafeTracker;
use crate::stem::lifecycle::step_stem_propagation_with;
use crate::stem::routing::RoutingDecision;
use crate::topology::NetworkTopology;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, RngCore, SeedableRng};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const TRIALS: usize = 100;
pub const NETWORK_SIZE: usize = 200;
pub const SYBIL_PCT: usize = 10;
pub const SYBIL_COUNT: usize = NETWORK_SIZE * SYBIL_PCT / 100;
pub const MODEL_MAX_PEERS: usize = 10;
pub const MODEL_LINK_LATENCY_MS: u64 = 10;
pub const MODEL_SEED: u64 = 0x0065_7468_0036_3800;

/// Relay rule under test. `Eth68Flood` is the unmitigated baseline; `DandelionStem`
/// routes through the stem digraph and fluffs on coin failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    Eth68Flood,
    DandelionStem,
}

/// Seeded 200-node graph: shuffled ring plus random links, then 10% marked Sybil.
pub fn random_topology(rng: &mut StdRng) -> NetworkTopology {
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

/// One trial. Returns whether the first-spy estimator named the true origin, and the
/// simulated millisecond of the first Sybil observation (0 if no Sybil ever observed).
pub fn run_trial(
    topo: &NetworkTopology,
    routes: &StemTopology,
    protocol: Protocol,
    config: &StemConfig,
    origin: NodeId,
    tx_hash: [u8; 32],
) -> (bool, u64) {
    let (hit, first_ms, _) = run_trial_observed(topo, routes, protocol, config, origin, tx_hash);
    (hit, first_ms)
}

/// Outcome of a full trial, including liveness (coverage) and whether the stem phase
/// actually preceded the first fluff broadcast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialTrace {
    pub hit: bool,
    pub first_ms: u64,
    pub observations: Vec<(NodeId, u64)>,
    pub reached: usize,
    pub stem_hop_before_fluff: bool,
}

/// Deterministic trial: `seed` fixes the stem coin, so repeated runs are identical.
pub fn run_trial_seeded(
    topo: &NetworkTopology,
    routes: &StemTopology,
    protocol: Protocol,
    config: &StemConfig,
    origin: NodeId,
    tx_hash: [u8; 32],
    seed: u64,
) -> TrialTrace {
    simulate(
        topo,
        routes,
        protocol,
        config,
        origin,
        tx_hash,
        &mut StdRng::seed_from_u64(seed),
    )
}

/// Same trial with an unseeded coin, additionally returning every Sybil's first arrival
/// time as `(sybil_id, ms)` ascending by Sybil id, for arrival-time (timing) adversaries.
/// Propagation runs to completion rather than stopping at the first observation. The
/// first-spy verdict is unaffected: `estimate_origin` takes the earliest observation, and
/// strictly later arrivals can never beat it.
pub fn run_trial_observed(
    topo: &NetworkTopology,
    routes: &StemTopology,
    protocol: Protocol,
    config: &StemConfig,
    origin: NodeId,
    tx_hash: [u8; 32],
) -> (bool, u64, Vec<(NodeId, u64)>) {
    let trace = simulate(
        topo,
        routes,
        protocol,
        config,
        origin,
        tx_hash,
        &mut StdRng::from_entropy(),
    );
    (trace.hit, trace.first_ms, trace.observations)
}

/// Core simulation. Dandelion++ has a per-transaction phase: once any relay fluffs the
/// transaction, every later relay of it is fluff (a node in the fluff phase does not
/// re-enter the stem). Stem forwards register a `FailSafeTracker` deadline; when it
/// expires the forwarding node fluffs, so a stem chain that dead-ends (every node relays
/// only on first receipt) cannot silently drop the transaction.
fn simulate(
    topo: &NetworkTopology,
    routes: &StemTopology,
    protocol: Protocol,
    config: &StemConfig,
    origin: NodeId,
    tx_hash: [u8; 32],
    rng: &mut StdRng,
) -> TrialTrace {
    let mut engine = SimulationEngine::new();
    let mut estimator = FirstSpyEstimator::new();
    let mut fail_safe = FailSafeTracker::new();
    let mut arrivals = BTreeMap::new();
    let mut stem_waiting = BTreeSet::new();
    let mut has_tx = vec![false; topo.nodes.len()];
    let mut fetching = vec![false; topo.nodes.len()];
    let mut first_observation_ms = None;
    let mut fluffed = false;
    let mut stem_hop_before_fluff = false;
    has_tx[origin] = true;
    relay(
        &mut engine,
        topo,
        routes,
        protocol,
        config,
        origin,
        tx_hash,
        rng,
        &mut fail_safe,
        &mut stem_waiting,
        &mut fluffed,
    );

    loop {
        let Some(event) = engine.step() else {
            let Some(deadline) = fail_safe
                .pending_deadlines
                .values()
                .map(|&(_, deadline_ms)| deadline_ms)
                .min()
            else {
                break;
            };
            engine.current_time_ms = deadline;
            fluffed |= expire(&mut fail_safe, &mut engine, topo, deadline);
            continue;
        };
        fluffed |= expire(&mut fail_safe, &mut engine, topo, event.scheduled_time_ms);
        if is_sybil_receipt(topo, &event) {
            estimator.record_observation(event.to, event.from, event.scheduled_time_ms);
            arrivals.entry(event.to).or_insert(event.scheduled_time_ms);
            first_observation_ms.get_or_insert(event.scheduled_time_ms);
        }
        match event.payload {
            EventType::AnnounceHash { tx_hash } if !has_tx[event.to] && !fetching[event.to] => {
                fetching[event.to] = true;
                respond(
                    &mut engine,
                    event.to,
                    event.from,
                    EventType::FetchRequest { tx_hash },
                );
            }
            EventType::FetchRequest { tx_hash } if has_tx[event.to] => {
                respond(
                    &mut engine,
                    event.to,
                    event.from,
                    EventType::DeliverTx { tx_hash },
                );
            }
            EventType::DeliverTx { tx_hash } if !has_tx[event.to] => {
                has_tx[event.to] = true;
                let was_waiting = stem_waiting.remove(&event.to);
                if was_waiting && !fluffed && topo.nodes[event.to].node_type != NodeType::Sybil {
                    stem_hop_before_fluff = true;
                }
                relay(
                    &mut engine,
                    topo,
                    routes,
                    protocol,
                    config,
                    event.to,
                    tx_hash,
                    rng,
                    &mut fail_safe,
                    &mut stem_waiting,
                    &mut fluffed,
                );
            }
            _ => {}
        }
    }
    TrialTrace {
        hit: estimator.estimate_origin() == Some(origin),
        first_ms: first_observation_ms.unwrap_or(0),
        observations: arrivals.into_iter().collect(),
        reached: has_tx.iter().filter(|received| **received).count(),
        stem_hop_before_fluff,
    }
}

/// Fluffs every deadline at or before `now`; returns whether any broadcast fired. Firing
/// means the transaction is now flooding, so the tracker is cleared of the rest.
fn expire(
    fail_safe: &mut FailSafeTracker,
    engine: &mut SimulationEngine,
    topo: &NetworkTopology,
    now: u64,
) -> bool {
    let expired = fail_safe.check_expired(now);
    for (tx_hash, origin) in &expired {
        propagate_eth68(engine, topo, *origin, *tx_hash, MODEL_LINK_LATENCY_MS);
    }
    if !expired.is_empty() {
        fail_safe.pending_deadlines.clear();
    }
    !expired.is_empty()
}

/// Maximum-likelihood origin under the shortest-path arrival model: the candidate whose
/// expected arrival times best fit the observed Sybil times (`adversary/timing.rs:22-58`).
/// Ties resolve to the lowest node id. `None` when nothing was observed or no candidate
/// reaches every observer.
pub fn timing_estimate(topo: &NetworkTopology, observations: &[(NodeId, u64)]) -> Option<NodeId> {
    let mut estimator = TimingEstimator::new();
    for &(sybil, timestamp_ms) in observations {
        estimator.record_observation(sybil, timestamp_ms);
    }
    (0..topo.nodes.len())
        .map(|candidate| {
            let score = estimator.score_candidate_origin(topo, candidate, MODEL_LINK_LATENCY_MS);
            (score, candidate)
        })
        .filter(|(score, _)| score.is_finite())
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, candidate)| candidate)
}

/// BFS hop count from `from` to `to` over the symmetric adjacency; `usize::MAX` when
/// either id is out of bounds or the pair is disconnected.
pub fn hop_distance(topo: &NetworkTopology, from: NodeId, to: NodeId) -> usize {
    if from >= topo.nodes.len() || to >= topo.nodes.len() {
        return usize::MAX;
    }
    let mut distances = vec![usize::MAX; topo.nodes.len()];
    let mut queue = VecDeque::from([from]);
    distances[from] = 0;
    while let Some(node) = queue.pop_front() {
        if node == to {
            return distances[to];
        }
        for &peer in topo.adjacency.get(&node).into_iter().flatten() {
            if distances[peer] == usize::MAX {
                distances[peer] = distances[node] + 1;
                queue.push_back(peer);
            }
        }
    }
    distances[to]
}

/// Draws a uniformly random honest node, marks it `Target`, and returns its id. The RNG
/// call order matches `run_baseline`, so seeded experiments stay reproducible.
pub fn draw_origin(topo: &mut NetworkTopology, rng: &mut StdRng) -> NodeId {
    let honest: Vec<NodeId> = topo
        .nodes
        .iter()
        .filter(|node| node.node_type == NodeType::Honest)
        .map(|node| node.id)
        .collect();
    let origin = honest[rng.gen_range(0..honest.len())];
    topo.nodes[origin].node_type = NodeType::Target;
    origin
}

/// `TRIALS` trials on freshly seeded graphs. The RNG call order per trial matches
/// `run_baseline`, so `Protocol::Eth68Flood` reproduces its published precision.
pub fn run_experiment(protocol: Protocol) -> SimulationMetrics {
    let mut rng = StdRng::seed_from_u64(MODEL_SEED);
    let config = StemConfig::default();
    let mut metrics = SimulationMetrics::new();
    for _ in 0..TRIALS {
        let mut topo = random_topology(&mut rng);
        let routes = build_stem_digraph(&topo, config.stem_degree);
        let origin = draw_origin(&mut topo, &mut rng);
        let mut tx_hash = [0; 32];
        rng.fill_bytes(&mut tx_hash);
        let (deanonymized, first_ms) =
            run_trial(&topo, &routes, protocol, &config, origin, tx_hash);
        metrics.record_result(deanonymized, first_ms, hops(first_ms));
    }
    metrics
}

/// Simulated hops elapsed before the first Sybil observation.
pub fn hops(first_observation_ms: u64) -> usize {
    (first_observation_ms / MODEL_LINK_LATENCY_MS) as usize
}

/// Relays from `node`. In the fluff phase the transaction floods; only the stem phase
/// consults the coin. Stem forwards arm a fail-safe deadline for `node` and record the
/// next hop so a later receipt can be attributed to the stem phase.
#[allow(clippy::too_many_arguments)]
fn relay(
    engine: &mut SimulationEngine,
    topo: &NetworkTopology,
    routes: &StemTopology,
    protocol: Protocol,
    config: &StemConfig,
    node: NodeId,
    tx_hash: [u8; 32],
    rng: &mut StdRng,
    fail_safe: &mut FailSafeTracker,
    stem_waiting: &mut BTreeSet<NodeId>,
    fluffed: &mut bool,
) {
    match protocol {
        Protocol::DandelionStem if !*fluffed => {
            let decision = step_stem_propagation_with(
                rng,
                engine,
                topo,
                routes,
                config,
                tx_hash,
                node,
                MODEL_LINK_LATENCY_MS,
            );
            match decision {
                RoutingDecision::ForwardStem { next_hop } => {
                    stem_waiting.insert(next_hop);
                    let deadline = engine.current_time_ms + config.fail_safe_timeout_ms;
                    fail_safe.register_stem_tx(tx_hash, node, deadline);
                }
                RoutingDecision::BroadcastFluff => {
                    *fluffed = true;
                    fail_safe.pending_deadlines.clear();
                }
            }
        }
        _ => propagate_eth68(engine, topo, node, tx_hash, MODEL_LINK_LATENCY_MS),
    }
}

fn respond(engine: &mut SimulationEngine, from: NodeId, to: NodeId, payload: EventType) {
    engine.schedule(Event {
        scheduled_time_ms: engine.current_time_ms + MODEL_LINK_LATENCY_MS,
        from,
        to,
        payload,
    });
}

fn is_sybil_receipt(topo: &NetworkTopology, event: &Event) -> bool {
    topo.nodes[event.to].node_type == NodeType::Sybil
        && topo.nodes[event.from].node_type != NodeType::Sybil
        && matches!(
            event.payload,
            EventType::AnnounceHash { .. } | EventType::DeliverTx { .. }
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_graph_is_deterministic_connected_and_ten_percent_sybil() {
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
        let mut seen = vec![false; NETWORK_SIZE];
        let mut pending = vec![0];
        seen[0] = true;
        while let Some(node) = pending.pop() {
            for &peer in &topo.adjacency[&node] {
                if !seen[peer] {
                    seen[peer] = true;
                    pending.push(peer);
                }
            }
        }
        assert!(seen.into_iter().all(|visited| visited));
        assert_eq!(
            topo,
            random_topology(&mut StdRng::seed_from_u64(MODEL_SEED))
        );
    }

    #[test]
    fn adjacent_sybil_deanonymizes_under_both_protocols() {
        let mut topo = NetworkTopology::new(2, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        let routes = build_stem_digraph(&topo, StemConfig::default().stem_degree);
        topo.nodes[1].node_type = NodeType::Sybil;
        for protocol in [Protocol::Eth68Flood, Protocol::DandelionStem] {
            let (hit, first_ms) =
                run_trial(&topo, &routes, protocol, &StemConfig::default(), 0, [1; 32]);
            assert!(hit, "{protocol:?} missed a directly adjacent Sybil");
            assert_eq!(first_ms, MODEL_LINK_LATENCY_MS);
            assert_eq!(hops(first_ms), 1);
        }
    }

    #[test]
    fn distant_sybil_blames_the_relay_not_the_origin() {
        let mut topo = NetworkTopology::new(3, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        topo.add_symmetric_link(1, 2).unwrap();
        let routes = build_stem_digraph(&topo, StemConfig::default().stem_degree);
        topo.nodes[2].node_type = NodeType::Sybil;
        let (hit, _) = run_trial(
            &topo,
            &routes,
            Protocol::Eth68Flood,
            &StemConfig::default(),
            0,
            [2; 32],
        );
        assert!(!hit, "flooding must not be credited with an impossible hit");
    }

    #[test]
    fn no_sybil_observation_is_a_miss_with_zero_latency() {
        let mut topo = NetworkTopology::new(3, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        let routes = build_stem_digraph(&topo, StemConfig::default().stem_degree);
        topo.nodes[2].node_type = NodeType::Sybil;
        assert_eq!(
            run_trial(
                &topo,
                &routes,
                Protocol::Eth68Flood,
                &StemConfig::default(),
                0,
                [3; 32]
            ),
            (false, 0)
        );
    }

    #[test]
    fn hop_distance_is_symmetric_and_reports_unreachable_pairs() {
        let mut topo = NetworkTopology::new(5, 2);
        for (a, b) in [(0, 1), (1, 2), (2, 3)] {
            topo.add_symmetric_link(a, b).unwrap();
        }
        assert_eq!(hop_distance(&topo, 0, 3), 3);
        assert_eq!(hop_distance(&topo, 3, 0), 3);
        assert_eq!(hop_distance(&topo, 2, 2), 0);
        assert_eq!(hop_distance(&topo, 0, 4), usize::MAX);
        assert_eq!(hop_distance(&topo, 0, 9), usize::MAX);
        assert_eq!(hop_distance(&topo, 9, 0), usize::MAX);
    }

    #[test]
    fn timing_estimate_picks_the_candidate_matching_arrivals() {
        let mut topo = NetworkTopology::new(5, 2);
        for (a, b) in [(0, 1), (1, 2), (2, 3)] {
            topo.add_symmetric_link(a, b).unwrap();
        }
        topo.nodes[2].node_type = NodeType::Sybil;
        topo.nodes[3].node_type = NodeType::Sybil;
        // Sybil 2 is 2 hops from node 0 (20ms) and Sybil 3 is 3 hops (30ms).
        assert_eq!(timing_estimate(&topo, &[(2, 20), (3, 30)]), Some(0));
        // The same arrivals are 1 and 2 hops from node 1, so node 1 is also consistent.
        assert_eq!(timing_estimate(&topo, &[(2, 10), (3, 20)]), Some(1));
        assert_eq!(timing_estimate(&topo, &[]), None);
        // A Sybil outside every candidate's component makes the estimate undefined.
        assert_eq!(timing_estimate(&topo, &[(2, 20), (4, 10)]), None);
    }

    #[test]
    fn observed_trial_collects_every_arrival_without_changing_the_verdict() {
        let mut topo = NetworkTopology::new(4, 4);
        for (a, b) in [(0, 1), (1, 2), (0, 3)] {
            topo.add_symmetric_link(a, b).unwrap();
        }
        topo.nodes[2].node_type = NodeType::Sybil;
        topo.nodes[3].node_type = NodeType::Sybil;
        let routes = build_stem_digraph(&topo, StemConfig::default().stem_degree);
        let config = StemConfig::default();
        let (hit, first_ms, observations) =
            run_trial_observed(&topo, &routes, Protocol::Eth68Flood, &config, 0, [4; 32]);

        // Sybil 3 hears from the origin at 10ms; Sybil 2 hears from honest relay 1 at
        // 20ms. Both are retained, ascending by Sybil id.
        assert_eq!(observations, vec![(2, 20), (3, 10)]);
        assert_eq!(first_ms, 10, "first observation stays the earliest arrival");
        assert!(hit, "the earliest observer still names the true origin");
        assert_eq!(
            run_trial(&topo, &routes, Protocol::Eth68Flood, &config, 0, [4; 32]),
            (hit, first_ms)
        );
    }

    #[test]
    fn sybil_to_sybil_arrivals_are_not_observed() {
        let mut topo = NetworkTopology::new(3, 2);
        topo.add_symmetric_link(0, 1).unwrap();
        topo.add_symmetric_link(1, 2).unwrap();
        topo.nodes[1].node_type = NodeType::Sybil;
        topo.nodes[2].node_type = NodeType::Sybil;
        let routes = build_stem_digraph(&topo, StemConfig::default().stem_degree);
        let config = StemConfig::default();
        let (hit, first_ms, observations) =
            run_trial_observed(&topo, &routes, Protocol::Eth68Flood, &config, 0, [5; 32]);

        // Sybil 2 is reached by Sybil 1, which `is_sybil_receipt` excludes, matching the
        // first-spy convention already used by `run_baseline`.
        assert_eq!(observations, vec![(1, 10)]);
        assert_eq!(first_ms, 10);
        assert!(hit);
    }
}
