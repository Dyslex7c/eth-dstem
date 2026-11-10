use crate::engine::SimulationEngine;
use crate::event::{Event, EventType};
use crate::node::NodeId;
use crate::topology::NetworkTopology;
use std::collections::BTreeSet;

/// Propagate one transaction from `origin` using the geth `eth/68` broadcast pattern:
/// `ceil(sqrt(peers))` peers receive the full transaction (`DeliverTx`) and every other
/// peer receives only the announcement hash (`AnnounceHash`).
///
/// Both event kinds are scheduled one `link_latency_ms` hop after
/// `engine.current_time_ms`; this simulator models a single one-way hop.
///
/// Peers are the deduplicated union of the origin's inbound and outbound lists, so a
/// symmetric link counts once. Recipient selection is deterministic but keyed by
/// `tx_hash`, so the direct set does not systematically favour low node IDs and repeated
/// runs are reproducible.
pub fn propagate_eth68(
    engine: &mut SimulationEngine,
    topo: &NetworkTopology,
    origin: NodeId,
    tx_hash: [u8; 32],
    link_latency_ms: u64,
) {
    let Some(origin_node) = topo.nodes.get(origin) else {
        return;
    };

    let mut peers: Vec<NodeId> = origin_node
        .inbound_peers
        .iter()
        .chain(origin_node.outbound_peers.iter())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|peer| *peer != origin && *peer < topo.nodes.len())
        .collect();

    peers.sort_by(|a, b| {
        peer_rank(&tx_hash, *a)
            .cmp(&peer_rank(&tx_hash, *b))
            .then_with(|| a.cmp(b))
    });

    let direct_count = ceil_sqrt(peers.len());
    let scheduled_time_ms = engine.current_time_ms.saturating_add(link_latency_ms);

    for (index, peer) in peers.into_iter().enumerate() {
        let payload = if index < direct_count {
            EventType::DeliverTx { tx_hash }
        } else {
            EventType::AnnounceHash { tx_hash }
        };
        engine.schedule(Event {
            scheduled_time_ms,
            from: origin,
            to: peer,
            payload,
        });
    }
}

/// Smallest `k` with `k * k >= n`; the `eth/68` direct-broadcast fan-out.
fn ceil_sqrt(n: usize) -> usize {
    if n <= 1 {
        return n;
    }
    let root = n.isqrt();
    if root * root == n {
        root
    } else {
        root + 1
    }
}

