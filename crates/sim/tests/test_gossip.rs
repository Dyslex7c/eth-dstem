use eth_tx_sim::engine::SimulationEngine;
use eth_tx_sim::event::EventType;
use eth_tx_sim::gossip::propagate_eth68;
use eth_tx_sim::topology::NetworkTopology;
use std::collections::BTreeSet;

const NODE_COUNT: usize = 16;
const ORIGIN: usize = 0;
const LINK_LATENCY_MS: u64 = 40;

#[test]
fn wheel_of_16_nodes_splits_direct_and_announce_by_ceil_sqrt() {
    // 16-node wheel: hub at 0 linked to all 15 spokes, plus a ring over 1..16.
    // Every node is connected; node 0 has 15 peers and each spoke has 3.
    let mut topo = NetworkTopology::new(NODE_COUNT, NODE_COUNT);
    for spoke in 1..NODE_COUNT {
        topo.add_symmetric_link(ORIGIN, spoke).unwrap();
    }
    for spoke in 1..NODE_COUNT {
        let next = if spoke + 1 < NODE_COUNT { spoke + 1 } else { 1 };
        topo.add_symmetric_link(spoke, next).unwrap();
    }

    let total_peers = topo.adjacency[&ORIGIN].len();
    assert_eq!(total_peers, 15);
    let expected_direct = (total_peers as f64).sqrt().ceil() as usize;
    let expected_announced = total_peers - expected_direct;
    assert_eq!((expected_direct, expected_announced), (4, 11));

    let tx_hash = [0xab; 32];
    let mut engine = SimulationEngine::new();
    propagate_eth68(&mut engine, &topo, ORIGIN, tx_hash, LINK_LATENCY_MS);

    let mut direct_peers = BTreeSet::new();
    let mut announced_peers = BTreeSet::new();
    while let Some(event) = engine.step() {
        assert_eq!(event.from, ORIGIN);
        assert_eq!(event.scheduled_time_ms, LINK_LATENCY_MS);
        assert_eq!(engine.current_time_ms, LINK_LATENCY_MS);
        match event.payload {
            EventType::DeliverTx { tx_hash: got } => {
                assert_eq!(got, tx_hash);
                assert!(
                    direct_peers.insert(event.to),
                    "duplicate direct to {}",
                    event.to
                );
            }
            EventType::AnnounceHash { tx_hash: got } => {
                assert_eq!(got, tx_hash);
                assert!(
                    announced_peers.insert(event.to),
                    "duplicate announce to {}",
                    event.to
                );
            }
            EventType::FetchRequest { .. } => {
                panic!("unexpected FetchRequest from propagate_eth68")
            }
        }
    }

    assert!(engine.queue.is_empty());
    assert_eq!(engine.step(), None);
    assert_eq!(direct_peers.len(), expected_direct);
    assert_eq!(announced_peers.len(), expected_announced);
    assert!(direct_peers.is_disjoint(&announced_peers));

    let all_notified: BTreeSet<_> = direct_peers.union(&announced_peers).copied().collect();
    assert_eq!(all_notified, topo.adjacency[&ORIGIN]);
}
