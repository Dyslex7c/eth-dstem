use crate::engine::SimulationEngine;
use crate::event::{Event, EventType};
use crate::gossip::propagate_eth68;
use crate::node::NodeId;
use crate::stem::config::StemConfig;
use crate::stem::digraph::StemTopology;
use crate::stem::routing::{evaluate_routing_step, RoutingDecision};
use crate::topology::NetworkTopology;
use rand::{thread_rng, Rng};

/// Drives one stem/fluff step with the caller's RNG, returning the decision taken so the
/// caller can schedule a fail-safe for stem forwards. Seeded callers get reproducible
/// routing; `step_stem_propagation` keeps the non-deterministic `thread_rng` behaviour.
#[allow(clippy::too_many_arguments)]
pub fn step_stem_propagation_with<R: Rng>(
    rng: &mut R,
    engine: &mut SimulationEngine,
    topo: &NetworkTopology,
    stem_topo: &StemTopology,
    config: &StemConfig,
    tx_hash: [u8; 32],
    current: NodeId,
    latency_ms: u64,
) -> RoutingDecision {
    let decision = evaluate_routing_step(rng, current, stem_topo, config);
    match decision {
        RoutingDecision::ForwardStem { next_hop } => engine.schedule(Event {
            scheduled_time_ms: engine.current_time_ms.saturating_add(latency_ms),
            from: current,
            to: next_hop,
            payload: EventType::AnnounceHash { tx_hash },
        }),
        RoutingDecision::BroadcastFluff => {
            propagate_eth68(engine, topo, current, tx_hash, latency_ms);
        }
    }
    decision
}

#[allow(clippy::too_many_arguments)]
pub fn step_stem_propagation(
    engine: &mut SimulationEngine,
    topo: &NetworkTopology,
    stem_topo: &StemTopology,
    config: &StemConfig,
    tx_hash: [u8; 32],
    current: NodeId,
    latency_ms: u64,
) {
    step_stem_propagation_with(
        &mut thread_rng(),
        engine,
        topo,
        stem_topo,
        config,
        tx_hash,
        current,
        latency_ms,
    );
}