/// FNV-1a over `tx_hash || peer` used only to order recipients deterministically.
fn peer_rank(tx_hash: &[u8; 32], peer: NodeId) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for byte in tx_hash.iter().copied().chain(peer.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    fn star_topology(peer_count: usize) -> NetworkTopology {
        let mut topo = NetworkTopology::new(peer_count + 1, peer_count.max(1));
        for peer in 1..=peer_count {
            topo.add_symmetric_link(0, peer).unwrap();
        }
        topo
    }

    fn drain(engine: &mut SimulationEngine) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = engine.step() {
            events.push(event);
        }
        events
    }

    fn split(events: &[Event]) -> (Vec<NodeId>, Vec<NodeId>) {
        let mut direct = Vec::new();
        let mut announced = Vec::new();
        for event in events {
            assert_eq!(event.from, 0);
            match event.payload {
                EventType::DeliverTx { .. } => direct.push(event.to),
                EventType::AnnounceHash { .. } => announced.push(event.to),
                EventType::FetchRequest { .. } => panic!("unexpected fetch request"),
            }
        }
        (direct, announced)
    }

    #[test]
    fn nine_peers_split_three_direct_six_announce() {
        let topo = star_topology(9);
        let mut engine = SimulationEngine::new();

        propagate_eth68(&mut engine, &topo, 0, hash(7), 30);
        assert_eq!(engine.queue.len(), 9);

        let events = drain(&mut engine);
        let (direct, announced) = split(&events);

        assert_eq!(direct.len(), 3);
        assert_eq!(announced.len(), 6);
        assert!(events.iter().all(|e| e.scheduled_time_ms == 30));
        assert!(events
            .iter()
            .all(|e| e.payload == EventType::DeliverTx { tx_hash: hash(7) }
                || e.payload == EventType::AnnounceHash { tx_hash: hash(7) }));
    }

    #[test]
    fn non_square_peer_counts_round_up() {
        for peers in 1..=20usize {
            let topo = star_topology(peers);
            let mut engine = SimulationEngine::new();
            propagate_eth68(&mut engine, &topo, 0, hash(3), 1);
            let events = drain(&mut engine);
            let (direct, announced) = split(&events);

            let expected = (peers as f64).sqrt().ceil() as usize;
            assert_eq!(direct.len(), expected, "peers={peers}");
            assert_eq!(direct.len() + announced.len(), peers, "peers={peers}");
            assert_eq!(direct.len(), ceil_sqrt(peers), "peers={peers}");
        }
    }

    #[test]
    fn every_peer_is_covered_exactly_once() {
        let topo = star_topology(10);
        let mut engine = SimulationEngine::new();
        propagate_eth68(&mut engine, &topo, 0, hash(1), 5);

        let events = drain(&mut engine);
        let recipients: BTreeSet<_> = events.iter().map(|event| event.to).collect();
        let expected: BTreeSet<_> = (1..=10).collect();

        assert_eq!(events.len(), 10);
        assert_eq!(recipients, expected);
    }

    #[test]
    fn symmetric_links_are_not_double_counted() {
        let topo = star_topology(4);
        assert_eq!(topo.nodes[0].inbound_peers.len(), 4);
        assert_eq!(topo.nodes[0].outbound_peers.len(), 4);

        let mut engine = SimulationEngine::new();
        propagate_eth68(&mut engine, &topo, 0, hash(2), 5);

        assert_eq!(engine.queue.len(), 4);
        let (direct, announced) = split(&drain(&mut engine));
        assert_eq!(direct.len(), 2);
        assert_eq!(announced.len(), 2);
    }

    #[test]
    fn schedules_relative_to_current_engine_time() {
        let topo = star_topology(9);
        let mut engine = SimulationEngine::new();
        engine.schedule(Event {
            scheduled_time_ms: 100,
            from: 1,
            to: 2,
            payload: EventType::DeliverTx { tx_hash: hash(9) },
        });
        engine.step().unwrap();
        assert_eq!(engine.current_time_ms, 100);

        propagate_eth68(&mut engine, &topo, 0, hash(4), 250);
        assert!(engine.queue.iter().all(|e| e.scheduled_time_ms == 350));

        let events = drain(&mut engine);
        assert_eq!(events.len(), 9);
        assert!(events.iter().all(|e| e.scheduled_time_ms == 350));
        assert_eq!(engine.current_time_ms, 350);
    }

    #[test]
    fn zero_latency_schedules_at_current_time() {
        let topo = star_topology(3);
        let mut engine = SimulationEngine::new();
        propagate_eth68(&mut engine, &topo, 0, hash(5), 0);

        let events = drain(&mut engine);
        assert_eq!(events.len(), 3);
        assert!(events.iter().all(|e| e.scheduled_time_ms == 0));
    }

    #[test]
    fn infinite_latency_saturates_instead_of_overflowing() {
        let topo = star_topology(2);
        let mut engine = SimulationEngine::new();
        propagate_eth68(&mut engine, &topo, 0, hash(6), u64::MAX);

        assert!(engine.queue.iter().all(|e| e.scheduled_time_ms == u64::MAX));
    }

    #[test]
    fn isolated_or_unknown_origin_schedules_nothing() {
        let topo = NetworkTopology::new(3, 5);
        let mut engine = SimulationEngine::new();

        propagate_eth68(&mut engine, &topo, 1, hash(0), 10);
        propagate_eth68(&mut engine, &topo, 3, hash(0), 10);

        assert!(engine.queue.is_empty());
        assert_eq!(engine.current_time_ms, 0);
    }

    #[test]
    fn selection_is_deterministic_for_the_same_hash() {
        let topo = star_topology(10);
        let mut first = SimulationEngine::new();
        let mut second = SimulationEngine::new();

        propagate_eth68(&mut first, &topo, 0, hash(42), 0);
        propagate_eth68(&mut second, &topo, 0, hash(42), 0);

        assert_eq!(drain(&mut first), drain(&mut second));
    }

    #[test]
    fn ceil_sqrt_matches_integer_definition() {
        for n in 0..1000usize {
            let value = ceil_sqrt(n);
            assert!(value * value >= n);
            if value > 0 {
                assert!((value - 1) * (value - 1) < n);
            }
        }
        assert_eq!(ceil_sqrt(0), 0);
        assert_eq!(ceil_sqrt(1), 1);
        assert_eq!(ceil_sqrt(9), 3);
        assert_eq!(ceil_sqrt(10), 4);
    }
}
