// Sources: fixtures; dstem/src/disconnect.rs:11-31.
use dstem::tracker::{InFlightRecord as R, InFlightTracker as T};
use dstem::{disconnect::handle_peer_drop, peers::PeerRegistry, route::StemRouteTable};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn disconnect_returns_two_hashes_and_removes_peer() {
    let tracker = T::default();
    let (x, registry) = ([0x78; 64], PeerRegistry::default());
    let routes = StemRouteTable::default();
    registry.add_peer(x);
    routes.update_routes(vec![x]);
    let a = R {
        tx_hash: [0xaa; 32],
        forwarded_to: x,
        deadline_ms: 1_000,
    };
    let b = R {
        tx_hash: [0xbb; 32],
        ..a
    };
    tracker.track(a);
    tracker.track(b);
    let stranded = handle_peer_drop(&x, &registry, &routes, &tracker);
    assert_eq!(stranded, vec![a.tx_hash, b.tx_hash]);
    assert!(!registry.peers().contains(&x));
    let mut rng = StdRng::seed_from_u64(68);
    assert_eq!(routes.select_forwarding_peer(&mut rng), None);
}
